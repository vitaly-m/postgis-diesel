//! Submodule to test the use the `LineString` type with SQLite backend.

#[macro_use]
extern crate diesel;

use std::sync::Once;

use diesel::SqliteConnection;

use diesel::Connection;
use diesel::RunQueryDsl;

use postgis_diesel::types::LineString;
use postgis_diesel::types::Point;
static INIT: Once = Once::new();

#[derive(Insertable)]
#[diesel(table_name = geom_accessor_functions)]
struct NewGeometrySample {
    name: String,
    point: LineString<Point>,
}

#[derive(Queryable)]
#[diesel(table_name = geom_accessor_functions)]
#[allow(dead_code)]
struct GeometrySample {
    id: i32,
    name: String,
    point: LineString<Point>,
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

fn new_line(points: Vec<(f64, f64)>) -> LineString<Point> {
    let mut l_points = Vec::with_capacity(points.len());
    for p in points {
        l_points.push(Point {
            x: p.0,
            y: p.1,
            srid: Option::Some(4326),
        });
    }
    //just to check that add_points works
    LineString::new(Option::Some(4326))
        .add_points(l_points)
        .unwrap()
        .to_owned()
}

fn establish_sqlite_connection() -> SqliteConnection {
    // We delete the database file if it exists
    let _ = std::fs::remove_file("test_linestring.sqlite");

    let mut conn =
        SqliteConnection::establish("test_linestring.sqlite").expect("Error connecting to sqlite");
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
            point: new_line(vec![(72.0, 64.0), (73.0, 64.0), (23.0, 104.0)]),
        };
        let east_sample = NewGeometrySample {
            name: "eastern".to_string(),
            point: new_line(vec![(72.0, 64.0), (73.0, 64.0)]),
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
