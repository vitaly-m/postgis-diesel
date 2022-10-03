use std::fmt;

use crate::sql_types::Geometry;


#[derive(Debug, Clone)]
pub struct PointConstructorError {
    pub reason: String,
}

impl fmt::Display for PointConstructorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "can't construct point: {}", self.reason)
    }
}

impl std::error::Error for PointConstructorError {}

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

#[derive(Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
pub struct MultiPoint<T> {
    pub points: Vec<T>,
    pub srid: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
pub struct LineString<T> {
    pub points: Vec<T>,
    pub srid: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
pub struct MultiLineString<T> {
    pub lines: Vec<LineString<T>>,
    pub srid: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
pub struct Polygon<T> {
    pub rings: Vec<Vec<T>>,
    pub srid: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
pub struct MultiPolygon<T> {
    pub polygons: Vec<Polygon<T>>,
    pub srid: Option<u32>,
}


#[derive(Clone, Debug, PartialEq, FromSqlRow)]
pub enum GeometryContainer<T> {
    Point(T),
    LineString(LineString<T>),
    Polygon(Polygon<T>),
    MultiPoint(MultiPoint<T>),
    MultiLineString(MultiLineString<T>),
    MultiPolygon(MultiPolygon<T>),
    GeometryCollection(GeometryCollection<T>),
}

#[derive(Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
pub struct GeometryCollection<T> {
    pub geometries: Vec<GeometryContainer<T>>,
    pub srid: Option<u32>,
}