use std::io::Cursor;

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use diesel::{
    backend::Backend,
    deserialize,
    pg::Pg,
    serialize::{self, Output},
    types::{FromSql, IsNull, ToSql},
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
#[sql_type = "Geometry"]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub srid: Option<u32>,
}

#[derive(Copy, Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[sql_type = "Geometry"]
pub struct PointZ {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub srid: Option<u32>,
}

#[derive(Copy, Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[sql_type = "Geometry"]
pub struct PointM {
    pub x: f64,
    pub y: f64,
    pub m: f64,
    pub srid: Option<u32>,
}

#[derive(Copy, Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[sql_type = "Geometry"]
pub struct PointZM {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub m: f64,
    pub srid: Option<u32>,
}

pub enum PointE {
    Point(Point),
    PointZ(PointZ),
    PointM(PointM),
    PointZM(PointZM),
}

pub trait PointT {
    fn get_x(&self) -> f64;
    fn get_y(&self) -> f64;
    fn get_srid(&self) -> Option<u32>;
    fn get_z(&self) -> Option<f64>;
    fn get_m(&self) -> Option<f64>;
}

#[derive(Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[sql_type = "Geometry"]
pub struct LineString<T> {
    pub points: Vec<T>,
    pub srid: Option<u32>,
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
}

fn new_point(x: f64, y: f64, srid: Option<u32>, z: Option<f64>, m: Option<f64>) -> PointE {
    if z.is_none() && m.is_none() {
        PointE::Point(Point { x, y, srid })
    } else if z.is_some() && m.is_some() {
        PointE::PointZM(PointZM {
            x,
            y,
            z: z.unwrap(),
            m: m.unwrap(),
            srid,
        })
    } else if z.is_some() {
        PointE::PointZ(PointZ {
            x,
            y,
            z: z.unwrap(),
            srid,
        })
    } else {
        PointE::PointM(PointM {
            x,
            y,
            m: m.unwrap(),
            srid,
        })
    }
}

impl FromSql<Geometry, Pg> for Point {
    fn from_sql(bytes: Option<&<Pg as Backend>::RawValue>) -> deserialize::Result<Self> {
        if let PointE::Point(point) = read_point(bytes)? {
            return Ok(point);
        }
        Err("Geometry is not a Point".into())
    }
}

impl FromSql<Geometry, Pg> for PointZ {
    fn from_sql(bytes: Option<&<Pg as Backend>::RawValue>) -> deserialize::Result<Self> {
        if let PointE::PointZ(point) = read_point(bytes)? {
            return Ok(point);
        }
        Err("Geometry is not a PointZ".into())
    }
}

impl FromSql<Geometry, Pg> for PointM {
    fn from_sql(bytes: Option<&<Pg as Backend>::RawValue>) -> deserialize::Result<Self> {
        if let PointE::PointM(point) = read_point(bytes)? {
            return Ok(point);
        }
        Err("Geometry is not a PointM".into())
    }
}

impl FromSql<Geometry, Pg> for PointZM {
    fn from_sql(bytes: Option<&<Pg as Backend>::RawValue>) -> deserialize::Result<Self> {
        if let PointE::PointZM(point) = read_point(bytes)? {
            return Ok(point);
        }
        Err("Geometry is not a PointZM".into())
    }
}

impl ToSql<Geometry, Pg> for Point {
    fn to_sql<W: std::io::Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        write_point(self, out)
    }
}

impl ToSql<Geometry, Pg> for PointZ {
    fn to_sql<W: std::io::Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        write_point(self, out)
    }
}

impl ToSql<Geometry, Pg> for PointM {
    fn to_sql<W: std::io::Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        write_point(self, out)
    }
}

impl ToSql<Geometry, Pg> for PointZM {
    fn to_sql<W: std::io::Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        write_point(self, out)
    }
}

fn read_point(bytes: Option<&<Pg as Backend>::RawValue>) -> deserialize::Result<PointE>
{
    let bytes = not_none!(bytes);
    let mut r = Cursor::new(bytes);
    let end = r.read_u8()?;
    if end == BIG_ENDIAN {
        read_point_e::<BigEndian>(&mut r)
    } else {
        read_point_e::<LittleEndian>(&mut r)
    }
}

fn read_point_e<T>(cursor: &mut Cursor<&[u8]>) -> deserialize::Result<PointE>
where
    T: byteorder::ByteOrder,
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
    read_point_with_type_srid::<T>(cursor, g_type, srid)
}

fn read_point_with_type_srid<T>(cursor: &mut Cursor<&[u8]>, g_type: u32, srid: Option<u32>) -> deserialize::Result<PointE> {
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
    Ok(new_point(x, y, srid, z, m))
}

fn write_point<W, T>(point: &T, out: &mut Output<W, Pg>) -> serialize::Result
where
    T: PointT,
    W: std::io::Write,
{
    out.write_u8(LITTLE_ENDIAN)?;
    let mut p_type = GeometryType::Point as u32;
    if point.get_srid().is_some() {
        p_type |= SRID;
    }
    if point.get_z().is_some() {
        p_type |= Dimension::Z as u32;
    }
    if point.get_m().is_some() {
        p_type |= Dimension::M as u32;
    }
    match point.get_srid() {
        Some(srid) => {
            out.write_u32::<LittleEndian>(p_type)?;
            out.write_u32::<LittleEndian>(srid)?;
        }
        None => out.write_u32::<LittleEndian>(p_type)?,
    }
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
