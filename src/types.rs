use std::fmt::{self, Debug};
use std::io::Cursor;

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use diesel::{
    deserialize::{self, FromSql},
    pg::{self, Pg},
    serialize::{self, IsNull, Output, ToSql},
};

use crate::sql_types::*;

#[derive(Debug, PartialEq)]
pub enum GeometryType {
    Point = 1,
    LineString,
    Polygon,
    MultiPoint,
    MultiLineString,
    MultiPolygon,
    GeometryCollection,
    Unknown,
}

pub enum Dimension {
    Z = 0x80000000,
    M = 0x40000000,
    ZM = 0x40000000 | 0x80000000,
}

pub const SRID: u32 = 0x20000000;
const LITTLE_ENDIAN: u8 = 1;
const BIG_ENDIAN: u8 = 0;

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
        } else if t & 1 == 1 {
            return Self::Point;
        } else {
            return Self::Unknown;
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub srid: Option<u32>,
}

#[derive(Copy, Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
pub struct PointZ {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub srid: Option<u32>,
}

#[derive(Copy, Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
pub struct PointM {
    pub x: f64,
    pub y: f64,
    pub m: f64,
    pub srid: Option<u32>,
}

#[derive(Copy, Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
pub struct PointZM {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub m: f64,
    pub srid: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
pub struct LineString<T> {
    pub points: Vec<T>,
    pub srid: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct PointConstructorError {
    reason: String,
}

impl fmt::Display for PointConstructorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "can't construct point: {}", self.reason)
    }
}

impl std::error::Error for PointConstructorError {}

pub trait PointT {
    fn new_point(
        x: f64,
        y: f64,
        srid: Option<u32>,
        z: Option<f64>,
        m: Option<f64>,
    ) -> Result<Self, PointConstructorError>
    where
        Self: Sized;
    fn get_x(&self) -> f64;
    fn get_y(&self) -> f64;
    fn get_srid(&self) -> Option<u32>;
    fn get_z(&self) -> Option<f64>;
    fn get_m(&self) -> Option<f64>;
    fn dimension(&self) -> u32;
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
                reason: format!("unexpectedly defined z {:?} or m {:?} for Point", z, m)
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
    };
}

impl_point_from_sql!(Point);
impl_point_from_sql!(PointZ);
impl_point_from_sql!(PointM);
impl_point_from_sql!(PointZM);

impl ToSql<Geometry, Pg> for Point {
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
        write_point(self, out)
    }
}

impl ToSql<Geometry, Pg> for PointZ {
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
        write_point(self, out)
    }
}

impl ToSql<Geometry, Pg> for PointM {
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
        write_point(self, out)
    }
}

impl ToSql<Geometry, Pg> for PointZM {
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
        write_point(self, out)
    }
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
                "Linestring must contain at least two points but has {}",
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

fn read_point<T, P>(cursor: &mut Cursor<&[u8]>) -> deserialize::Result<P>
where
    T: byteorder::ByteOrder,
    P: PointT,
{
    let g_type = cursor.read_u32::<T>()?;
    if GeometryType::from(g_type) != GeometryType::Point {
        return Err(format!("Geometry {:?} is not a point", GeometryType::from(g_type)).into());
    }
    let mut srid = None;
    // SRID included
    if g_type & SRID == SRID {
        srid = Some(cursor.read_u32::<T>()?);
    }
    read_point_coordinates::<T, P>(cursor, g_type, srid)
}

fn read_linestring<T, P>(cursor: &mut Cursor<&[u8]>) -> deserialize::Result<LineString<P>>
where
    T: byteorder::ByteOrder,
    P: PointT,
{
    let g_type = cursor.read_u32::<T>()?;
    if GeometryType::from(g_type) != GeometryType::LineString {
        return Err(format!(
            "Geometry {:?} is not a linestring",
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

fn read_point_coordinates<T, P>(
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

fn write_point<T>(point: &T, out: &mut Output<Pg>) -> serialize::Result
where
    T: PointT,
{
    out.write_u8(LITTLE_ENDIAN)?;
    let mut p_type = GeometryType::Point as u32;
    if point.get_srid().is_some() {
        p_type |= SRID;
    }
    p_type |= point.dimension();

    match point.get_srid() {
        Some(srid) => {
            out.write_u32::<LittleEndian>(p_type)?;
            out.write_u32::<LittleEndian>(srid)?;
        }
        None => out.write_u32::<LittleEndian>(p_type)?,
    }
    write_point_coordinates(point, out)?;
    Ok(IsNull::No)
}

fn write_point_coordinates<T>(point: &T, out: &mut Output<Pg>) -> serialize::Result
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