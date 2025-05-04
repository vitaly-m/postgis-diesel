#![cfg(feature = "postgres")]
#[macro_use]
extern crate diesel;

use std::env;
use std::sync::Once;

use diesel::pg::PgConnection;
use diesel::Connection;
use diesel::{QueryDsl, RunQueryDsl};
use dotenvy::dotenv;

use postgis_diesel::functions::*;
use postgis_diesel::types::Point;

use crate::diesel::ExpressionMethods;
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

fn establish_connection() -> PgConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");

    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

fn initialize() -> PgConnection {
    let mut conn = establish_connection();
    INIT.call_once(|| {
        let _ = diesel::sql_query("CREATE EXTENSION IF NOT EXISTS postgis").execute(&mut conn);
        let _ = diesel::sql_query("DROP TABLE geom_accessor_functions").execute(&mut conn);

        let _ = diesel::sql_query(
            "CREATE TABLE geom_accessor_functions
(
    id                SERIAL PRIMARY KEY,
    name              TEXT NOT NULL,
    point             geometry(POINT, 4326) NOT NULL
)",
        )
        .execute(&mut conn);
        let north_sample = NewGeometrySample {
            name: "northern".to_string(),
            point: Point {
                x: 1.0,
                y: 0.0,
                srid: Some(4326),
            },
        };
        let east_sample = NewGeometrySample {
            name: "eastern".to_string(),
            point: Point {
                x: 0.0,
                y: 1.0,
                srid: Some(4326),
            },
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
fn point_x_test() {
    let mut conn = initialize();
    let found_samples: Vec<GeometrySample> = geom_accessor_functions::table
        .filter(st_x(geom_accessor_functions::point).lt(0.0))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(0, found_samples.len());
    let found_samples: Vec<GeometrySample> = geom_accessor_functions::table
        .filter(st_x(geom_accessor_functions::point).gt(0.0))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(1, found_samples.len());
    for gs in found_samples {
        assert_eq!("northern".to_string(), gs.name);
    }
}

#[test]
fn point_y_test() {
    let mut conn = initialize();
    let found_samples: Vec<GeometrySample> = geom_accessor_functions::table
        .filter(st_y(geom_accessor_functions::point).lt(0.0))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(0, found_samples.len());
    let found_samples: Vec<GeometrySample> = geom_accessor_functions::table
        .filter(st_y(geom_accessor_functions::point).gt(0.0))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(1, found_samples.len());
    for gs in found_samples {
        assert_eq!("eastern".to_string(), gs.name);
    }
}
