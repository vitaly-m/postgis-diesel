use std::io::Cursor;

use crate::{
    ewkb::{read_ewkb_header, EwkbSerializable, GeometryType, BIG_ENDIAN},
    types::*,
};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use diesel::{
    deserialize::{self, FromSql},
    pg::{self, Pg},
    serialize::{self, IsNull, Output, ToSql},
};

use crate::{ewkb::write_ewkb_header, sql_types::*};

pub enum Dimension {
    None = 0,
    Z = 0x80000000,
    M = 0x40000000,
    ZM = 0x40000000 | 0x80000000,
}

impl EwkbSerializable for Point {
    fn geometry_type(&self) -> u32 {
        GeometryType::Point as u32
    }
}

impl EwkbSerializable for PointZ {
    fn geometry_type(&self) -> u32 {
        GeometryType::Point as u32 | Dimension::Z as u32
    }
}

impl EwkbSerializable for PointM {
    fn geometry_type(&self) -> u32 {
        GeometryType::Point as u32 | Dimension::M as u32
    }
}

impl EwkbSerializable for PointZM {
    fn geometry_type(&self) -> u32 {
        GeometryType::Point as u32 | Dimension::ZM as u32
    }
}

impl Point {
    pub fn new(x: f64, y: f64, srid: Option<u32>) -> Self {
        Self { x, y, srid }
    }
}

impl PointZ {
    pub fn new(x: f64, y: f64, z: f64, srid: Option<u32>) -> Self {
        Self { x, y, z, srid }
    }
}

impl PointM {
    pub fn new(x: f64, y: f64, m: f64, srid: Option<u32>) -> Self {
        Self { x, y, m, srid }
    }
}

impl PointZM {
    pub fn new(x: f64, y: f64, z: f64, m: f64, srid: Option<u32>) -> Self {
        Self { x, y, z, m, srid }
    }
}

impl PointT for Point {
    fn get_x(&self) -> f64 {
        self.x
    }

    fn get_y(&self) -> f64 {
        self.y
    }

    fn get_srid(&self) -> Option<u32> {
        self.srid
    }

    fn get_z(&self) -> Option<f64> {
        None
    }

    fn get_m(&self) -> Option<f64> {
        None
    }

    fn dimension(&self) -> u32 {
        0
    }

    fn new_point(
        x: f64,
        y: f64,
        srid: Option<u32>,
        z: Option<f64>,
        m: Option<f64>,
    ) -> Result<Self, PointConstructorError> {
        if z.is_some() || m.is_some() {
            return Err(PointConstructorError {
                reason: format!("unexpectedly defined Z {:?} or M {:?} for Point", z, m)
                    .to_string(),
            });
        }
        Ok(Point { x, y, srid })
    }
}

impl PointT for PointZ {
    fn get_x(&self) -> f64 {
        self.x
    }

    fn get_y(&self) -> f64 {
        self.y
    }

    fn get_srid(&self) -> Option<u32> {
        self.srid
    }

    fn get_z(&self) -> Option<f64> {
        Some(self.z)
    }

    fn get_m(&self) -> Option<f64> {
        None
    }

    fn dimension(&self) -> u32 {
        Dimension::Z as u32
    }

    fn new_point(
        x: f64,
        y: f64,
        srid: Option<u32>,
        z: Option<f64>,
        m: Option<f64>,
    ) -> Result<Self, PointConstructorError> {
        if z.is_none() {
            return Err(PointConstructorError {
                reason: "Z is not defined, but mandatory for PointZ".to_string(),
            });
        }
        if m.is_some() {
            return Err(PointConstructorError {
                reason: format!("unexpectedly defined m {:?} for PointZ", m).to_string(),
            });
        }
        Ok(PointZ {
            x,
            y,
            z: z.unwrap(),
            srid,
        })
    }
}

impl PointT for PointM {
    fn get_x(&self) -> f64 {
        self.x
    }

    fn get_y(&self) -> f64 {
        self.y
    }

    fn get_srid(&self) -> Option<u32> {
        self.srid
    }

    fn get_z(&self) -> Option<f64> {
        None
    }

    fn get_m(&self) -> Option<f64> {
        Some(self.m)
    }

    fn dimension(&self) -> u32 {
        Dimension::M as u32
    }

    fn new_point(
        x: f64,
        y: f64,
        srid: Option<u32>,
        z: Option<f64>,
        m: Option<f64>,
    ) -> Result<Self, PointConstructorError> {
        if m.is_none() {
            return Err(PointConstructorError {
                reason: "M is not defined, but mandatory for PointM".to_string(),
            });
        }
        if z.is_some() {
            return Err(PointConstructorError {
                reason: format!("unexpectedly defined z {:?} for PointM", z).to_string(),
            });
        }
        Ok(PointM {
            x,
            y,
            m: m.unwrap(),
            srid,
        })
    }
}

impl PointT for PointZM {
    fn get_x(&self) -> f64 {
        self.x
    }

    fn get_y(&self) -> f64 {
        self.y
    }

    fn get_srid(&self) -> Option<u32> {
        self.srid
    }

    fn get_z(&self) -> Option<f64> {
        Some(self.z)
    }

    fn get_m(&self) -> Option<f64> {
        Some(self.m)
    }

    fn dimension(&self) -> u32 {
        Dimension::ZM as u32
    }

    fn new_point(
        x: f64,
        y: f64,
        srid: Option<u32>,
        z: Option<f64>,
        m: Option<f64>,
    ) -> Result<Self, PointConstructorError> {
        if z.is_none() {
            return Err(PointConstructorError {
                reason: "Z is not defined, but mandatory for PointZM".to_string(),
            });
        }
        if m.is_none() {
            return Err(PointConstructorError {
                reason: "M is not defined, but mandatory for PointZM".to_string(),
            });
        }
        Ok(PointZM {
            x,
            y,
            z: z.unwrap(),
            m: m.unwrap(),
            srid,
        })
    }
}

macro_rules! impl_point_from_sql {
    ($p:ident) => {
        impl FromSql<Geometry, Pg> for $p {
            fn from_sql(bytes: pg::PgValue) -> deserialize::Result<Self> {
                let mut r = Cursor::new(bytes.as_bytes());
                let end = r.read_u8()?;
                if end == BIG_ENDIAN {
                    read_point::<BigEndian, $p>(&mut r)
                } else {
                    read_point::<LittleEndian, $p>(&mut r)
                }
            }
        }

        impl FromSql<Geography, Pg> for $p {
            fn from_sql(bytes: pg::PgValue) -> deserialize::Result<Self> {
                let mut r = Cursor::new(bytes.as_bytes());
                let end = r.read_u8()?;
                if end == BIG_ENDIAN {
                    read_point::<BigEndian, $p>(&mut r)
                } else {
                    read_point::<LittleEndian, $p>(&mut r)
                }
            }
        }

        impl ToSql<Geometry, Pg> for $p {
            fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
                write_point(self, self.get_srid(), out)?;
                Ok(IsNull::No)
            }
        }

        impl ToSql<Geography, Pg> for $p {
            fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
                write_point(self, self.get_srid(), out)?;
                Ok(IsNull::No)
            }
        }
    };
}

impl_point_from_sql!(Point);
impl_point_from_sql!(PointZ);
impl_point_from_sql!(PointM);
impl_point_from_sql!(PointZM);

pub fn write_point<T>(point: &T, srid: Option<u32>, out: &mut Output<Pg>) -> serialize::Result
where
    T: PointT + EwkbSerializable,
{
    write_ewkb_header(point, srid, out)?;
    write_point_coordinates(point, out)?;
    Ok(IsNull::No)
}

pub fn write_point_coordinates<T>(point: &T, out: &mut Output<Pg>) -> serialize::Result
where
    T: PointT,
{
    out.write_f64::<LittleEndian>(point.get_x())?;
    out.write_f64::<LittleEndian>(point.get_y())?;
    if point.get_z().is_some() {
        out.write_f64::<LittleEndian>(point.get_z().unwrap())?;
    }
    if point.get_m().is_some() {
        out.write_f64::<LittleEndian>(point.get_m().unwrap())?;
    }
    Ok(IsNull::No)
}

fn read_point<T, P>(cursor: &mut Cursor<&[u8]>) -> deserialize::Result<P>
where
    T: byteorder::ByteOrder,
    P: PointT,
{
    let g_header = read_ewkb_header::<T>(GeometryType::Point, cursor)?;
    read_point_coordinates::<T, P>(cursor, g_header.g_type, g_header.srid)
}

pub fn read_point_coordinates<T, P>(
    cursor: &mut Cursor<&[u8]>,
    g_type: u32,
    srid: Option<u32>,
) -> deserialize::Result<P>
where
    T: byteorder::ByteOrder,
    P: PointT,
{
    let x = cursor.read_f64::<T>()?;
    let y = cursor.read_f64::<T>()?;
    let mut z = None;
    if g_type & Dimension::Z as u32 == Dimension::Z as u32 {
        z = Some(cursor.read_f64::<T>()?);
    }
    let mut m = None;
    if g_type & Dimension::M as u32 == Dimension::M as u32 {
        m = Some(cursor.read_f64::<T>()?);
    }
    Ok(P::new_point(x, y, srid, z, m)?)
}
