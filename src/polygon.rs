use std::fmt::Debug;
use std::io::Cursor;

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

#[cfg(feature = "diesel")]
use crate::{
    ewkb::{read_ewkb_header, write_ewkb_header},
    write_to_read_from_sql::{ReadFromSql, WriteToSql},
    points::{read_point_coordinates, write_point_coordinates}
};
use crate::{
    ewkb::{EwkbSerializable, GeometryType, BIG_ENDIAN},
    types::{PointT, Polygon},
};

use crate::points::Dimension;

impl<P> Polygon<P>
where
    P: PointT + Clone,
{
    pub fn new(srid: Option<u32>) -> Self {
        Self::with_capacity(srid, 0)
    }

    pub fn with_capacity(srid: Option<u32>, cap: usize) -> Self {
        Polygon {
            rings: Vec::with_capacity(cap),
            srid,
        }
    }

    pub fn add_ring(&mut self) -> &mut Self {
        self.add_ring_with_capacity(0)
    }

    pub fn add_ring_with_capacity(&mut self, cap: usize) -> &mut Self {
        self.rings.push(Vec::with_capacity(cap));
        self
    }

    pub fn add_point(&mut self, point: P) -> &mut Self {
        if self.rings.last().is_none() {
            self.add_ring();
        }
        self.rings.last_mut().unwrap().push(point);
        self
    }

    pub fn add_points(&mut self, points: impl IntoIterator<Item = P>) -> &mut Self {
        if self.rings.last().is_none() {
            self.add_ring();
        }
        let last = self.rings.last_mut().unwrap();
        for point in points {
            last.push(point);
        }
        self
    }

    pub fn dimension(&self) -> u32 {
        let mut dimension = Dimension::None as u32;
        if let Some(ring) = self.rings.first() {
            if let Some(point) = ring.first() {
                dimension |= point.dimension();
            }
        }
        dimension
    }
}

impl<P> EwkbSerializable for Polygon<P>
where
    P: PointT + Clone,
{
    fn geometry_type(&self) -> u32 {
        GeometryType::Polygon as u32 | self.dimension()
    }
}

#[cfg(feature = "diesel")]
impl<P> WriteToSql for Polygon<P>
where
    P: PointT + Clone + EwkbSerializable,
{
    fn write_to_sql<W>(&self, out: &mut W) -> diesel::serialize::Result
    where
        W: std::io::Write,
    {
        write_ewkb_header(self, self.srid, out)?;
        // number of rings
        out.write_u32::<LittleEndian>(self.rings.len() as u32)?;
        for ring in self.rings.iter() {
            //number of points in ring
            out.write_u32::<LittleEndian>(ring.len() as u32)?;
            for point in ring.iter() {
                write_point_coordinates(point, out)?;
            }
        }
        Ok(diesel::serialize::IsNull::No)
    }
}

#[cfg(feature = "diesel")]
impl<P> ReadFromSql for Polygon<P>
where
    P: PointT + Debug + Clone,
{
    fn read_from_sql(bytes: &[u8]) -> diesel::deserialize::Result<Self> {
        let mut r = Cursor::new(bytes);
        let end = r.read_u8()?;
        if end == BIG_ENDIAN {
            read_polygon::<BigEndian, P>(&mut r)
        } else {
            read_polygon::<LittleEndian, P>(&mut r)
        }
    }
}

#[cfg(feature = "diesel")]
fn read_polygon<T, P>(cursor: &mut Cursor<&[u8]>) -> diesel::deserialize::Result<Polygon<P>>
where
    T: byteorder::ByteOrder,
    P: PointT + Clone,
{
    let g_header = read_ewkb_header::<T>(cursor)?.expect(GeometryType::Polygon)?;
    read_polygon_body::<T, P>(g_header.g_type, g_header.srid, cursor)
}

#[cfg(feature = "diesel")]
pub fn read_polygon_body<T, P>(
    g_type: u32,
    srid: Option<u32>,
    cursor: &mut Cursor<&[u8]>,
) -> diesel::deserialize::Result<Polygon<P>>
where
    T: byteorder::ByteOrder,
    P: PointT + Clone,
{
    let rings_n = cursor.read_u32::<T>()?;
    let mut polygon = Polygon::with_capacity(srid, rings_n as usize);
    for _i in 0..rings_n {
        let points_n = cursor.read_u32::<T>()?;
        polygon.add_ring_with_capacity(points_n as usize);
        for _p in 0..points_n {
            polygon.add_point(read_point_coordinates::<T, P>(cursor, g_type, srid)?);
        }
    }
    Ok(polygon)
}
