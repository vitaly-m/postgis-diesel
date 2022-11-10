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

#[derive(SqlType, QueryId)]
#[diesel(postgres_type(name = "geography"))]
pub struct Geography;
