use std::io::Cursor;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use diesel::{
    deserialize,
    pg::Pg,
    serialize::{self, IsNull, Output},
};

#[derive(Debug, PartialEq)]
pub enum GeometryType {
    Point = 1,
    LineString,
    Polygon,
    MultiPoint,
    MultiLineString,
    MultiPolygon,
    GeometryCollection,
}

pub const SRID: u32 = 0x20000000;
pub const LITTLE_ENDIAN: u8 = 1;
pub const BIG_ENDIAN: u8 = 0;

impl From<u32> for GeometryType {
    fn from(t: u32) -> Self {
        if t & 7 == 7 {
            return Self::GeometryCollection;
        } else if t & 6 == 6 {
            return Self::MultiPolygon;
        } else if t & 5 == 5 {
            return Self::MultiLineString;
        } else if t & 4 == 4 {
            return Self::MultiPoint;
        } else if t & 3 == 3 {
            return Self::Polygon;
        } else if t & 2 == 2 {
            return Self::LineString;
        } else {
            return Self::Point;
        }
    }
}

pub trait EwkbSerializable {
    fn geometry_type(&self) -> u32;
}

pub fn write_ewkb_header<T>(
    geometry: &T,
    srid: Option<u32>,
    out: &mut Output<Pg>,
) -> serialize::Result
where
    T: EwkbSerializable,
{
    out.write_u8(LITTLE_ENDIAN)?;
    let mut p_type = geometry.geometry_type();
    match srid {
        Some(srid) => {
            p_type |= SRID;
            out.write_u32::<LittleEndian>(p_type)?;
            out.write_u32::<LittleEndian>(srid)?;
        }
        None => out.write_u32::<LittleEndian>(p_type)?,
    }
    Ok(IsNull::No)
}

pub struct EwkbHeader {
    pub g_type: u32,
    pub srid: Option<u32>,
}

pub fn read_ewkb_header<T>(
    expected_type: GeometryType,
    cursor: &mut Cursor<&[u8]>,
) -> deserialize::Result<EwkbHeader>
where
    T: byteorder::ByteOrder,
{
    let g_type = cursor.read_u32::<T>()?;
    if GeometryType::from(g_type) != expected_type {
        return Err(format!(
            "Geometry {:?} is not a {:?}",
            GeometryType::from(g_type),
            expected_type
        )
        .into());
    }
    let mut srid = None;
    // SRID included
    if g_type & SRID == SRID {
        srid = Some(cursor.read_u32::<T>()?);
    }
    Ok(EwkbHeader { g_type, srid })
}
