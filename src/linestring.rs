use std::fmt::Debug;
use std::io::Cursor;

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

#[cfg(feature = "diesel")]
use crate::ewkb::{read_ewkb_header, write_ewkb_header};
use crate::points::Dimension;
#[cfg(feature = "diesel")]
use crate::points::{read_point_coordinates, write_point_coordinates};
use crate::write_to_read_from_sql::{ReadFromSql, WriteToSql};
use crate::{
    ewkb::{EwkbSerializable, GeometryType, BIG_ENDIAN},
    types::{LineString, PointT},
};

impl<P> EwkbSerializable for LineString<P>
where
    P: PointT,
{
    fn geometry_type(&self) -> u32 {
        GeometryType::LineString as u32 | self.dimension()
    }
}

impl<P> LineString<P>
where
    P: PointT,
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

    pub fn add_point(&mut self, point: P) -> &mut Self {
        self.points.push(point);
        self
    }

    pub fn add_points(&mut self, points: impl IntoIterator<Item = P>) -> &mut Self {
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
impl<P> ReadFromSql for LineString<P>
where
    P: PointT + Debug + Clone,
{
    fn read_from_sql(bytes: &[u8]) -> diesel::deserialize::Result<Self> {
        let mut r = Cursor::new(bytes);
        let end = r.read_u8()?;
        if end == BIG_ENDIAN {
            read_linestring::<BigEndian, P>(&mut r)
        } else {
            read_linestring::<LittleEndian, P>(&mut r)
        }
    }
}

#[cfg(feature = "diesel")]
impl<P> WriteToSql for LineString<P>
where
    P: PointT + EwkbSerializable,
{
    fn write_to_sql<W>(&self, out: &mut W) -> diesel::serialize::Result
    where
        W: std::io::Write,
    {
        write_ewkb_header(self, self.srid, out)?;
        // size and points
        out.write_u32::<LittleEndian>(self.points.len() as u32)?;
        for point in self.points.iter() {
            write_point_coordinates(point, out)?;
        }
        Ok(diesel::serialize::IsNull::No)
    }
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
