#[derive(SqlType, QueryId)]
#[postgres(type_name = "geometry")]
pub struct Geometry;