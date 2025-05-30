use std::fmt;

use crate::write_to_read_from_sql::ReadFromSql;
use crate::write_to_read_from_sql::WriteToSql;

/// Error which may be returned if point constructed without required fields or has some unexpected fields for type.
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

impl From<PointConstructorError> for std::io::Error {
    fn from(err: PointConstructorError) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::InvalidData, err)
    }
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
///     poinP: Point,
/// }
/// ```
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(
    feature = "diesel",
    derive(diesel::deserialize::FromSqlRow, diesel::expression::AsExpression)
)]
#[cfg_attr(feature = "diesel", diesel(sql_type = crate::sql_types::Geometry))]
#[cfg_attr(feature = "diesel", diesel(sql_type = crate::sql_types::Geography))]
#[cfg_attr(
    all(feature = "serde", not(feature = "serde_geojson")),
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct Point {
    pub x: f64,
    pub y: f64,
    #[cfg_attr(
        all(feature = "serde", not(feature = "serde_geojson")),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub srid: Option<u32>,
}

/// Use that structure in `Insertable` or `Queryable` struct if you work with PointZ geometry.
/// ```
/// #[macro_use] extern crate diesel;
/// use postgis_diesel::types::PointZ;
/// #[derive(Queryable)]
/// struct QueryablePointZExample {
///     id: i32,
///     poinP: PointZ,
/// }
/// ```
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(
    feature = "diesel",
    derive(diesel::deserialize::FromSqlRow, diesel::expression::AsExpression)
)]
#[cfg_attr(feature = "diesel", diesel(sql_type = crate::sql_types::Geometry))]
#[cfg_attr(feature = "diesel", diesel(sql_type = crate::sql_types::Geography))]
#[cfg_attr(
    all(feature = "serde", not(feature = "serde_geojson")),
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct PointZ {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    #[cfg_attr(
        all(feature = "serde", not(feature = "serde_geojson")),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub srid: Option<u32>,
}

/// Use that structure in `Insertable` or `Queryable` struct if you work with PointM geometry.
/// ```
/// #[macro_use] extern crate diesel;
/// use postgis_diesel::types::PointM;
/// #[derive(Queryable)]
/// struct QueryablePointMExample {
///     id: i32,
///     poinP: PointM,
/// }
/// ```
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(
    feature = "diesel",
    derive(diesel::deserialize::FromSqlRow, diesel::expression::AsExpression)
)]
#[cfg_attr(feature = "diesel", diesel(sql_type = crate::sql_types::Geometry))]
#[cfg_attr(feature = "diesel", diesel(sql_type = crate::sql_types::Geography))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
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
///     poinP: PointZM,
/// }
/// ```
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(
    feature = "diesel",
    derive(diesel::deserialize::FromSqlRow, diesel::expression::AsExpression)
)]
#[cfg_attr(feature = "diesel", diesel(sql_type = crate::sql_types::Geometry))]
#[cfg_attr(feature = "diesel", diesel(sql_type = crate::sql_types::Geography))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct PointZM {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub m: f64,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub srid: Option<u32>,
}

/// Allows uniform access across the four point types
pub trait PointT: ReadFromSql + WriteToSql + Copy + core::fmt::Debug {
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
#[derive(Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(
    feature = "diesel",
    derive(diesel::deserialize::FromSqlRow, diesel::expression::AsExpression)
)]
#[cfg_attr(feature = "diesel", diesel(sql_type = crate::sql_types::Geometry))]
#[cfg_attr(feature = "diesel", diesel(sql_type = crate::sql_types::Geography))]
#[cfg_attr(
    all(feature = "serde", not(feature = "serde_geojson")),
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct MultiPoint<T> {
    pub points: Vec<T>,
    #[cfg_attr(
        all(feature = "serde", not(feature = "serde_geojson")),
        serde(skip_serializing_if = "Option::is_none")
    )]
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
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "diesel",
    derive(diesel::deserialize::FromSqlRow, diesel::expression::AsExpression)
)]
#[cfg_attr(feature = "diesel", diesel(sql_type = crate::sql_types::Geometry))]
#[cfg_attr(feature = "diesel", diesel(sql_type = crate::sql_types::Geography))]
#[cfg_attr(
    all(feature = "serde", not(feature = "serde_geojson")),
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct LineString<T> {
    pub points: Vec<T>,
    #[cfg_attr(
        all(feature = "serde", not(feature = "serde_geojson")),
        serde(skip_serializing_if = "Option::is_none")
    )]
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
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "diesel",
    derive(diesel::deserialize::FromSqlRow, diesel::expression::AsExpression)
)]
#[cfg_attr(feature = "diesel", diesel(sql_type = crate::sql_types::Geometry))]
#[cfg_attr(feature = "diesel", diesel(sql_type = crate::sql_types::Geography))]
#[cfg_attr(
    all(feature = "serde", not(feature = "serde_geojson")),
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct MultiLineString<T> {
    pub lines: Vec<LineString<T>>,
    #[cfg_attr(
        all(feature = "serde", not(feature = "serde_geojson")),
        serde(skip_serializing_if = "Option::is_none")
    )]
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
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "diesel",
    derive(diesel::deserialize::FromSqlRow, diesel::expression::AsExpression)
)]
#[cfg_attr(feature = "diesel", diesel(sql_type = crate::sql_types::Geometry))]
#[cfg_attr(feature = "diesel", diesel(sql_type = crate::sql_types::Geography))]
#[cfg_attr(
    all(feature = "serde", not(feature = "serde_geojson")),
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct Polygon<T> {
    pub rings: Vec<Vec<T>>,
    #[cfg_attr(
        all(feature = "serde", not(feature = "serde_geojson")),
        serde(skip_serializing_if = "Option::is_none")
    )]
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
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "diesel",
    derive(diesel::deserialize::FromSqlRow, diesel::expression::AsExpression)
)]
#[cfg_attr(feature = "diesel", diesel(sql_type = crate::sql_types::Geometry))]
#[cfg_attr(feature = "diesel", diesel(sql_type = crate::sql_types::Geography))]
#[cfg_attr(
    all(feature = "serde", not(feature = "serde_geojson")),
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct MultiPolygon<T> {
    pub polygons: Vec<Polygon<T>>,
    #[cfg_attr(
        all(feature = "serde", not(feature = "serde_geojson")),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub srid: Option<u32>,
}

/// Represents any type that can appear in a geometry or geography column.
///
/// T is the Point type (Point or PointZ or PointM)
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "diesel",
    derive(diesel::deserialize::FromSqlRow, diesel::expression::AsExpression)
)]
#[cfg_attr(feature = "diesel", diesel(sql_type = crate::sql_types::Geometry))]
#[cfg_attr(feature = "diesel", diesel(sql_type = crate::sql_types::Geography))]
#[cfg_attr(
    all(feature = "serde", not(feature = "serde_geojson")),
    derive(serde::Serialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(
    feature = "serde_geojson",
    serde(
        tag = "type",
        bound(
            deserialize = "T: crate::geojson::GeoJsonGeometry<f64> + PointT + serde::Deserialize<'de>"
        )
    )
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
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
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "diesel",
    derive(diesel::deserialize::FromSqlRow, diesel::expression::AsExpression)
)]
#[cfg_attr(feature = "diesel", diesel(sql_type = crate::sql_types::Geometry))]
#[cfg_attr(feature = "diesel", diesel(sql_type = crate::sql_types::Geography))]
#[cfg_attr(
    all(feature = "serde", not(feature = "serde_geojson")),
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct GeometryCollection<T> {
    pub geometries: Vec<GeometryContainer<T>>,
    #[cfg_attr(
        all(feature = "serde", not(feature = "serde_geojson")),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub srid: Option<u32>,
}

#[cfg(feature = "serde_geojson")]
#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
#[serde(
    tag = "type",
    bound(
        deserialize = "T: crate::geojson::GeoJsonGeometry<f64> + PointT + serde::Deserialize<'de>, P: serde::Deserialize<'de>"
    )
)]
pub struct Feature<T, P: serde::Serialize> {
    pub id: Option<String>,
    pub geometry: Option<GeometryContainer<T>>,
    pub properties: Option<P>,
}

#[cfg(feature = "serde_geojson")]
#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
#[serde(
    tag = "type",
    bound(
        deserialize = "T: crate::geojson::GeoJsonGeometry<f64> + PointT + serde::Deserialize<'de>, P: serde::Deserialize<'de>"
    )
)]
pub struct FeatureCollection<T, P: serde::Serialize> {
    pub features: Vec<Feature<T, P>>,
}

#[cfg(test)]
#[cfg(all(feature = "serde", not(feature = "serde_geojson")))]
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

        #[cfg(feature = "schemars")]
        {
            let schema = schema_for!(Point);
            let schema_json = serde_json::to_string(&schema).unwrap();
            let expected_schema = r#"{"$schema":"http://json-schema.org/draft-07/schema#","title":"Point","description":"Use that structure in `Insertable` or `Queryable` struct if you work with Point geometry. ``` #[macro_use] extern crate diesel; use postgis_diesel::types::Point; #[derive(Queryable)] struct QueryablePointExample { id: i32, poinP: Point, } ```","type":"object","required":["x","y"],"properties":{"srid":{"type":["integer","null"],"format":"uint32","minimum":0.0},"x":{"type":"number","format":"double"},"y":{"type":"number","format":"double"}}}"#;
            assert_eq!(expected_schema, schema_json);

            let schema_for_value = schema_for_value!(point);
            let schema_for_value_json = serde_json::to_string(&schema_for_value).unwrap();
            println!("{}", schema_for_value_json);
            let expected_schema_for_value = r#"{"$schema":"http://json-schema.org/draft-07/schema#","title":"Point","examples":[{"x":72.0,"y":64.0}],"type":"object","properties":{"x":{"type":"number"},"y":{"type":"number"}}}"#;
            assert_eq!(expected_schema_for_value, schema_for_value_json);
        }
    }
}
