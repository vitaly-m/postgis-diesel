#[cfg(feature = "diesel")]
#[macro_use]
extern crate diesel;
#[cfg(any(feature = "serde", feature = "serde_geojson"))]
#[macro_use]
extern crate serde;

#[cfg(feature = "schemars")]
#[macro_use]
extern crate schemars;

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
pub mod types;
