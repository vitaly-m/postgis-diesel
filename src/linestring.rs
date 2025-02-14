use std::fmt::Debug;
use std::io::Cursor;

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

#[cfg(feature = "diesel")]
use crate::ewkb::{read_ewkb_header, write_ewkb_header};
use crate::points::Dimension;
#[cfg(feature = "diesel")]
use crate::points::{read_point_coordinates, write_point_coordinates};
use crate::sql_types::*;
use crate::{
    ewkb::{EwkbSerializable, GeometryType, BIG_ENDIAN},
    types::{LineString, PointT},
};

impl<T> EwkbSerializable for LineString<T>
where
    T: PointT,
{
    fn geometry_type(&self) -> u32 {
        GeometryType::LineString as u32 | self.dimension()
    }
}

impl<T> LineString<T>
where
    T: PointT,
{
    pub fn new(srid: Option<u32>) -> Self {
        Self::with_capacity(srid, 0)
    }

    pub fn with_capacity(srid: Option<u32>, cap: usize) -> Self {
        LineString {
            points: Vec::with_capacity(cap),
            srid,
        }
    }

    pub fn add_point<'a>(&'a mut self, point: T) -> &mut Self {
        self.points.push(point);
        self
    }

    pub fn add_points<'a>(&'a mut self, points: impl IntoIterator<Item = T>) -> &mut Self {
        for point in points {
            self.points.push(point);
        }
        self
    }

    pub fn dimension(&self) -> u32 {
        let mut dimension = Dimension::None as u32;
        if let Some(point) = self.points.first() {
            dimension |= point.dimension();
        }
        dimension
    }
}

#[cfg(feature = "diesel")]
impl<T> diesel::deserialize::FromSql<Geometry, diesel::pg::Pg> for LineString<T>
where
    T: PointT + Debug + Clone,
{
    fn from_sql(bytes: diesel::pg::PgValue) -> diesel::deserialize::Result<Self> {
        let mut r = Cursor::new(bytes.as_bytes());
        let end = r.read_u8()?;
        if end == BIG_ENDIAN {
            read_linestring::<BigEndian, T>(&mut r)
        } else {
            read_linestring::<LittleEndian, T>(&mut r)
        }
    }
}

#[cfg(feature = "diesel")]
impl<T> diesel::deserialize::FromSql<Geography, diesel::pg::Pg> for LineString<T>
where
    T: PointT + Debug + Clone,
{
    fn from_sql(bytes: diesel::pg::PgValue) -> diesel::deserialize::Result<Self> {
        diesel::deserialize::FromSql::<Geometry, diesel::pg::Pg>::from_sql(bytes)
    }
}

#[cfg(feature = "diesel")]
impl<T> diesel::serialize::ToSql<Geometry, diesel::pg::Pg> for LineString<T>
where
    T: PointT + Debug + EwkbSerializable,
{
    fn to_sql(
        &self,
        out: &mut diesel::serialize::Output<diesel::pg::Pg>,
    ) -> diesel::serialize::Result {
        write_linestring(self, self.srid, out)
    }
}

#[cfg(feature = "diesel")]
impl<T> diesel::serialize::ToSql<Geography, diesel::pg::Pg> for LineString<T>
where
    T: PointT + Debug + EwkbSerializable,
{
    fn to_sql(
        &self,
        out: &mut diesel::serialize::Output<diesel::pg::Pg>,
    ) -> diesel::serialize::Result {
        write_linestring(self, self.srid, out)
    }
}

#[cfg(feature = "diesel")]
pub fn write_linestring<T>(
    linestring: &LineString<T>,
    srid: Option<u32>,
    out: &mut diesel::serialize::Output<diesel::pg::Pg>,
) -> diesel::serialize::Result
where
    T: PointT + EwkbSerializable,
{
    write_ewkb_header(linestring, srid, out)?;
    // size and points
    out.write_u32::<LittleEndian>(linestring.points.len() as u32)?;
    for point in linestring.points.iter() {
        write_point_coordinates(point, out)?;
    }
    Ok(diesel::serialize::IsNull::No)
}

#[cfg(feature = "diesel")]
fn read_linestring<T, P>(cursor: &mut Cursor<&[u8]>) -> diesel::deserialize::Result<LineString<P>>
where
    T: byteorder::ByteOrder,
    P: PointT + Clone,
{
    let g_header = read_ewkb_header::<T>(cursor)?.expect(GeometryType::LineString)?;
    read_linestring_body::<T, P>(g_header.g_type, g_header.srid, cursor)
}

#[cfg(feature = "diesel")]
pub fn read_linestring_body<T, P>(
    g_type: u32,
    srid: Option<u32>,
    cursor: &mut Cursor<&[u8]>,
) -> diesel::deserialize::Result<LineString<P>>
where
    T: byteorder::ByteOrder,
    P: PointT + Clone,
{
    let len = cursor.read_u32::<T>()?;
    let mut ls = LineString::with_capacity(srid, len as usize);
    for _i in 0..len {
        ls.add_point(read_point_coordinates::<T, P>(cursor, g_type, srid)?);
    }
    Ok(ls)
}
