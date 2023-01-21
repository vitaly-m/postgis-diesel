use diesel::sql_types::SingleValue;

/// SQL type which may be used in table definition.
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

/// SQL type which may be used in table definition.
/// ```
///#[macro_use] extern crate diesel;
///table! {
///    use postgis_diesel::sql_types::*;
///    use diesel::sql_types::*;
///    geography_example (id) {
///        id -> Int4,
///        point -> Geography,
///    }
///}
/// ```
#[derive(SqlType, QueryId)]
#[diesel(postgres_type(name = "geography"))]
pub struct Geography;

pub trait GeoType: SingleValue {

}

impl GeoType for Geometry {

}

impl GeoType for Geography {

}
