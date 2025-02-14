use std::fmt::Debug;
use std::io::Cursor;

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

#[cfg(feature = "diesel")]
use crate::ewkb::{read_ewkb_header, write_ewkb_header};
use crate::{
    ewkb::{EwkbSerializable, GeometryType, BIG_ENDIAN},
    types::{PointT, Polygon},
};

use crate::points::Dimension;
#[cfg(feature = "diesel")]
use crate::points::{read_point_coordinates, write_point_coordinates};
use crate::sql_types::*;

impl<T> Polygon<T>
where
    T: PointT + Clone,
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

    pub fn add_ring<'a>(&'a mut self) -> &mut Self {
        self.add_ring_with_capacity(0)
    }

    pub fn add_ring_with_capacity<'a>(&'a mut self, cap: usize) -> &mut Self {
        self.rings.push(Vec::with_capacity(cap));
        self
    }

    pub fn add_point<'a>(&'a mut self, point: T) -> &mut Self {
        if self.rings.last().is_none() {
            self.add_ring();
        }
        self.rings.last_mut().unwrap().push(point);
        self
    }

    pub fn add_points<'a>(&'a mut self, points: impl IntoIterator<Item = T>) -> &mut Self {
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

impl<T> EwkbSerializable for Polygon<T>
where
    T: PointT + Clone,
{
    fn geometry_type(&self) -> u32 {
        GeometryType::Polygon as u32 | self.dimension()
    }
}

#[cfg(feature = "diesel")]
impl<T> diesel::serialize::ToSql<Geometry, diesel::pg::Pg> for Polygon<T>
where
    T: PointT + Debug + PartialEq + Clone + EwkbSerializable,
{
    fn to_sql(
        &self,
        out: &mut diesel::serialize::Output<diesel::pg::Pg>,
    ) -> diesel::serialize::Result {
        write_polygon(self, self.srid, out)
    }
}

#[cfg(feature = "diesel")]
impl<T> diesel::serialize::ToSql<Geography, diesel::pg::Pg> for Polygon<T>
where
    T: PointT + Debug + PartialEq + Clone + EwkbSerializable,
{
    fn to_sql(
        &self,
        out: &mut diesel::serialize::Output<diesel::pg::Pg>,
    ) -> diesel::serialize::Result {
        write_polygon(self, self.srid, out)
    }
}

#[cfg(feature = "diesel")]
pub fn write_polygon<T>(
    polygon: &Polygon<T>,
    srid: Option<u32>,
    out: &mut diesel::serialize::Output<diesel::pg::Pg>,
) -> diesel::serialize::Result
where
    T: PointT + EwkbSerializable + Clone,
{
    write_ewkb_header(polygon, srid, out)?;
    // number of rings
    out.write_u32::<LittleEndian>(polygon.rings.len() as u32)?;
    for ring in polygon.rings.iter() {
        //number of points in ring
        out.write_u32::<LittleEndian>(ring.len() as u32)?;
        for point in ring.iter() {
            write_point_coordinates(point, out)?;
        }
    }
    Ok(diesel::serialize::IsNull::No)
}

#[cfg(feature = "diesel")]
impl<T> diesel::deserialize::FromSql<Geometry, diesel::pg::Pg> for Polygon<T>
where
    T: PointT + Debug + Clone,
{
    fn from_sql(bytes: diesel::pg::PgValue) -> diesel::deserialize::Result<Self> {
        let mut r = Cursor::new(bytes.as_bytes());
        let end = r.read_u8()?;
        if end == BIG_ENDIAN {
            read_polygon::<BigEndian, T>(&mut r)
        } else {
            read_polygon::<LittleEndian, T>(&mut r)
        }
    }
}

#[cfg(feature = "diesel")]
impl<T> diesel::deserialize::FromSql<Geography, diesel::pg::Pg> for Polygon<T>
where
    T: PointT + Debug + Clone,
{
    fn from_sql(bytes: diesel::pg::PgValue) -> diesel::deserialize::Result<Self> {
        diesel::deserialize::FromSql::<Geometry, diesel::pg::Pg>::from_sql(bytes)
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
