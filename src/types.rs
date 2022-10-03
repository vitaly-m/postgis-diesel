use std::fmt;

use crate::sql_types::Geometry;

/// Error which may be returned if point cinstructed without required fields or has some unexpected fields for type.
/// ```
/// use postgis_diesel::types::{PointT, PointZ, Point, PointConstructorError};
/// let point = PointZ::new_point(72.0, 63.0, None, None, None);
/// assert!(point.is_err());
/// assert_eq!(Result::Err(PointConstructorError{reason:"Z is not defined, but mandatory for PointZ".to_string()}), point);
/// let point = Point::new_point(72.0, 63.0, None, Some(10.0), None);
/// assert!(point.is_err());
/// assert_eq!(Result::Err(PointConstructorError{reason:"unexpectedly defined Z Some(10.0) or M None for Point".to_string()}), point);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct PointConstructorError {
    pub reason: String,
}

impl fmt::Display for PointConstructorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "can't construct point: {}", self.reason)
    }
}

impl std::error::Error for PointConstructorError {}

/// Use that structure in `Insertable` or `Queryable` struct if you work with Point geometry.
/// ```
/// #[macro_use] extern crate diesel;
/// use postgis_diesel::types::Point;
/// #[derive(Queryable)]
/// struct QueryablePointExample {
///     id: i32,
///     point: Point,
/// }
/// ```
#[derive(Copy, Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Point {
    pub x: f64,
    pub y: f64,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub srid: Option<u32>,
}

/// Use that structure in `Insertable` or `Queryable` struct if you work with PointZ geometry.
/// ```
/// #[macro_use] extern crate diesel;
/// use postgis_diesel::types::PointZ;
/// #[derive(Queryable)]
/// struct QueryablePointZExample {
///     id: i32,
///     point: PointZ,
/// }
/// ```
#[derive(Copy, Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PointZ {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub srid: Option<u32>,
}

/// Use that structure in `Insertable` or `Queryable` struct if you work with PointM geometry.
/// ```
/// #[macro_use] extern crate diesel;
/// use postgis_diesel::types::PointM;
/// #[derive(Queryable)]
/// struct QueryablePointMExample {
///     id: i32,
///     point: PointM,
/// }
/// ```
#[derive(Copy, Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PointM {
    pub x: f64,
    pub y: f64,
    pub m: f64,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub srid: Option<u32>,
}

/// Use that structure in `Insertable` or `Queryable` struct if you work with PointZM geometry.
/// ```
/// #[macro_use] extern crate diesel;
/// use postgis_diesel::types::PointZM;
/// #[derive(Queryable)]
/// struct QueryablePointZMExample {
///     id: i32,
///     point: PointZM,
/// }
/// ```
#[derive(Copy, Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PointZM {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub m: f64,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
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

/// Use that structure in `Insertable` or `Queryable` struct if you work with MultiPoint geometry.
/// ```
/// #[macro_use] extern crate diesel;
/// use postgis_diesel::types::{MultiPoint,Point};
/// #[derive(Queryable)]
/// struct QueryableMultiPointExample {
///     id: i32,
///     multipoint: MultiPoint<Point>,
/// }
/// ```
#[derive(Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MultiPoint<T> {
    pub points: Vec<T>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub srid: Option<u32>,
}

/// Use that structure in `Insertable` or `Queryable` struct if you work with LineString geometry.
/// ```
/// #[macro_use] extern crate diesel;
/// use postgis_diesel::types::{LineString,Point};
/// #[derive(Queryable)]
/// struct QueryableLineStringExample {
///     id: i32,
///     linestring: LineString<Point>,
/// }
/// ```
#[derive(Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LineString<T> {
    pub points: Vec<T>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub srid: Option<u32>,
}

/// Use that structure in `Insertable` or `Queryable` struct if you work with MultiLineString geometry.
/// ```
/// #[macro_use] extern crate diesel;
/// use postgis_diesel::types::{MultiLineString, LineString,Point};
/// #[derive(Queryable)]
/// struct QueryableMultiLineStringExample {
///     id: i32,
///     multilinestring: MultiLineString<LineString<Point>>,
/// }
/// ```
#[derive(Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MultiLineString<T> {
    pub lines: Vec<LineString<T>>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub srid: Option<u32>,
}

/// Use that structure in `Insertable` or `Queryable` struct if you work with Polygon geometry.
/// ```
/// #[macro_use] extern crate diesel;
/// use postgis_diesel::types::{Polygon,Point};
/// #[derive(Queryable)]
/// struct QueryablePolygonExample {
///     id: i32,
///     polygon: Polygon<Point>,
/// }
/// ```
#[derive(Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Polygon<T> {
    pub rings: Vec<Vec<T>>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub srid: Option<u32>,
}

/// Use that structure in `Insertable` or `Queryable` struct if you work with MultiPolygon geometry.
/// ```
/// #[macro_use] extern crate diesel;
/// use postgis_diesel::types::{MultiPolygon, Polygon,Point};
/// #[derive(Queryable)]
/// struct QueryableMultiPolygonExample {
///     id: i32,
///     multipolygon: MultiPolygon<Polygon<Point>>,
/// }
/// ```
#[derive(Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MultiPolygon<T> {
    pub polygons: Vec<Polygon<T>>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub srid: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, FromSqlRow)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GeometryContainer<T> {
    Point(T),
    LineString(LineString<T>),
    Polygon(Polygon<T>),
    MultiPoint(MultiPoint<T>),
    MultiLineString(MultiLineString<T>),
    MultiPolygon(MultiPolygon<T>),
    GeometryCollection(GeometryCollection<T>),
}

/// Use that structure in `Insertable` or `Queryable` struct if you work with GeometryCollection geometry.
/// ```
/// #[macro_use] extern crate diesel;
/// use postgis_diesel::types::{GeometryCollection, GeometryContainer, Point};
/// #[derive(Queryable)]
/// struct QueryableGeometryCollectionExample {
///     id: i32,
///     geometrycollection: GeometryCollection<GeometryContainer<Point>>,
/// }
/// ```
#[derive(Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GeometryCollection<T> {
    pub geometries: Vec<GeometryContainer<T>>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub srid: Option<u32>,
}

#[cfg(test)]
#[cfg(feature = "serde")]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_point_serde() {
        let point = Point::new(72.0, 64.0, None);
        let expected_point = "{\"x\":72.0,\"y\":64.0}";
        let point_from_json = serde_json::from_str(expected_point).unwrap();
        assert_eq!(point, point_from_json);
        let point_json = serde_json::to_string(&point).unwrap();
        assert_eq!(expected_point, point_json);
    }
}
