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
pub struct LineString<T> {
    pub points: Vec<T>,
    pub srid: Option<u32>,
}

impl<T> FromSql<Geometry, Pg> for LineString<T>
where
    T: PointT + Debug,
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
    T: PointT + Debug,
{
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
        if self.points.len() < 2 {
            return Err(format!(
                "LineString must contain at least two points but has {}",
                self.points.len()
            )
            .into());
        }
        out.write_u8(LITTLE_ENDIAN)?;
        // linestring can have points of the same type
        let mut g_type = GeometryType::LineString as u32;
        let first_point = self.points.first().unwrap();
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
        // size and points
        out.write_u32::<LittleEndian>(self.points.len() as u32)?;
        for point in self.points.iter() {
            write_point_coordinates(point, out)?;
        }
        Ok(IsNull::No)
    }
}

fn read_linestring<T, P>(cursor: &mut Cursor<&[u8]>) -> deserialize::Result<LineString<P>>
where
    T: byteorder::ByteOrder,
    P: PointT,
{
    let g_type = cursor.read_u32::<T>()?;
    if GeometryType::from(g_type) != GeometryType::LineString {
        return Err(format!(
            "Geometry {:?} is not a LineString",
            GeometryType::from(g_type)
        )
        .into());
    }
    let mut srid = None;
    // SRID included
    if g_type & SRID == SRID {
        srid = Some(cursor.read_u32::<T>()?);
    }
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
