/// SQL types which may be used in table definition.
/// ```
///#[macro_use] extern crate diesel;
///table! {
///    use postgis_diesel::sql_types::*;
///    use diesel::sql_types::*;
///    geometry_example (id) {
///        id -> Int4,
///        point -> Nullable<Geometry>,
///        linestring -> Geometry,
///    }
///}
/// ```
#[derive(SqlType, QueryId)]
#[diesel(postgres_type(name = "geometry"))]
pub struct Geometry;

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
        } else if t & 1 == 1 {
            return Self::Point;
        } else {
            return Self::Unknown;
        }
    }
}