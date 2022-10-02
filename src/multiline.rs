use std::fmt::Debug;
use std::io::Cursor;

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use diesel::{
    deserialize::{self, FromSql},
    pg::{self, Pg},
    serialize::{self, IsNull, Output, ToSql},
};

use crate::{
    ewkb::{read_ewkb_header, write_ewkb_header, EwkbSerializable, GeometryType, BIG_ENDIAN},
    linestring::write_linestring,
    points::Dimension,
};

use crate::linestring::LineString;
use crate::points::{read_point_coordinates, PointT};
use crate::sql_types::*;

#[derive(Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
pub struct MultiLineString<T> {
    pub lines: Vec<LineString<T>>,
    pub srid: Option<u32>,
}

impl<T> MultiLineString<T>
where
    T: PointT + Clone,
{
    pub fn new(srid: Option<u32>) -> Self {
        MultiLineString {
            lines: Vec::new(),
            srid: srid,
        }
    }

    pub fn add_line<'a>(&'a mut self) {
        self.lines.push(LineString {
            points: Vec::new(),
            srid: self.srid,
        });
    }

    pub fn add_point<'a>(&'a mut self, point: T) {
        if self.lines.last().is_none() {
            self.add_line();
        }
        self.lines.last_mut().unwrap().points.push(point);
    }

    pub fn add_points<'a>(&'a mut self, points: &[T]) {
        if self.lines.last().is_none() {
            self.add_line();
        }
        let last = self.lines.last_mut().unwrap();
        for point in points {
            last.points.push(point.to_owned());
        }
    }

    pub fn dimension(&self) -> u32 {
        let mut dimension = Dimension::None as u32;
        if let Some(line) = self.lines.first() {
            dimension |= line.dimension();
        }
        dimension
    }
}

impl<T> EwkbSerializable for MultiLineString<T>
where
    T: PointT,
{
    fn geometry_type(&self) -> u32 {
        let mut g_type = GeometryType::MultiLineString as u32;
        if let Some(line) = self.lines.first() {
            g_type |= line.dimension();
        }
        g_type
    }
}

impl<T> ToSql<Geometry, Pg> for MultiLineString<T>
where
    T: PointT + Debug + PartialEq + EwkbSerializable + Clone,
{
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
        write_multiline(self, self.srid, out)
    }
}

impl<T> FromSql<Geometry, Pg> for MultiLineString<T>
where
    T: PointT + Debug + Clone,
{
    fn from_sql(bytes: pg::PgValue) -> deserialize::Result<Self> {
        let mut r = Cursor::new(bytes.as_bytes());
        let end = r.read_u8()?;
        if end == BIG_ENDIAN {
            read_multiline::<BigEndian, T>(&mut r)
        } else {
            read_multiline::<LittleEndian, T>(&mut r)
        }
    }
}

pub fn write_multiline<T>(
    multiline: &MultiLineString<T>,
    srid: Option<u32>,
    out: &mut Output<Pg>,
) -> serialize::Result
where
    T: PointT + EwkbSerializable + Clone,
{
    write_ewkb_header(multiline, srid, out)?;
    // number of lines
    out.write_u32::<LittleEndian>(multiline.lines.len() as u32)?;
    for line in multiline.lines.iter() {
        write_linestring(line, None, out)?;
    }
    Ok(IsNull::No)
}

fn read_multiline<T, P>(cursor: &mut Cursor<&[u8]>) -> deserialize::Result<MultiLineString<P>>
where
    T: byteorder::ByteOrder,
    P: PointT + Clone,
{
    let g_header = read_ewkb_header::<T>(GeometryType::MultiLineString, cursor)?;
    read_multiline_body::<T, P>(g_header.g_type, g_header.srid, cursor)
}

pub fn read_multiline_body<T, P>(g_type: u32, srid: Option<u32>, cursor: &mut Cursor<&[u8]>) -> deserialize::Result<MultiLineString<P>>
where
    T: byteorder::ByteOrder,
    P: PointT + Clone,
{
    let lines_n = cursor.read_u32::<T>()?;
    let mut multiline = MultiLineString::new(srid);
    for _i in 0..lines_n {
        multiline.add_line();
        // skip 1 byte for byte order and 4 bytes for point type
        cursor.read_u8()?;
        cursor.read_u32::<T>()?;
        let points_n = cursor.read_u32::<T>()?;
        for _p in 0..points_n {
            multiline.add_point(read_point_coordinates::<T, P>(
                cursor,
                g_type,
                srid,
            )?);
        }
    }
    Ok(multiline)
}