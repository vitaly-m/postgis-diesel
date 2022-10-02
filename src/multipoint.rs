use std::fmt::Debug;
use std::io::Cursor;

use crate::{ewkb::{
    write_ewkb_header, EwkbSerializable, GeometryType, BIG_ENDIAN, read_ewkb_header,
}, points::write_point};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use diesel::{
    deserialize::{self, FromSql},
    pg::{self, Pg},
    serialize::{self, IsNull, Output, ToSql},
};

use crate::points::{read_point_coordinates, PointT};
use crate::sql_types::*;

#[derive(Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
pub struct MultiPoint<T> {
    pub points: Vec<T>,
    pub srid: Option<u32>,
}

impl<T> EwkbSerializable for MultiPoint<T>
where
    T: PointT,
{
    fn geometry_type(&self) -> u32 {
        let mut g_type = GeometryType::MultiPoint as u32;
        if let Some(point) = self.points.first() {
            g_type |= point.dimension();
        }
        g_type
    }
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
    T: PointT + Debug + EwkbSerializable,
{
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
        write_ewkb_header(self, self.srid, out)?;
        // size and points
        out.write_u32::<LittleEndian>(self.points.len() as u32)?;
        for point in self.points.iter() {
            write_point(point, None, out)?;
        }
        Ok(IsNull::No)
    }
}

fn read_multipoint<T, P>(cursor: &mut Cursor<&[u8]>) -> deserialize::Result<MultiPoint<P>>
where
    T: byteorder::ByteOrder,
    P: PointT,
{
    let g_header = read_ewkb_header::<T>(GeometryType::MultiPoint, cursor)?;    
    let len = cursor.read_u32::<T>()?;

    let mut points = Vec::with_capacity(len as usize);
    for _i in 0..len {
        // skip 1 byte for byte order and 4 bytes for point type
        cursor.read_u8()?;
        cursor.read_u32::<T>()?;
        points.push(read_point_coordinates::<T, P>(cursor, g_header.g_type, g_header.srid)?);
    }
    Ok(MultiPoint {
        points: points,
        srid: g_header.srid,
    })
}
