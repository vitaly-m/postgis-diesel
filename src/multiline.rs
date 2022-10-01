use std::fmt::{Debug};
use std::io::Cursor;

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use diesel::{
    deserialize::{self, FromSql},
    pg::{self, Pg},
    serialize::{self, IsNull, Output, ToSql},
};

use crate::points::{PointT, write_point_coordinates, read_point_coordinates};
use crate::sql_types::*;

#[derive(Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
pub struct MultiLineString<T> {
    pub lines: Vec<Vec<T>>,
    pub srid: Option<u32>,
}

impl<T> MultiLineString<T>
where T: PointT + Clone {
    pub fn new(srid: Option<u32>) -> Self {
        MultiLineString { lines: Vec::new(), srid: srid }
    }

    pub fn add_line<'a>(&'a mut self) {
        self.lines.push(Vec::new());
    }

    pub fn add_point<'a>(&'a mut self, point: T) {
        if self.lines.last().is_none() {
            self.add_line();
        }
        self.lines.last_mut().unwrap().push(point);
    }

    pub fn add_points<'a>(&'a mut self, points: &[T]) {
        if self.lines.last().is_none() {
            self.add_line();
        }
        let last = self.lines.last_mut().unwrap();
        for point in points {
            last.push(point.to_owned());
        }
    }
}

impl<T> ToSql<Geometry, Pg> for MultiLineString<T>
where
    T: PointT + Debug + PartialEq,
{
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
        if self.lines.len() < 1 {
            return Err(format!(
                "MultiLineString must contain at least one line but has {}",
                self.lines.len()
            )
            .into());
        }
        out.write_u8(LITTLE_ENDIAN)?;
        // polygon can have points of the same type
        let mut g_type = GeometryType::MultiLineString as u32;
        let first_point = self.lines.first().unwrap().first().unwrap();
        if self.srid.is_some() {
            g_type |= SRID;
        }
        g_type |= first_point.dimension();
        match self.srid {
            Some(srid) => {
                out.write_u32::<LittleEndian>(g_type)?;
                out.write_u32::<LittleEndian>(srid)?;
            }
            None => out.write_u32::<LittleEndian>(g_type)?,
        }
        // number of lines
        out.write_u32::<LittleEndian>(self.lines.len() as u32)?;
        for (line_n, line) in self.lines.iter().enumerate() {
            if line.len() < 2 {
                return Err(format!(
                    "Line {} must contain at least two points but has {}",
                    line_n,
                    self.lines.len()
                )
                .into());
            }
            let mut l_type = GeometryType::LineString as u32;
            l_type |= line.first().unwrap().dimension();
            out.write_u8(LITTLE_ENDIAN)?;
            out.write_u32::<LittleEndian>(l_type)?;
            //number of points in line
            out.write_u32::<LittleEndian>(line.len() as u32)?;
            for point in line.iter() {
                write_point_coordinates(point, out)?;
            }
        }
        Ok(IsNull::No)
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
            read_polygon::<BigEndian, T>(&mut r)
        } else {
            read_polygon::<LittleEndian, T>(&mut r)
        }
    }
}


fn read_polygon<T, P>(cursor: &mut Cursor<&[u8]>) -> deserialize::Result<MultiLineString<P>>
where
    T: byteorder::ByteOrder,
    P: PointT + Clone,
{
    let g_type = cursor.read_u32::<T>()?;
    if GeometryType::from(g_type) != GeometryType::MultiLineString {
        return Err(format!(
            "Geometry {:?} is not a MultiLineString",
            GeometryType::from(g_type)
        )
        .into());
    }
    let mut srid = None;
    // SRID included
    if g_type & SRID == SRID {
        srid = Some(cursor.read_u32::<T>()?);
    }
    let lines_n = cursor.read_u32::<T>()?;
    let mut polygon = MultiLineString::new(srid);

    for _i in 0..lines_n {
        polygon.add_line();
        // skip 1 byte for byte order and 4 bytes for point type
        cursor.read_u8()?;
        cursor.read_u32::<T>()?;
        let points_n = cursor.read_u32::<T>()?;
        for _p in 0..points_n {
            polygon.add_point(read_point_coordinates::<T, P>(cursor, g_type, srid)?);
        }
    }
    Ok(polygon)
}

