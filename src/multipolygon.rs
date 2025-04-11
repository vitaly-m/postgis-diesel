use std::fmt::Debug;
use std::io::Cursor;

#[cfg(feature = "diesel")]
use crate::{
    ewkb::{read_ewkb_header, write_ewkb_header},
    polygon::read_polygon_body,
    write_to_read_from_sql::{ReadFromSql, WriteToSql},
};
use crate::{
    ewkb::{EwkbSerializable, GeometryType, BIG_ENDIAN},
    points::Dimension,
    types::*,
};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

impl<P> MultiPolygon<P>
where
    P: PointT + Clone,
{
    pub fn new(srid: Option<u32>) -> Self {
        Self::with_capacity(srid, 0)
    }

    pub fn with_capacity(srid: Option<u32>, cap: usize) -> Self {
        MultiPolygon {
            polygons: Vec::with_capacity(cap),
            srid,
        }
    }

    pub fn add_empty_polygon(&mut self) -> &mut Self {
        self.add_empty_polygon_with_capacity(0)
    }

    pub fn add_empty_polygon_with_capacity(&mut self, cap: usize) -> &mut Self {
        self.polygons.push(Polygon {
            rings: Vec::with_capacity(cap),
            srid: self.srid,
        });
        self
    }

    pub fn add_point(&mut self, point: P) -> &mut Self {
        if self.polygons.is_empty() {
            self.add_empty_polygon();
        }
        self.polygons.last_mut().unwrap().add_point(point);
        self
    }

    pub fn add_points(&mut self, points: impl IntoIterator<Item = P>) -> &mut Self {
        if self.polygons.is_empty() {
            self.add_empty_polygon();
        }
        let last = self.polygons.last_mut().unwrap();
        for point in points {
            last.add_point(point);
        }
        self
    }

    pub fn dimension(&self) -> u32 {
        let mut dimension = Dimension::None as u32;
        if let Some(polygon) = self.polygons.first() {
            dimension |= polygon.dimension();
        }
        dimension
    }
}

impl<P> EwkbSerializable for MultiPolygon<P>
where
    P: PointT + Clone,
{
    fn geometry_type(&self) -> u32 {
        let mut g_type = GeometryType::MultiPolygon as u32;
        if let Some(polygon) = self.polygons.first() {
            g_type |= polygon.dimension();
        }
        g_type
    }
}

#[cfg(feature = "diesel")]
impl<P> ReadFromSql for MultiPolygon<P>
where
    P: PointT + Debug + Clone,
{
    fn read_from_sql(bytes: &[u8]) -> diesel::deserialize::Result<Self> {
        let mut r = Cursor::new(bytes);
        let end = r.read_u8()?;
        if end == BIG_ENDIAN {
            read_multi_polygon::<BigEndian, P>(&mut r)
        } else {
            read_multi_polygon::<LittleEndian, P>(&mut r)
        }
    }
}

#[cfg(feature = "diesel")]
impl<P> WriteToSql for MultiPolygon<P>
where
    P: PointT + Clone + EwkbSerializable,
{
    fn write_to_sql<W>(&self, out: &mut W) -> diesel::serialize::Result
    where
        W: std::io::Write,
    {
        write_ewkb_header(self, self.srid, out)?;
        // number of polygons
        out.write_u32::<LittleEndian>(self.polygons.len() as u32)?;
        for polygon in self.polygons.iter() {
            polygon.write_to_sql(out)?;
        }
        Ok(diesel::serialize::IsNull::No)
    }
}

#[cfg(feature = "diesel")]
fn read_multi_polygon<T, P>(
    cursor: &mut Cursor<&[u8]>,
) -> diesel::deserialize::Result<MultiPolygon<P>>
where
    T: byteorder::ByteOrder,
    P: PointT + Clone,
{
    let g_header = read_ewkb_header::<T>(cursor)?.expect(GeometryType::MultiPolygon)?;
    read_multi_polygon_body::<T, P>(g_header.g_type, g_header.srid, cursor)
}

#[cfg(feature = "diesel")]
pub fn read_multi_polygon_body<T, P>(
    g_type: u32,
    srid: Option<u32>,
    cursor: &mut Cursor<&[u8]>,
) -> diesel::deserialize::Result<MultiPolygon<P>>
where
    T: byteorder::ByteOrder,
    P: PointT + Clone,
{
    let polygons_n = cursor.read_u32::<T>()?;
    let mut polygon = MultiPolygon::with_capacity(srid, polygons_n as usize);

    for _i in 0..polygons_n {
        // skip 1 byte for byte order and 4 bytes for point type
        cursor.read_u8()?;
        cursor.read_u32::<T>()?;
        polygon
            .polygons
            .push(read_polygon_body::<T, P>(g_type, srid, cursor)?);
    }
    Ok(polygon)
}
