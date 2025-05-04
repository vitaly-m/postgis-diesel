#[cfg(feature = "diesel")]
#[macro_use]
extern crate diesel;

#[cfg(feature = "schemars")]
#[macro_use]
extern crate schemars;

pub mod errors;
mod ewkb;
#[cfg(feature = "diesel")]
pub mod functions;
#[cfg(feature = "diesel")]
pub mod functions_nullable;
#[cfg(feature = "serde_geojson")]
mod geojson;
mod geometrycollection;
mod geometrycontainer;
mod linestring;
mod multiline;
mod multipoint;
mod multipolygon;
#[cfg(feature = "diesel")]
pub mod operators;
mod points;
mod polygon;
pub mod sql_types;
mod to_and_from_sql_geography;
mod to_and_from_sql_geometry;
pub mod types;
mod write_to_read_from_sql;
