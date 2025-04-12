//! Submodule to test the use the `MultiPolygon` type with SQLite backend.

#[macro_use]
extern crate diesel;

use std::sync::Once;

use diesel::SqliteConnection;

use diesel::Connection;
use diesel::RunQueryDsl;

use postgis_diesel::types::MultiPolygon;
use postgis_diesel::types::Point;
static INIT: Once = Once::new();

#[derive(Insertable)]
#[diesel(table_name = geom_accessor_functions)]
struct NewGeometrySample {
    name: String,
    multipolygon: MultiPolygon<Point>,
}

#[derive(Queryable)]
#[diesel(table_name = geom_accessor_functions)]
#[allow(dead_code)]
struct GeometrySample {
    id: i32,
    name: String,
    multipolygon: MultiPolygon<Point>,
}

table! {
    use postgis_diesel::sql_types::*;
    use diesel::sql_types::*;
    geom_accessor_functions (id) {
        id -> Int4,
        name -> Text,
        multipolygon -> Geometry,
    }
}

fn new_point(x: f64, y: f64) -> Point {
    Point::new(x, y, Some(4326))
}

fn establish_sqlite_connection() -> SqliteConnection {
    // We delete the database file if it exists
    let _ = std::fs::remove_file("test_multipolygon.sqlite");

    let mut conn = SqliteConnection::establish("test_multipolygon.sqlite")
        .expect("Error connecting to sqlite");
    INIT.call_once(|| {
        let _ = diesel::sql_query("DROP TABLE geom_accessor_functions").execute(&mut conn);

        let _ = diesel::sql_query(
            "CREATE TABLE geom_accessor_functions
(
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    name              TEXT NOT NULL,
    multipolygon      BLOB NOT NULL
)",
        )
        .execute(&mut conn);
        let mut multipolygon1 = MultiPolygon::new(Some(4326));
        multipolygon1
            .add_empty_polygon()
            .add_points([
                new_point(72.0, 64.0),
                new_point(73.0, 65.0),
                new_point(71.0, 62.0),
                new_point(72.0, 64.0),
            ]).unwrap()
            .add_empty_polygon()
            .add_points([
                new_point(75.0, 64.0),
                new_point(74.0, 65.0),
                new_point(74.0, 62.0),
                new_point(75.0, 64.0),
            ]).unwrap();
        let north_sample = NewGeometrySample {
            name: "northern".to_string(),
            multipolygon: multipolygon1,
        };
        let mut multipolygon2 = MultiPolygon::new(Some(4326));
        multipolygon2
            .add_empty_polygon()
            .add_points([
                new_point(72.0, 64.0),
                new_point(73.0, 65.0),
                new_point(71.0, 62.0),
                new_point(72.0, 64.0),
            ]).unwrap()
            .add_empty_polygon()
            .add_points([
                new_point(75.0, 64.0),
                new_point(74.0, 65.0),
                new_point(74.0, 62.0),
                new_point(75.0, 64.0),
            ]).unwrap();
        let east_sample = NewGeometrySample {
            name: "eastern".to_string(),
            multipolygon: multipolygon2,
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
fn multipolygon_test_sqlite() {
    let mut conn = establish_sqlite_connection();
    let found_samples: Vec<GeometrySample> = geom_accessor_functions::table
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(2, found_samples.len());
}
