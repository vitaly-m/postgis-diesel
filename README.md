# PostGIS Diesel
Extension for Diesel framework to support PostGIS types. 

# Example of Usage
To ensure that the `Geometry` type is in scope, read [this guide] and add use `postgis_diesel::sql_types::*` 
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
See [intergation test](tests/integration_test.rs) for more complete example.

[this guide]: http://diesel.rs/guides/configuring-diesel-cli/