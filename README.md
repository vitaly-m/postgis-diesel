# PostGIS Diesel
Extension for Diesel framework to support PostGIS types. 

# Example of Usage
To ensure that the `Geometry` type is in scope, read [this guide] and add `postgis_diesel::sql_types::*` 
to the import_types key in your `diesel.toml` file.

Assume that the table is defined like this:
```sql
CREATE EXTENSION IF NOT EXISTS postgis;
CREATE TABLE geometry_samples
(
    id         SERIAL                    PRIMARY KEY,
    point      geometry(Point,4326)      NOT NULL,
    linestring geometry(Linestring,4326) NOT NULL
);
```

Then Rust code may look like this:
```rust
#[macro_use]
extern crate diesel;

use postgis_diesel::operators::*;
use postgis_diesel::types::*;

#[derive(Insertable)]
#[diesel(table_name = geometry_samples)]
struct NewGeometrySample {
    point: Point,
    linestring: LineString<Point>,
}

#[derive(Queryable)]
struct GeometrySample {
    id: i32,
    point: Point,
    linestring: LineString<Point>,
}

table! {
    use postgis_diesel::sql_types::*;
    use diesel::sql_types::*;
    geometry_samples (id) {
        id -> Int4,
        point -> Geometry,
        linestring -> Geometry,
    }
}
```
See [integration test](tests/integration_test.rs) for more complete example.

[this guide]: http://diesel.rs/guides/configuring-diesel-cli/

## How to Remove Automatically Generated Types From Schema

1. Generate schema file with diesel `print-schema > src/full_schema.rs`.
2. Remove not required SQL types from it and save to `src/schema.rs`.
3. Run `diff -U6 src/full_schema.rs src/schema.rs > src/schema.patch`.
4. Add `patch_file = "src/schema.patch"` to diesel.toml.
5. Remove `src/full_schema.rs`, check that `diesel print-schema > src/schema.rs` will not add Geometry type.

Example of patch file:
```diff
@@ -1,12 +1,9 @@
 // @generated automatically by Diesel CLI.
 
 pub mod sql_types {
-    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
-    #[diesel(postgres_type(name = "geometry"))]
-    pub struct Geometry;
 
     #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
     #[diesel(postgres_type(name = "intensity"))]
     pub struct Intensity;
 
     #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
@@ -52,13 +49,12 @@
 
 diesel::table! {
     use diesel::sql_types::*;
     use postgis_diesel::sql_types::*;
     use super::sql_types::Intensity;
     use super::sql_types::Triggermethod;
-    use super::sql_types::Geometry;
 
     laps (activity_id, started_at, manual_track) {
         activity_id -> Uuid,
         started_at -> Timestamptz,
         total_time_seconds -> Float8,
         distance_meters -> Float8,
```

# How to Run Tests

1. Start Postgis DB 
```sh
docker compose up
```
2. Run tests
```
cargo test
```
