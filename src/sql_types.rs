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
#[derive(Clone, Copy)]
#[cfg_attr(
    feature = "diesel",
    derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)
)]
#[cfg_attr(feature = "postgres", diesel(postgres_type(name = "geometry")))]
#[cfg_attr(feature = "sqlite", diesel(sqlite_type(name = "Binary")))]
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
#[derive(Clone, Copy)]
#[cfg_attr(
    feature = "diesel",
    derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)
)]
#[cfg_attr(feature = "postgres", diesel(postgres_type(name = "geography")))]
#[cfg_attr(feature = "sqlite", diesel(sqlite_type(name = "Binary")))]
pub struct Geography;

#[cfg(feature = "diesel")]
pub trait GeoType: diesel::sql_types::SingleValue {}

#[cfg(feature = "diesel")]
impl GeoType for Geometry {}

#[cfg(feature = "diesel")]
impl GeoType for Geography {}
