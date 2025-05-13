#![doc = include_str!("../README.md")]

#[cfg(feature = "schemars")]
#[macro_use]
extern crate schemars;

pub mod errors;
mod ewkb;
pub mod functions;
pub mod functions_nullable;
#[cfg(feature = "serde_geojson")]
mod geojson;
mod geometrycollection;
mod geometrycontainer;
mod linestring;
mod multiline;
mod multipoint;
mod multipolygon;
pub mod operators;
mod points;
mod polygon;
pub mod sql_types;
mod to_and_from_sql_geography;
mod to_and_from_sql_geometry;
pub mod types;
mod write_to_read_from_sql;
