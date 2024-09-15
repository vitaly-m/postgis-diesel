#[macro_use]
extern crate diesel;

use std::env;
use std::sync::Once;

use diesel::pg::PgConnection;
use diesel::Connection;
use diesel::{QueryDsl, RunQueryDsl};
use dotenvy::dotenv;

use postgis_diesel::functions::*;
use postgis_diesel::types::{LineString, Point, PointZ, Polygon};

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

#[test]
fn point_z_test() {
    let mut conn = initialize();
    let z: Option<f64> = diesel::select(st_z(Point {
        x: 1.0,
        y: 1.0,
        srid: Some(4326),
    }))
    .first(&mut conn)
    .unwrap();
    assert_eq!(None, z);
    let z: Option<f64> = diesel::select(st_z(PointZ {
        x: 1.0,
        y: 1.0,
        z: 1.0,
        srid: Some(4326),
    }))
    .first(&mut conn)
    .unwrap();
    assert_eq!(Some(1.0), z);
}

#[test]
fn dimensions_test() {
    let mut conn = initialize();
    let point_dim = diesel::select(st_dimensions(Point {
        x: 1.0,
        y: 1.0,
        srid: Some(4326),
    }))
    .first(&mut conn)
    .unwrap();
    assert_eq!(0, point_dim);
    let null_point: Option<Point> = None;
    let point_dim: Option<i32> = diesel::select(postgis_diesel::functions_nullable::st_dimensions(
        null_point,
    ))
    .first(&mut conn)
    .unwrap();
    assert_eq!(None, point_dim);
    let line_dim = diesel::select(st_dimensions(LineString::<Point> {
        points: Vec::new(),
        srid: None,
    }))
    .first(&mut conn)
    .unwrap();
    assert_eq!(1, line_dim);
}

#[test]
fn end_point_test() {
    let mut conn = initialize();
    let first_point = Point {
        x: 1.0,
        y: 1.0,
        srid: Some(4326),
    };
    let end_point = Point {
        x: 2.0,
        y: 2.0,
        srid: Some(4326),
    };
    let found_end_point = diesel::select(st_end_point(LineString {
        points: vec![first_point, end_point],
        srid: Some(4326),
    }))
    .first(&mut conn)
    .unwrap();
    assert_eq!(Some(end_point), found_end_point);
    let found_end_point = diesel::select(st_end_point(first_point))
        .first(&mut conn)
        .unwrap();
    assert_eq!(None::<Point>, found_end_point);
    let found_end_point = diesel::select(postgis_diesel::functions_nullable::st_end_point(
        None::<LineString<Point>>,
    ))
    .first(&mut conn)
    .unwrap();
    assert_eq!(None::<Point>, found_end_point);
}

#[test]
fn envelop_test() {
    let mut conn = initialize();
    let envelop: Point = diesel::select(st_envelope(Point {
        x: 1.0,
        y: 3.0,
        srid: None,
    }))
    .first(&mut conn)
    .unwrap();
    assert_eq!(
        Point {
            x: 1.0,
            y: 3.0,
            srid: None,
        },
        envelop
    );
    let envelop: Option<Point> = diesel::select(postgis_diesel::functions_nullable::st_envelope(
        None::<Point>,
    ))
    .first(&mut conn)
    .unwrap();
    assert_eq!(None::<Point>, envelop)
}

#[test]
fn exterior_ring_test() {
    let mut conn = initialize();
    let mut polygon = Polygon::new(None);
    polygon.add_points(vec![
        PointZ::new(0.0, 0.0, 1.0, None),
        PointZ::new(1.0, 1.0, 1.0, None),
        PointZ::new(1.0, 2.0, 1.0, None),
        PointZ::new(1.0, 1.0, 1.0, None),
        PointZ::new(0.0, 0.0, 1.0, None),
    ]);
    let mut expected = LineString::new(None);
    expected.add_points(vec![
        PointZ::new(0.0, 0.0, 1.0, None),
        PointZ::new(1.0, 1.0, 1.0, None),
        PointZ::new(1.0, 2.0, 1.0, None),
        PointZ::new(1.0, 1.0, 1.0, None),
        PointZ::new(0.0, 0.0, 1.0, None),
    ]);
    let ring = diesel::select(st_exterior_ring(polygon))
        .first(&mut conn)
        .unwrap();
    assert_eq!(Some(expected), ring);
    let ring = diesel::select(st_exterior_ring(Point::new(0.0, 0.0, None)))
        .first(&mut conn)
        .unwrap();
    assert_eq!(None::<LineString<Point>>, ring);
    let ring = diesel::select(postgis_diesel::functions_nullable::st_exterior_ring(
        None::<Point>,
    ))
    .first(&mut conn)
    .unwrap();
    assert_eq!(None::<LineString<Point>>, ring)
}
