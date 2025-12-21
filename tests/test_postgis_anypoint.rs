#![cfg(feature = "postgres")]
#[macro_use]
extern crate diesel;

use diesel::pg::PgConnection;
use diesel::{Connection, TextExpressionMethods};
use diesel::{QueryDsl, RunQueryDsl};
use dotenvy::dotenv;
use std::env;
use std::sync::Once;

use postgis_diesel::types::*;

static INIT: Once = Once::new();

#[derive(Insertable)]
#[diesel(table_name = geometry_samples)]
struct NewGeometrySample {
    name: String,
    anypoint: AnyPoint,
}

#[derive(Insertable)]
#[diesel(table_name = geography_samples)]
struct NewGeographySample {
    name: String,
    anypoint: AnyPoint,
}

#[derive(Queryable, Debug, PartialEq)]
struct GeometrySample {
    id: i32,
    name: String,
    anypoint: AnyPoint,
}

#[derive(Queryable, Debug, PartialEq)]
struct GeographySample {
    id: i32,
    name: String,
    anypoint: AnyPoint,
}

table! {
    use postgis_diesel::sql_types::*;
    use diesel::sql_types::*;
    geometry_samples (id) {
        id -> Int4,
        name -> Text,
        anypoint -> Geometry,
    }
}

table! {
    use postgis_diesel::sql_types::*;
    use diesel::sql_types::*;
    geography_samples (id) {
        id -> Int4,
        name -> Text,
        anypoint -> Geography,
    }
}

fn establish_connection() -> PgConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");

    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

fn initialize() -> PgConnection {
    let mut conn = establish_connection();
    INIT.call_once(|| {
        let _ = diesel::sql_query("CREATE EXTENSION IF NOT EXISTS postgis").execute(&mut conn);
        let _ = diesel::sql_query("DROP TABLE geometry_samples").execute(&mut conn);
        let _ = diesel::sql_query("DROP TABLE geography_samples").execute(&mut conn);

        let _ = diesel::sql_query(
            "CREATE TABLE geometry_samples
(
    id                SERIAL PRIMARY KEY,
    name              text,
    anypoint          geometry NOT NULL
)",
        )
        .execute(&mut conn);

        let _ = diesel::sql_query(
            "CREATE TABLE geography_samples
(
    id                SERIAL PRIMARY KEY,
    name              text,
    anypoint          geography NOT NULL
)",
        )
        .execute(&mut conn);
    });
    conn
}

fn new_point(x: f64, y: f64) -> Point {
    Point::new(x, y, Some(4326))
}

fn new_point_z(x: f64, y: f64, z: f64) -> PointZ {
    PointZ::new(x, y, z, Some(4326))
}

fn new_point_m(x: f64, y: f64, m: f64) -> PointM {
    PointM::new(x, y, m, Some(4326))
}

fn new_point_zm(x: f64, y: f64, z: f64, m: f64) -> PointZM {
    PointZM::new(x, y, z, m, Some(4326))
}

#[test]
fn anypoint_geometry_test() {
    let mut conn = initialize();

    // Test all AnyPoint variants with geometry
    let samples = vec![
        NewGeometrySample {
            name: String::from("geom_anypoint_2d"),
            anypoint: AnyPoint::Point(new_point(72.0, 64.0)),
        },
        NewGeometrySample {
            name: String::from("geom_anypoint_3d"),
            anypoint: AnyPoint::PointZ(new_point_z(73.0, 65.0, 12.0)),
        },
        NewGeometrySample {
            name: String::from("geom_anypoint_m"),
            anypoint: AnyPoint::PointM(new_point_m(74.0, 66.0, 15.0)),
        },
        NewGeometrySample {
            name: String::from("geom_anypoint_4d"),
            anypoint: AnyPoint::PointZM(new_point_zm(75.0, 67.0, 16.0, 17.0)),
        },
    ];

    // Insert all samples
    for sample in samples {
        let _ = diesel::insert_into(geometry_samples::table)
            .values(&sample)
            .get_result::<GeometrySample>(&mut conn)
            .expect("Error saving geometry sample");
    }

    // Retrieve and verify all samples
    let found_samples: Vec<GeometrySample> = geometry_samples::table
        .filter(geometry_samples::name.like("geom_anypoint_%"))
        .get_results(&mut conn)
        .unwrap();

    assert_eq!(4, found_samples.len());

    for sample in found_samples {
        match sample.name.as_str() {
            "geom_anypoint_2d" => match sample.anypoint {
                AnyPoint::Point(p) => {
                    assert_eq!(p.x, 72.0);
                    assert_eq!(p.y, 64.0);
                    assert_eq!(p.srid, Some(4326));
                }
                _ => panic!("Expected Point variant"),
            },
            "geom_anypoint_3d" => match sample.anypoint {
                AnyPoint::PointZ(p) => {
                    assert_eq!(p.x, 73.0);
                    assert_eq!(p.y, 65.0);
                    assert_eq!(p.z, 12.0);
                    assert_eq!(p.srid, Some(4326));
                }
                _ => panic!("Expected PointZ variant"),
            },
            "geom_anypoint_m" => match sample.anypoint {
                AnyPoint::PointM(p) => {
                    assert_eq!(p.x, 74.0);
                    assert_eq!(p.y, 66.0);
                    assert_eq!(p.m, 15.0);
                    assert_eq!(p.srid, Some(4326));
                }
                _ => panic!("Expected PointM variant"),
            },
            "geom_anypoint_4d" => match sample.anypoint {
                AnyPoint::PointZM(p) => {
                    assert_eq!(p.x, 75.0);
                    assert_eq!(p.y, 67.0);
                    assert_eq!(p.z, 16.0);
                    assert_eq!(p.m, 17.0);
                    assert_eq!(p.srid, Some(4326));
                }
                _ => panic!("Expected PointZM variant"),
            },
            _ => panic!("Unexpected sample name: {}", sample.name),
        }
    }
}

#[test]
fn anypoint_geography_test() {
    let mut conn = initialize();

    // Test all AnyPoint variants with geography
    let samples = vec![
        NewGeographySample {
            name: String::from("geog_anypoint_2d"),
            anypoint: AnyPoint::Point(new_point(72.0, 64.0)),
        },
        NewGeographySample {
            name: String::from("geog_anypoint_3d"),
            anypoint: AnyPoint::PointZ(new_point_z(73.0, 65.0, 12.0)),
        },
        NewGeographySample {
            name: String::from("geog_anypoint_m"),
            anypoint: AnyPoint::PointM(new_point_m(74.0, 66.0, 15.0)),
        },
        NewGeographySample {
            name: String::from("geog_anypoint_4d"),
            anypoint: AnyPoint::PointZM(new_point_zm(75.0, 67.0, 16.0, 17.0)),
        },
    ];

    // Insert all samples
    for sample in samples {
        let _ = diesel::insert_into(geography_samples::table)
            .values(&sample)
            .get_result::<GeographySample>(&mut conn)
            .expect("Error saving geography sample");
    }

    // Retrieve and verify all samples
    let found_samples: Vec<GeographySample> = geography_samples::table
        .filter(geography_samples::name.like("geog_anypoint_%"))
        .get_results(&mut conn)
        .unwrap();

    assert_eq!(4, found_samples.len());

    for sample in found_samples {
        match sample.name.as_str() {
            "geog_anypoint_2d" => match sample.anypoint {
                AnyPoint::Point(p) => {
                    assert_eq!(p.x, 72.0);
                    assert_eq!(p.y, 64.0);
                    assert_eq!(p.srid, Some(4326));
                }
                _ => panic!("Expected Point variant"),
            },
            "geog_anypoint_3d" => match sample.anypoint {
                AnyPoint::PointZ(p) => {
                    assert_eq!(p.x, 73.0);
                    assert_eq!(p.y, 65.0);
                    assert_eq!(p.z, 12.0);
                    assert_eq!(p.srid, Some(4326));
                }
                _ => panic!("Expected PointZ variant"),
            },
            "geog_anypoint_m" => match sample.anypoint {
                AnyPoint::PointM(p) => {
                    assert_eq!(p.x, 74.0);
                    assert_eq!(p.y, 66.0);
                    assert_eq!(p.m, 15.0);
                    assert_eq!(p.srid, Some(4326));
                }
                _ => panic!("Expected PointM variant"),
            },
            "geog_anypoint_4d" => match sample.anypoint {
                AnyPoint::PointZM(p) => {
                    assert_eq!(p.x, 75.0);
                    assert_eq!(p.y, 67.0);
                    assert_eq!(p.z, 16.0);
                    assert_eq!(p.m, 17.0);
                    assert_eq!(p.srid, Some(4326));
                }
                _ => panic!("Expected PointZM variant"),
            },
            _ => panic!("Unexpected sample name: {}", sample.name),
        }
    }
}
