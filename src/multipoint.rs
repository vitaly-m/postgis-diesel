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
pub struct MultiPoint<T> {
    pub points: Vec<T>,
    pub srid: Option<u32>,
}

impl<T> FromSql<Geometry, Pg> for MultiPoint<T>
where
    T: PointT + Debug,
{
    fn from_sql(bytes: pg::PgValue) -> deserialize::Result<Self> {
        let mut r = Cursor::new(bytes.as_bytes());
        let end = r.read_u8()?;
        if end == BIG_ENDIAN {
            read_multipoint::<BigEndian, T>(&mut r)
        } else {
            read_multipoint::<LittleEndian, T>(&mut r)
        }
    }
}


impl<T> ToSql<Geometry, Pg> for MultiPoint<T>
where
    T: PointT + Debug,
{
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
        if self.points.len() < 1 {
            return Err(format!(
                "MultiPoint must contain at least one point but has {}",
                self.points.len()
            )
            .into());
        }
        out.write_u8(LITTLE_ENDIAN)?;
        let mut g_type = GeometryType::MultiPoint as u32;
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
            let mut p_type = GeometryType::Point as u32;
            p_type |= point.dimension();
            out.write_u8(LITTLE_ENDIAN)?;
            out.write_u32::<LittleEndian>(p_type)?;
            write_point_coordinates(point, out)?;
        }
        Ok(IsNull::No)
    }
}

fn read_multipoint<T, P>(cursor: &mut Cursor<&[u8]>) -> deserialize::Result<MultiPoint<P>>
where
    T: byteorder::ByteOrder,
    P: PointT,
{
    let g_type = cursor.read_u32::<T>()?;
    if GeometryType::from(g_type) != GeometryType::MultiPoint {
        return Err(format!(
            "Geometry {:?} is not a MultiPoint",
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
        // skip 1 byte for byte order and 4 bytes for point type
        cursor.read_u8()?;
        cursor.read_u32::<T>()?;
        points.push(read_point_coordinates::<T, P>(cursor, g_type, srid)?);
    }
    Ok(MultiPoint {
        points: points,
        srid: srid,
    })
}