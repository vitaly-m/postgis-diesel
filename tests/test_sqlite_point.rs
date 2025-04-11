//! Submodule to test the use the `Point` type with SQLite backend.

#[macro_use]
extern crate diesel;

use std::sync::Once;

use diesel::SqliteConnection;

use diesel::Connection;
use diesel::RunQueryDsl;

use postgis_diesel::types::Point;
static INIT: Once = Once::new();

#[derive(Insertable)]
#[diesel(table_name = geom_accessor_functions)]
struct NewGeometrySample {
    name: String,
    point: Point,
}

#[derive(Queryable)]
#[diesel(table_name = geom_accessor_functions)]
#[allow(dead_code)]
struct GeometrySample {
    id: i32,
    name: String,
    point: Point,
}

table! {
    use postgis_diesel::sql_types::*;
    use diesel::sql_types::*;
    geom_accessor_functions (id) {
        id -> Int4,
        name -> Text,
        point -> Geometry,
    }
}

fn new_point(x: f64, y: f64) -> Point {
    Point::new(x, y, Some(4326))
}

fn establish_sqlite_connection() -> SqliteConnection {
    // We delete the database file if it exists
    let _ = std::fs::remove_file("test_point.sqlite");

    let mut conn = SqliteConnection::establish("test_point.sqlite").expect("Error connecting to sqlite");
    INIT.call_once(|| {
        let _ = diesel::sql_query("DROP TABLE geom_accessor_functions").execute(&mut conn);

        let _ = diesel::sql_query(
            "CREATE TABLE geom_accessor_functions
(
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    name              TEXT NOT NULL,
    point             BLOB NOT NULL
)",
        )
        .execute(&mut conn);
        let north_sample = NewGeometrySample {
            name: "northern".to_string(),
            point: new_point(72.0, 64.0),
        };
        let east_sample = NewGeometrySample {
            name: "eastern".to_string(),
            point: new_point(64.0, 64.0),
        };
        let samples = vec![north_sample, east_sample];
        diesel::insert_into(geom_accessor_functions::table)
            .values(&samples)
            .execute(&mut conn)
            .unwrap();
    });
    conn
}

#[test]
fn point_test_sqlite() {
    let mut conn = establish_sqlite_connection();
    let found_samples: Vec<GeometrySample> = geom_accessor_functions::table
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(2, found_samples.len());
}
