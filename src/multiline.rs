use std::fmt::Debug;
use std::io::Cursor;

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

#[cfg(feature = "diesel")]
use crate::{
    ewkb::{read_ewkb_header, write_ewkb_header},
    points::read_point_coordinates,
    write_to_read_from_sql::{ReadFromSql, WriteToSql},
};
use crate::{
    ewkb::{EwkbSerializable, GeometryType, BIG_ENDIAN},
    points::Dimension,
    types::{LineString, MultiLineString, PointT},
};

impl<P> MultiLineString<P>
where
    P: PointT + Clone,
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

    pub fn add_point(&mut self, point: P) -> &mut Self {
        if self.lines.last().is_none() {
            self.add_line();
        }
        self.lines.last_mut().unwrap().add_point(point);
        self
    }

    pub fn add_points(&mut self, points: impl IntoIterator<Item = P>) -> &mut Self {
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

impl<P> EwkbSerializable for MultiLineString<P>
where
    P: PointT,
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
impl<P> WriteToSql for MultiLineString<P>
where
    P: PointT + Debug+ EwkbSerializable,
{
    fn write_to_sql<W>(&self, out: &mut W) -> diesel::serialize::Result
    where
        W: std::io::Write,
    {
        write_ewkb_header(self, self.srid, out)?;
        // number of lines
        out.write_u32::<LittleEndian>(self.lines.len() as u32)?;
        for line in self.lines.iter() {
            println!("write line: {:?}", line);
            line.write_to_sql(out)?;
        }
        Ok(diesel::serialize::IsNull::No)
    }
}

#[cfg(feature = "diesel")]
impl<P> ReadFromSql for MultiLineString<P>
where
    P: PointT + Debug + Clone,
{
    fn read_from_sql(bytes: &[u8]) -> diesel::deserialize::Result<Self> {
        let mut r = Cursor::new(bytes);
        let end = r.read_u8()?;
        if end == BIG_ENDIAN {
            read_multiline::<BigEndian, P>(&mut r)
        } else {
            read_multiline::<LittleEndian, P>(&mut r)
        }
    }
}

#[cfg(feature = "diesel")]
fn read_multiline<T, P>(
    cursor: &mut Cursor<&[u8]>,
) -> diesel::deserialize::Result<MultiLineString<P>>
where
    T: byteorder::ByteOrder,
    P: PointT + Clone + Debug,
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
    P: PointT + Clone + Debug,
{
    let lines_n = cursor.read_u32::<T>()?;
    let mut multiline = MultiLineString::with_capacity(srid, lines_n as usize);
    for _i in 0..lines_n {
        println!("read line");
        // skip 1 byte for byte order and 4 bytes for point type
        cursor.read_u8()?;
        cursor.read_u32::<T>()?;
        let points_n = cursor.read_u32::<T>()?;
        multiline.add_line_with_cap(points_n as usize);
        for _p in 0..points_n {
            let point = read_point_coordinates::<T, P>(cursor, g_type, srid)?;
            println!("read point {:?}", point);
            multiline.add_point(point);
        }
    }
    Ok(multiline)
}
