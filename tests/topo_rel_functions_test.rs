#![cfg(feature = "postgres")]
#[macro_use]
extern crate diesel;

use std::env;
use std::sync::Once;

use diesel::pg::PgConnection;
use diesel::sql_types::Text;
use diesel::{Connection, IntoSql};
use diesel::{QueryDsl, RunQueryDsl};
use dotenvy::dotenv;

use postgis_diesel::functions::*;
use postgis_diesel::sql_types::{Geography, Geometry};
use postgis_diesel::types::*;

use crate::diesel::ExpressionMethods;
static INIT: Once = Once::new();

#[derive(Insertable)]
#[diesel(table_name = topo_rel_functions)]
struct NewGeometrySample {
    name: String,
}

#[derive(Queryable)]
#[diesel(table_name = topo_rel_functions)]
#[allow(dead_code)]
struct GeometrySample {
    id: i32,
    name: String,
}

table! {
    use postgis_diesel::sql_types::*;
    use diesel::sql_types::*;
    topo_rel_functions (id) {
        id -> Int4,
        name -> Text,
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
        let _ = diesel::sql_query("DROP TABLE topo_rel_functions").execute(&mut conn);

        let _ = diesel::sql_query(
            "CREATE TABLE topo_rel_functions
(
    id                SERIAL PRIMARY KEY,
    name              text
)",
        )
        .execute(&mut conn);
        let sample = NewGeometrySample {
            name: "topo_rel_test".to_string(),
        };
        diesel::insert_into(topo_rel_functions::table)
            .values(&sample)
            .execute(&mut conn)
            .unwrap();
    });
    conn
}

#[test]
fn intersect_3d_test() {
    let mut conn = initialize();
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_3d_intersects(
            PointZ::new(0.0, 0.0, 2.0, Some(4326)),
            LineString::new(Some(4326))
                .add_point(PointZ::new(0.0, 0.0, 1.0, Some(4326)))
                .unwrap()
                .add_point(PointZ::new(0.0, 2.0, 3.0, Some(4326)))
                .unwrap()
                .to_owned(),
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(0, found_samples.len());
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_3d_intersects(
            PointZ::new(0.0, 0.0, 1.0, Some(4326)),
            LineString::new(Some(4326))
                .add_point(PointZ::new(0.0, 0.0, 1.0, Some(4326)))
                .unwrap()
                .add_point(PointZ::new(0.0, 2.0, 3.0, Some(4326)))
                .unwrap()
                .to_owned(),
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(1, found_samples.len());
    for gs in found_samples {
        assert_eq!("topo_rel_test".to_string(), gs.name);
    }
}

#[test]
fn contains_test() {
    let mut conn = initialize();
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_contains(
            Polygon::new(Some(4326))
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(2.0, 2.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(2.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .to_owned(),
            Point::new(3.0, 1.0, Some(4326)),
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(0, found_samples.len());
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_contains(
            Polygon::new(Some(4326))
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(2.0, 2.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(2.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .to_owned(),
            Point::new(1.0, 1.0, Some(4326)),
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(1, found_samples.len());
    for gs in found_samples {
        assert_eq!("topo_rel_test".to_string(), gs.name);
    }
}

#[test]
fn contains_properly_test() {
    let mut conn = initialize();
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_contains_properly(
            Polygon::new(Some(4326))
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(2.0, 2.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(2.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .to_owned(),
            Point::new(3.0, 1.0, Some(4326)),
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(0, found_samples.len());
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_contains_properly(
            Polygon::new(Some(4326))
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(2.0, 2.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(2.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .to_owned(),
            Point::new(1.0, 1.0, Some(4326)),
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(1, found_samples.len());
    for gs in found_samples {
        assert_eq!("topo_rel_test".to_string(), gs.name);
    }
}

#[test]
fn covered_by_geometry_test() {
    let mut conn = initialize();
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_covered_by::<Geometry, Point, Polygon<Point>>(
            Point::new(3.0, 1.0, Some(4326)),
            Polygon::new(Some(4326))
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(2.0, 2.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(2.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .to_owned(),
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(0, found_samples.len());
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_covered_by::<Geometry, Point, Polygon<Point>>(
            Point::new(1.0, 1.0, Some(4326)),
            Polygon::new(Some(4326))
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(2.0, 2.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(2.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .to_owned(),
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(1, found_samples.len());
    for gs in found_samples {
        assert_eq!("topo_rel_test".to_string(), gs.name);
    }
}

#[test]
fn covered_by_geography_test() {
    let mut conn = initialize();
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_covered_by::<Geography, Point, Polygon<Point>>(
            Point::new(3.0, 1.0, Some(4326)),
            Polygon::new(Some(4326))
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(2.0, 2.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(2.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .to_owned(),
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(0, found_samples.len());
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_covered_by::<Geography, Point, Polygon<Point>>(
            Point::new(1.0, 1.0, Some(4326)),
            Polygon::new(Some(4326))
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(2.0, 2.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(2.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .to_owned(),
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(1, found_samples.len());
    for gs in found_samples {
        assert_eq!("topo_rel_test".to_string(), gs.name);
    }
}

#[test]
fn covers_geometry_test() {
    let mut conn = initialize();
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_covers::<Geometry, Polygon<Point>, Point>(
            Polygon::new(Some(4326))
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(2.0, 2.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(2.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .to_owned(),
            Point::new(3.0, 1.0, Some(4326)),
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(0, found_samples.len());
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_covers::<Geometry, Polygon<Point>, Point>(
            Polygon::new(Some(4326))
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(2.0, 2.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(2.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .to_owned(),
            Point::new(1.0, 1.0, Some(4326)),
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(1, found_samples.len());
    for gs in found_samples {
        assert_eq!("topo_rel_test".to_string(), gs.name);
    }
}

#[test]
fn coveres_geography_test() {
    let mut conn = initialize();
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_covers::<Geography, Polygon<Point>, Point>(
            Polygon::new(Some(4326))
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(2.0, 2.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(2.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .to_owned(),
            Point::new(3.0, 1.0, Some(4326)),
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(0, found_samples.len());
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_covers::<Geography, Polygon<Point>, Point>(
            Polygon::new(Some(4326))
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(2.0, 2.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(2.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .to_owned(),
            Point::new(1.0, 1.0, Some(4326)),
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(1, found_samples.len());
    for gs in found_samples {
        assert_eq!("topo_rel_test".to_string(), gs.name);
    }
}

#[test]
fn crosses_test() {
    let mut conn = initialize();
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_crosses::<LineString<Point>, LineString<Point>>(
            LineString::new(Some(4326))
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .unwrap()
                .to_owned(),
            LineString::new(Some(4326))
                .add_point(Point::new(1.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(1.0, 2.0, Some(4326)))
                .unwrap()
                .to_owned(),
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(0, found_samples.len());
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_crosses::<LineString<Point>, LineString<Point>>(
            LineString::new(Some(4326))
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .unwrap()
                .to_owned(),
            LineString::new(Some(4326))
                .add_point(Point::new(-1.0, 1.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(1.0, 1.0, Some(4326)))
                .unwrap()
                .to_owned(),
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(1, found_samples.len());
    for gs in found_samples {
        assert_eq!("topo_rel_test".to_string(), gs.name);
    }
}

#[test]
fn disjoint_test() {
    let mut conn = initialize();
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_disjoint::<Point, LineString<Point>>(
            Point::new(0.0, 0.0, Some(4326)),
            LineString::new(Some(4326))
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .unwrap()
                .to_owned(),
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(0, found_samples.len());
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_disjoint::<Point, LineString<Point>>(
            Point::new(0.0, 0.0, Some(4326)),
            LineString::new(Some(4326))
                .add_point(Point::new(2.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .unwrap()
                .to_owned(),
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(1, found_samples.len());
    for gs in found_samples {
        assert_eq!("topo_rel_test".to_string(), gs.name);
    }
}

#[test]
fn equals_test() {
    let mut conn = initialize();
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_equals::<LineString<Point>, LineString<Point>>(
            LineString::new(Some(4326))
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .unwrap()
                .to_owned(),
            LineString::new(Some(4326))
                .add_point(Point::new(1.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(1.0, 2.0, Some(4326)))
                .unwrap()
                .to_owned(),
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(0, found_samples.len());
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_equals::<LineString<Point>, LineString<Point>>(
            LineString::new(Some(4326))
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .unwrap()
                .to_owned(),
            LineString::new(Some(4326))
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 1.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .unwrap()
                .to_owned(),
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(1, found_samples.len());
    for gs in found_samples {
        assert_eq!("topo_rel_test".to_string(), gs.name);
    }
}

#[test]
fn intersects_geometry_test() {
    let mut conn = initialize();
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_intersects::<Geometry, Point, LineString<Point>>(
            Point::new(0.0, 0.0, Some(4326)),
            LineString::new(Some(4326))
                .add_point(Point::new(2.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .unwrap()
                .to_owned(),
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(0, found_samples.len());
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_intersects::<Geometry, Point, LineString<Point>>(
            Point::new(0.0, 0.0, Some(4326)),
            LineString::new(Some(4326))
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .unwrap()
                .to_owned(),
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(1, found_samples.len());
    for gs in found_samples {
        assert_eq!("topo_rel_test".to_string(), gs.name);
    }
}

#[test]
fn intersects_geography_test() {
    let mut conn = initialize();
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_intersects::<Geography, Point, LineString<Point>>(
            Point::new(0.0, 0.0, Some(4326)),
            LineString::new(Some(4326))
                .add_point(Point::new(2.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .unwrap()
                .to_owned(),
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(0, found_samples.len());
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_intersects::<Geography, Point, LineString<Point>>(
            Point::new(0.0, 0.0, Some(4326)),
            LineString::new(Some(4326))
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .unwrap()
                .to_owned(),
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(1, found_samples.len());
    for gs in found_samples {
        assert_eq!("topo_rel_test".to_string(), gs.name);
    }
}

#[test]
fn line_crossing_direction_test() {
    let mut conn = initialize();
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(
            1.into_sql::<diesel::sql_types::Integer>()
                .eq(st_line_crossing_direction::<
                    LineString<Point>,
                    LineString<Point>,
                >(
                    LineString::new(Some(4326))
                        .add_point(Point::new(25.0, 169.0, Some(4326)))
                        .unwrap()
                        .add_point(Point::new(89.0, 114.0, Some(4326)))
                        .unwrap()
                        .add_point(Point::new(40.0, 70.0, Some(4326)))
                        .unwrap()
                        .add_point(Point::new(86.0, 43.0, Some(4326)))
                        .unwrap()
                        .to_owned(),
                    LineString::new(Some(4326))
                        .add_point(Point::new(20.0, 140.0, Some(4326)))
                        .unwrap()
                        .add_point(Point::new(71.0, 74.0, Some(4326)))
                        .unwrap()
                        .add_point(Point::new(161.0, 53.0, Some(4326)))
                        .unwrap()
                        .to_owned(),
                )),
        )
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(0, found_samples.len());
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(
            (-1).into_sql::<diesel::sql_types::Integer>()
                .eq(st_line_crossing_direction::<
                    LineString<Point>,
                    LineString<Point>,
                >(
                    LineString::new(Some(4326))
                        .add_point(Point::new(25.0, 169.0, Some(4326)))
                        .unwrap()
                        .add_point(Point::new(89.0, 114.0, Some(4326)))
                        .unwrap()
                        .add_point(Point::new(40.0, 70.0, Some(4326)))
                        .unwrap()
                        .add_point(Point::new(86.0, 43.0, Some(4326)))
                        .unwrap()
                        .to_owned(),
                    LineString::new(Some(4326))
                        .add_point(Point::new(20.0, 140.0, Some(4326)))
                        .unwrap()
                        .add_point(Point::new(71.0, 74.0, Some(4326)))
                        .unwrap()
                        .add_point(Point::new(161.0, 53.0, Some(4326)))
                        .unwrap()
                        .to_owned(),
                )),
        )
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(1, found_samples.len());
    for gs in found_samples {
        assert_eq!("topo_rel_test".to_string(), gs.name);
    }
}

#[test]
fn ordering_equals_test() {
    let mut conn = initialize();
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_ordering_equals::<LineString<Point>, LineString<Point>>(
            LineString::new(Some(4326))
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .unwrap()
                .to_owned(),
            LineString::new(Some(4326))
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 1.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .unwrap()
                .to_owned(),
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(0, found_samples.len());
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_ordering_equals::<LineString<Point>, LineString<Point>>(
            LineString::new(Some(4326))
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .unwrap()
                .to_owned(),
            LineString::new(Some(4326))
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .unwrap()
                .to_owned(),
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(1, found_samples.len());
    for gs in found_samples {
        assert_eq!("topo_rel_test".to_string(), gs.name);
    }
}

#[test]
fn overlaps_test() {
    let mut conn = initialize();
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_overlaps::<Polygon<Point>, LineString<Point>>(
            Polygon::new(Some(4326))
                .add_point(Point::new(40.0, 170.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(90.0, 30.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(180.0, 100.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(40.0, 170.0, Some(4326)))
                .unwrap()
                .to_owned(),
            LineString::new(Some(4326))
                .add_point(Point::new(10.0, 10.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(190.0, 190.0, Some(4326)))
                .unwrap()
                .to_owned(),
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(0, found_samples.len());
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_overlaps::<Polygon<Point>, Polygon<Point>>(
            Polygon::new(Some(4326))
                .add_point(Point::new(40.0, 170.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(90.0, 30.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(180.0, 100.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(40.0, 170.0, Some(4326)))
                .unwrap()
                .to_owned(),
            //110 180, 20 60, 130 90, 110 180
            Polygon::new(Some(4326))
                .add_point(Point::new(110.0, 180.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(20.0, 60.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(130.0, 90.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(110.0, 180.0, Some(4326)))
                .unwrap()
                .to_owned(),
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(1, found_samples.len());
    for gs in found_samples {
        assert_eq!("topo_rel_test".to_string(), gs.name);
    }
}

#[test]
fn relate_check_test() {
    let mut conn = initialize();
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_relate_check(
            LineString::new(Some(4326))
                .add_point(Point::new(1.0, 2.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(3.0, 4.0, Some(4326)))
                .unwrap()
                .to_owned(),
            LineString::new(Some(4326))
                .add_point(Point::new(5.0, 6.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(7.0, 8.0, Some(4326)))
                .unwrap()
                .to_owned(),
            "FF1FF0101",
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(0, found_samples.len());
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_relate_check(
            LineString::new(Some(4326))
                .add_point(Point::new(1.0, 2.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(3.0, 4.0, Some(4326)))
                .unwrap()
                .to_owned(),
            LineString::new(Some(4326))
                .add_point(Point::new(5.0, 6.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(7.0, 8.0, Some(4326)))
                .unwrap()
                .to_owned(),
            "FF1FF0102",
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(1, found_samples.len());
    for gs in found_samples {
        assert_eq!("topo_rel_test".to_string(), gs.name);
    }
}

#[test]
fn relate_test() {
    let mut conn = initialize();
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(
            "FF1FF0101".into_sql::<Text>().eq(st_relate(
                LineString::new(Some(4326))
                    .add_point(Point::new(1.0, 2.0, Some(4326)))
                    .unwrap()
                    .add_point(Point::new(3.0, 4.0, Some(4326)))
                    .unwrap()
                    .to_owned(),
                LineString::new(Some(4326))
                    .add_point(Point::new(5.0, 6.0, Some(4326)))
                    .unwrap()
                    .add_point(Point::new(7.0, 8.0, Some(4326)))
                    .unwrap()
                    .to_owned(),
            )),
        )
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(0, found_samples.len());
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(
            "FF1FF0102".into_sql::<Text>().eq(st_relate(
                LineString::new(Some(4326))
                    .add_point(Point::new(1.0, 2.0, Some(4326)))
                    .unwrap()
                    .add_point(Point::new(3.0, 4.0, Some(4326)))
                    .unwrap()
                    .to_owned(),
                LineString::new(Some(4326))
                    .add_point(Point::new(5.0, 6.0, Some(4326)))
                    .unwrap()
                    .add_point(Point::new(7.0, 8.0, Some(4326)))
                    .unwrap()
                    .to_owned(),
            )),
        )
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(1, found_samples.len());
    for gs in found_samples {
        assert_eq!("topo_rel_test".to_string(), gs.name);
    }
}

#[test]
fn relate_match_test() {
    let mut conn = initialize();
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_relate_match("101202FFF", "TTTTTTFFF"))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(1, found_samples.len());
    for gs in found_samples {
        assert_eq!("topo_rel_test".to_string(), gs.name);
    }
}

#[test]
fn touches_test() {
    let mut conn = initialize();
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_touches(
            LineString::new(Some(4326))
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(1.0, 1.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .unwrap()
                .to_owned(),
            Point::new(1.0, 1.0, Some(4326)),
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(0, found_samples.len());
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_touches(
            LineString::new(Some(4326))
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(1.0, 1.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .unwrap()
                .to_owned(),
            Point::new(0.0, 2.0, Some(4326)),
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(1, found_samples.len());
    for gs in found_samples {
        assert_eq!("topo_rel_test".to_string(), gs.name);
    }
}

#[test]
fn within_test() {
    let mut conn = initialize();
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_within(
            Point::new(3.0, 1.0, Some(4326)),
            Polygon::new(Some(4326))
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(2.0, 2.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(2.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .to_owned(),
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(0, found_samples.len());
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_within(
            Point::new(1.0, 1.0, Some(4326)),
            Polygon::new(Some(4326))
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(2.0, 2.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(2.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .to_owned(),
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(1, found_samples.len());
    for gs in found_samples {
        assert_eq!("topo_rel_test".to_string(), gs.name);
    }
}

#[test]
fn d_within_test() {
    let mut conn = initialize();
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_d_within::<Geometry, _, _, _>(
            Point::new(3.0, 3.0, Some(4326)),
            Point::new(4.0, 4.0, Some(4326)),
            1.0,
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(0, found_samples.len());
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_d_within::<Geometry, _, _, _>(
            Point::new(3.0, 3.0, Some(4326)),
            Point::new(4.0, 4.0, Some(4326)),
            2.0,
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(1, found_samples.len());
    for gs in found_samples {
        assert_eq!("topo_rel_test".to_string(), gs.name);
    }
}
