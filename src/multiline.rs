use std::fmt::Debug;
use std::io::Cursor;

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

#[cfg(feature = "diesel")]
use crate::{
    ewkb::{read_ewkb_header, write_ewkb_header},
    linestring::write_linestring,
    points::read_point_coordinates,
};
use crate::{
    ewkb::{EwkbSerializable, GeometryType, BIG_ENDIAN},
    points::Dimension,
    types::{LineString, MultiLineString, PointT},
};

use crate::sql_types::*;

impl<T> MultiLineString<T>
where
    T: PointT + Clone,
{
    pub fn new(srid: Option<u32>) -> Self {
        Self::with_capacity(srid, 0)
    }

    pub fn with_capacity(srid: Option<u32>, cap: usize) -> Self {
        MultiLineString {
            lines: Vec::with_capacity(cap),
            srid,
        }
    }

    pub fn add_line(&mut self) -> &mut Self {
        self.add_line_with_cap(0)
    }

    pub fn add_line_with_cap(&mut self, cap: usize) -> &mut Self {
        self.lines.push(LineString::with_capacity(self.srid, cap));
        self
    }

    pub fn add_point(&mut self, point: T) -> &mut Self {
        if self.lines.last().is_none() {
            self.add_line();
        }
        self.lines.last_mut().unwrap().add_point(point);
        self
    }

    pub fn add_points(&mut self, points: impl IntoIterator<Item = T>) -> &mut Self {
        if self.lines.last().is_none() {
            self.add_line();
        }
        let last = self.lines.last_mut().unwrap();
        for point in points {
            last.points.push(point);
        }
        self
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

#[cfg(feature = "diesel")]
impl<T> diesel::serialize::ToSql<Geometry, diesel::pg::Pg> for MultiLineString<T>
where
    T: PointT + Debug + PartialEq + EwkbSerializable + Clone,
{
    fn to_sql(
        &self,
        out: &mut diesel::serialize::Output<diesel::pg::Pg>,
    ) -> diesel::serialize::Result {
        write_multiline(self, self.srid, out)
    }
}

#[cfg(feature = "diesel")]
impl<T> diesel::serialize::ToSql<Geography, diesel::pg::Pg> for MultiLineString<T>
where
    T: PointT + Debug + PartialEq + EwkbSerializable + Clone,
{
    fn to_sql(
        &self,
        out: &mut diesel::serialize::Output<diesel::pg::Pg>,
    ) -> diesel::serialize::Result {
        write_multiline(self, self.srid, out)
    }
}

#[cfg(feature = "diesel")]
impl<T> diesel::deserialize::FromSql<Geometry, diesel::pg::Pg> for MultiLineString<T>
where
    T: PointT + Debug + Clone,
{
    fn from_sql(bytes: diesel::pg::PgValue) -> diesel::deserialize::Result<Self> {
        let mut r = Cursor::new(bytes.as_bytes());
        let end = r.read_u8()?;
        if end == BIG_ENDIAN {
            read_multiline::<BigEndian, T>(&mut r)
        } else {
            read_multiline::<LittleEndian, T>(&mut r)
        }
    }
}

#[cfg(feature = "diesel")]
impl<T> diesel::deserialize::FromSql<Geography, diesel::pg::Pg> for MultiLineString<T>
where
    T: PointT + Debug + Clone,
{
    fn from_sql(bytes: diesel::pg::PgValue) -> diesel::deserialize::Result<Self> {
        diesel::deserialize::FromSql::<Geometry, diesel::pg::Pg>::from_sql(bytes)
    }
}

#[cfg(feature = "diesel")]
pub fn write_multiline<T>(
    multiline: &MultiLineString<T>,
    srid: Option<u32>,
    out: &mut diesel::serialize::Output<diesel::pg::Pg>,
) -> diesel::serialize::Result
where
    T: PointT + EwkbSerializable + Clone,
{
    write_ewkb_header(multiline, srid, out)?;
    // number of lines
    out.write_u32::<LittleEndian>(multiline.lines.len() as u32)?;
    for line in multiline.lines.iter() {
        write_linestring(line, None, out)?;
    }
    Ok(diesel::serialize::IsNull::No)
}

#[cfg(feature = "diesel")]
fn read_multiline<T, P>(
    cursor: &mut Cursor<&[u8]>,
) -> diesel::deserialize::Result<MultiLineString<P>>
where
    T: byteorder::ByteOrder,
    P: PointT + Clone,
{
    let g_header = read_ewkb_header::<T>(cursor)?.expect(GeometryType::MultiLineString)?;
    read_multiline_body::<T, P>(g_header.g_type, g_header.srid, cursor)
}

#[cfg(feature = "diesel")]
pub fn read_multiline_body<T, P>(
    g_type: u32,
    srid: Option<u32>,
    cursor: &mut Cursor<&[u8]>,
) -> diesel::deserialize::Result<MultiLineString<P>>
where
    T: byteorder::ByteOrder,
    P: PointT + Clone,
{
    let lines_n = cursor.read_u32::<T>()?;
    let mut multiline = MultiLineString::with_capacity(srid, lines_n as usize);
    for _i in 0..lines_n {
        // skip 1 byte for byte order and 4 bytes for point type
        cursor.read_u8()?;
        cursor.read_u32::<T>()?;
        let points_n = cursor.read_u32::<T>()?;
        multiline.add_line_with_cap(points_n as usize);
        for _p in 0..points_n {
            multiline.add_point(read_point_coordinates::<T, P>(cursor, g_type, srid)?);
        }
    }
    Ok(multiline)
}
