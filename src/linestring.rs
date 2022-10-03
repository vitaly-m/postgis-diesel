use std::fmt::Debug;
use std::io::Cursor;

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use diesel::{
    deserialize::{self, FromSql},
    pg::{self, Pg},
    serialize::{self, IsNull, Output, ToSql},
};

use crate::points::{read_point_coordinates, write_point_coordinates, Dimension};
use crate::sql_types::*;
use crate::{
    ewkb::{read_ewkb_header, write_ewkb_header, EwkbSerializable, GeometryType, BIG_ENDIAN},
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
    pub fn dimension(&self) -> u32 {
        let mut dimension = Dimension::None as u32;
        if let Some(point) = self.points.first() {
            dimension |= point.dimension();
        }
        dimension
    }
}

impl<T> FromSql<Geometry, Pg> for LineString<T>
where
    T: PointT + Debug + Clone,
{
    fn from_sql(bytes: pg::PgValue) -> deserialize::Result<Self> {
        let mut r = Cursor::new(bytes.as_bytes());
        let end = r.read_u8()?;
        if end == BIG_ENDIAN {
            read_linestring::<BigEndian, T>(&mut r)
        } else {
            read_linestring::<LittleEndian, T>(&mut r)
        }
    }
}

impl<T> ToSql<Geometry, Pg> for LineString<T>
where
    T: PointT + Debug + EwkbSerializable,
{
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
        write_linestring(self, self.srid, out)
    }
}

pub fn write_linestring<T>(
    linestring: &LineString<T>,
    srid: Option<u32>,
    out: &mut Output<Pg>,
) -> serialize::Result
where
    T: PointT + EwkbSerializable,
{
    write_ewkb_header(linestring, srid, out)?;
    // size and points
    out.write_u32::<LittleEndian>(linestring.points.len() as u32)?;
    for point in linestring.points.iter() {
        write_point_coordinates(point, out)?;
    }
    Ok(IsNull::No)
}

fn read_linestring<T, P>(cursor: &mut Cursor<&[u8]>) -> deserialize::Result<LineString<P>>
where
    T: byteorder::ByteOrder,
    P: PointT + Clone,
{
    let g_header = read_ewkb_header::<T>(GeometryType::LineString, cursor)?;
    read_linestring_body::<T, P>(g_header.g_type, g_header.srid, cursor)
}

pub fn read_linestring_body<T, P>(
    g_type: u32,
    srid: Option<u32>,
    cursor: &mut Cursor<&[u8]>,
) -> deserialize::Result<LineString<P>>
where
    T: byteorder::ByteOrder,
    P: PointT + Clone,
{
    let len = cursor.read_u32::<T>()?;
    let mut points = Vec::with_capacity(len as usize);
    for _i in 0..len {
        points.push(read_point_coordinates::<T, P>(cursor, g_type, srid)?);
    }
    Ok(LineString {
        points: points,
        srid: srid,
    })
}
