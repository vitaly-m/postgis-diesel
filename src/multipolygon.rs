use std::fmt::Debug;
use std::io::Cursor;

#[cfg(feature = "diesel")]
use crate::{
    ewkb::{read_ewkb_header, write_ewkb_header},
    polygon::{read_polygon_body, write_polygon},
};
use crate::{
    ewkb::{EwkbSerializable, GeometryType, BIG_ENDIAN},
    points::Dimension,
    types::*,
};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::sql_types::*;

impl<T> MultiPolygon<T>
where
    T: PointT + Clone,
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

    pub fn add_empty_polygon<'a>(&'a mut self) -> &mut Self {
        self.add_empty_polygon_with_capacity(0)
    }

    pub fn add_empty_polygon_with_capacity<'a>(&'a mut self, cap: usize) -> &mut Self {
        self.polygons.push(Polygon {
            rings: Vec::with_capacity(cap),
            srid: self.srid,
        });
        self
    }

    pub fn add_point<'a>(&'a mut self, point: T) -> &mut Self {
        if self.polygons.is_empty() {
            self.add_empty_polygon();
        }
        self.polygons.last_mut().unwrap().add_point(point);
        self
    }

    pub fn add_points<'a>(&'a mut self, points: impl IntoIterator<Item = T>) -> &mut Self {
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

impl<T> EwkbSerializable for MultiPolygon<T>
where
    T: PointT + Clone,
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
impl<T> diesel::serialize::ToSql<Geometry, diesel::pg::Pg> for MultiPolygon<T>
where
    T: PointT + Debug + PartialEq + Clone + EwkbSerializable,
{
    fn to_sql(
        &self,
        out: &mut diesel::serialize::Output<diesel::pg::Pg>,
    ) -> diesel::serialize::Result {
        write_multi_polygon(self, self.srid, out)
    }
}

#[cfg(feature = "diesel")]
impl<T> diesel::serialize::ToSql<Geography, diesel::pg::Pg> for MultiPolygon<T>
where
    T: PointT + Debug + PartialEq + Clone + EwkbSerializable,
{
    fn to_sql(
        &self,
        out: &mut diesel::serialize::Output<diesel::pg::Pg>,
    ) -> diesel::serialize::Result {
        write_multi_polygon(self, self.srid, out)
    }
}

#[cfg(feature = "diesel")]
impl<T> diesel::deserialize::FromSql<Geometry, diesel::pg::Pg> for MultiPolygon<T>
where
    T: PointT + Debug + Clone,
{
    fn from_sql(bytes: diesel::pg::PgValue) -> diesel::deserialize::Result<Self> {
        let mut r = Cursor::new(bytes.as_bytes());
        let end = r.read_u8()?;
        if end == BIG_ENDIAN {
            read_multi_polygon::<BigEndian, T>(&mut r)
        } else {
            read_multi_polygon::<LittleEndian, T>(&mut r)
        }
    }
}

#[cfg(feature = "diesel")]
impl<T> diesel::deserialize::FromSql<Geography, diesel::pg::Pg> for MultiPolygon<T>
where
    T: PointT + Debug + Clone,
{
    fn from_sql(bytes: diesel::pg::PgValue) -> diesel::deserialize::Result<Self> {
        diesel::deserialize::FromSql::<Geometry, diesel::pg::Pg>::from_sql(bytes)
    }
}

#[cfg(feature = "diesel")]
pub fn write_multi_polygon<T>(
    multipolygon: &MultiPolygon<T>,
    srid: Option<u32>,
    out: &mut diesel::serialize::Output<diesel::pg::Pg>,
) -> diesel::serialize::Result
where
    T: PointT + EwkbSerializable + Clone,
{
    write_ewkb_header(multipolygon, srid, out)?;
    // number of polygons
    out.write_u32::<LittleEndian>(multipolygon.polygons.len() as u32)?;
    for polygon in multipolygon.polygons.iter() {
        write_polygon(polygon, None, out)?;
    }
    Ok(diesel::serialize::IsNull::No)
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
