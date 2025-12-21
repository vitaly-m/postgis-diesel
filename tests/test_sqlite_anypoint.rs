#![cfg(feature = "sqlite")]
//! Submodule to test the use the `AnyPoint` type with SQLite backend.

#[macro_use]
extern crate diesel;

use std::sync::Once;

use diesel::Connection;
use diesel::SqliteConnection;
use diesel::{QueryDsl, RunQueryDsl, TextExpressionMethods};

use postgis_diesel::types::{AnyPoint, PointT};

static INIT: Once = Once::new();

#[derive(Insertable)]
#[diesel(table_name = geom_accessor_functions)]
struct NewGeometrySample {
    name: String,
    point: AnyPoint,
}

#[derive(Queryable)]
#[diesel(table_name = geom_accessor_functions)]
#[allow(dead_code)]
struct GeometrySample {
    id: i32,
    name: String,
    point: AnyPoint,
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

fn new_anypoint(x: f64, y: f64, z: Option<f64>, m: Option<f64>) -> AnyPoint {
    AnyPoint::new_point(x, y, Some(4326), z, m).unwrap()
}

fn establish_sqlite_connection() -> SqliteConnection {
    // We delete the database file if it exists
    let _ = std::fs::remove_file("test_anypoint.sqlite");

    let mut conn =
        SqliteConnection::establish("test_anypoint.sqlite").expect("Error connecting to sqlite");
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

        // Insert samples individually to avoid batch insert issues
        let sample_2d = NewGeometrySample {
            name: "point_2d".to_string(),
            point: new_anypoint(72.0, 64.0, None, None),
        };
        let sample_3d = NewGeometrySample {
            name: "point_3d".to_string(),
            point: new_anypoint(72.0, 64.0, Some(10.0), None),
        };
        let sample_m = NewGeometrySample {
            name: "point_m".to_string(),
            point: new_anypoint(72.0, 64.0, None, Some(11.0)),
        };
        let sample_4d = NewGeometrySample {
            name: "point_4d".to_string(),
            point: new_anypoint(72.0, 64.0, Some(10.0), Some(11.0)),
        };

        diesel::insert_into(geom_accessor_functions::table)
            .values(&sample_2d)
            .execute(&mut conn)
            .unwrap();
        diesel::insert_into(geom_accessor_functions::table)
            .values(&sample_3d)
            .execute(&mut conn)
            .unwrap();
        diesel::insert_into(geom_accessor_functions::table)
            .values(&sample_m)
            .execute(&mut conn)
            .unwrap();
        diesel::insert_into(geom_accessor_functions::table)
            .values(&sample_4d)
            .execute(&mut conn)
            .unwrap();
    });
    conn
}

#[test]
fn point_test_sqlite() {
    let mut conn = establish_sqlite_connection();
    let found_samples: Vec<GeometrySample> = geom_accessor_functions::table
        .filter(geom_accessor_functions::name.like("point_%"))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(4, found_samples.len());

    // Verify each point type was stored correctly
    for sample in found_samples {
        match sample.name.as_str() {
            "point_2d" => match sample.point {
                AnyPoint::Point(p) => {
                    assert_eq!(p.x, 72.0);
                    assert_eq!(p.y, 64.0);
                    assert_eq!(p.srid, Some(4326));
                }
                _ => panic!("Expected Point variant"),
            },
            "point_3d" => match sample.point {
                AnyPoint::PointZ(p) => {
                    assert_eq!(p.x, 72.0);
                    assert_eq!(p.y, 64.0);
                    assert_eq!(p.z, 10.0);
                    assert_eq!(p.srid, Some(4326));
                }
                _ => panic!("Expected PointZ variant"),
            },
            "point_m" => match sample.point {
                AnyPoint::PointM(p) => {
                    assert_eq!(p.x, 72.0);
                    assert_eq!(p.y, 64.0);
                    assert_eq!(p.m, 11.0);
                    assert_eq!(p.srid, Some(4326));
                }
                _ => panic!("Expected PointM variant"),
            },
            "point_4d" => match sample.point {
                AnyPoint::PointZM(p) => {
                    assert_eq!(p.x, 72.0);
                    assert_eq!(p.y, 64.0);
                    assert_eq!(p.z, 10.0);
                    assert_eq!(p.m, 11.0);
                    assert_eq!(p.srid, Some(4326));
                }
                _ => panic!("Expected PointZM variant"),
            },
            _ => panic!("Unexpected sample name: {}", sample.name),
        }
    }
}
