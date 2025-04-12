#[macro_use]
extern crate diesel;

use std::env;
use std::sync::Once;

use diesel::pg::PgConnection;
use diesel::{Connection, IntoSql};
use diesel::{QueryDsl, RunQueryDsl};
use dotenvy::dotenv;

use postgis_diesel::functions_nullable::*;
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
        .filter(
            st_3d_intersects::<Option<PointZ>, LineString<PointZ>>(
                None,
                LineString::new(Some(4326))
                    .add_point(PointZ::new(0.0, 0.0, 1.0, Some(4326)))
                    .unwrap()
                    .add_point(PointZ::new(0.0, 2.0, 3.0, Some(4326)))
                    .unwrap()
                    .to_owned(),
            )
            .eq(true),
        )
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(0, found_samples.len());
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_3d_intersects::<Option<PointZ>, LineString<PointZ>>(
            Some(PointZ::new(0.0, 0.0, 1.0, Some(4326))),
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
        .filter(st_contains::<Polygon<Point>, Option<Point>>(
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
            None,
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
            Some(Point::new(1.0, 1.0, Some(4326))),
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
        .filter(st_contains_properly::<Polygon<Point>, Option<Point>>(
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
            None,
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(0, found_samples.len());
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_contains_properly::<Option<Polygon<Point>>, Point>(
            Some(
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
            ),
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
        .filter(st_covered_by::<Geometry, Option<Point>, Polygon<Point>>(
            None,
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
        .filter(st_covered_by::<Geometry, Point, Option<Polygon<Point>>>(
            Point::new(1.0, 1.0, Some(4326)),
            Some(
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
            ),
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
        .filter(st_covered_by::<Geography, Option<Point>, Polygon<Point>>(
            None,
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
        .filter(st_covered_by::<
            Geography,
            Option<Point>,
            Option<Polygon<Point>>,
        >(
            Some(Point::new(1.0, 1.0, Some(4326))),
            Some(
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
            ),
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
        .filter(st_covers::<Geometry, Polygon<Point>, Option<Point>>(
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
            None,
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(0, found_samples.len());
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(
            st_covers::<Geometry, Option<Polygon<Point>>, Option<Point>>(
                Some(
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
                ),
                Some(Point::new(1.0, 1.0, Some(4326))),
            ),
        )
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
        .filter(st_covers::<Geography, Polygon<Point>, Option<Point>>(
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
            None,
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(0, found_samples.len());
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_covers::<Geography, Option<Polygon<Point>>, Point>(
            Some(
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
            ),
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
        .filter(st_crosses::<Option<LineString<Point>>, LineString<Point>>(
            None,
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
        .filter(st_crosses::<LineString<Point>, Option<LineString<Point>>>(
            LineString::new(Some(4326))
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .unwrap()
                .to_owned(),
            Some(
                LineString::new(Some(4326))
                    .add_point(Point::new(-1.0, 1.0, Some(4326)))
                    .unwrap()
                    .add_point(Point::new(1.0, 1.0, Some(4326)))
                    .unwrap()
                    .to_owned(),
            ),
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
        .filter(st_disjoint::<Option<Point>, LineString<Point>>(
            None,
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
        .filter(st_disjoint::<Option<Point>, Option<LineString<Point>>>(
            Some(Point::new(0.0, 0.0, Some(4326))),
            Some(
                LineString::new(Some(4326))
                    .add_point(Point::new(2.0, 0.0, Some(4326)))
                    .unwrap()
                    .add_point(Point::new(0.0, 2.0, Some(4326)))
                    .unwrap()
                    .to_owned(),
            ),
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
        .filter(st_equals::<Option<LineString<Point>>, LineString<Point>>(
            None,
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
        .filter(st_equals::<LineString<Point>, Option<LineString<Point>>>(
            LineString::new(Some(4326))
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .unwrap()
                .to_owned(),
            Some(
                LineString::new(Some(4326))
                    .add_point(Point::new(0.0, 0.0, Some(4326)))
                    .unwrap()
                    .add_point(Point::new(0.0, 1.0, Some(4326)))
                    .unwrap()
                    .add_point(Point::new(0.0, 2.0, Some(4326)))
                    .unwrap()
                    .to_owned(),
            ),
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
        .filter(st_intersects::<Geometry, Option<Point>, LineString<Point>>(
            Some(Point::new(0.0, 0.0, Some(4326))),
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
        .filter(st_intersects::<Geometry, Point, Option<LineString<Point>>>(
            Point::new(0.0, 0.0, Some(4326)),
            Some(
                LineString::new(Some(4326))
                    .add_point(Point::new(0.0, 0.0, Some(4326)))
                    .unwrap()
                    .add_point(Point::new(0.0, 2.0, Some(4326)))
                    .unwrap()
                    .to_owned(),
            ),
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
        .filter(
            st_intersects::<Geography, Option<Point>, LineString<Point>>(
                None,
                LineString::new(Some(4326))
                    .add_point(Point::new(2.0, 0.0, Some(4326)))
                    .unwrap()
                    .add_point(Point::new(0.0, 2.0, Some(4326)))
                    .unwrap()
                    .to_owned(),
            ),
        )
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(0, found_samples.len());
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(
            st_intersects::<Geography, Point, Option<LineString<Point>>>(
                Point::new(0.0, 0.0, Some(4326)),
                Some(
                    LineString::new(Some(4326))
                        .add_point(Point::new(0.0, 0.0, Some(4326)))
                        .unwrap()
                        .add_point(Point::new(0.0, 2.0, Some(4326)))
                        .unwrap()
                        .to_owned(),
                ),
            ),
        )
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
            Some(1).into_sql().eq(st_line_crossing_direction::<
                Option<LineString<Point>>,
                LineString<Point>,
            >(
                None,
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
            Some(-1).into_sql().eq(st_line_crossing_direction::<
                LineString<Point>,
                Option<LineString<Point>>,
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
                Some(
                    LineString::new(Some(4326))
                        .add_point(Point::new(20.0, 140.0, Some(4326)))
                        .unwrap()
                        .add_point(Point::new(71.0, 74.0, Some(4326)))
                        .unwrap()
                        .add_point(Point::new(161.0, 53.0, Some(4326)))
                        .unwrap()
                        .to_owned(),
                ),
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
        .filter(st_ordering_equals::<
            Option<LineString<Point>>,
            LineString<Point>,
        >(
            None,
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
        .filter(st_ordering_equals::<
            LineString<Point>,
            Option<LineString<Point>>,
        >(
            LineString::new(Some(4326))
                .add_point(Point::new(0.0, 0.0, Some(4326)))
                .unwrap()
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .unwrap()
                .to_owned(),
            Some(
                LineString::new(Some(4326))
                    .add_point(Point::new(0.0, 0.0, Some(4326)))
                    .unwrap()
                    .add_point(Point::new(0.0, 2.0, Some(4326)))
                    .unwrap()
                    .to_owned(),
            ),
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
        .filter(st_overlaps::<Option<Polygon<Point>>, LineString<Point>>(
            None,
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
        .filter(st_overlaps::<Polygon<Point>, Option<Polygon<Point>>>(
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
            Some(
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
            ),
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
        .filter(st_relate_check::<
            Option<LineString<Point>>,
            LineString<Point>,
            &'static str,
        >(
            None,
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
        .filter(st_relate_check::<
            Option<LineString<Point>>,
            Option<LineString<Point>>,
            Option<&'static str>,
        >(
            Some(
                LineString::new(Some(4326))
                    .add_point(Point::new(1.0, 2.0, Some(4326)))
                    .unwrap()
                    .add_point(Point::new(3.0, 4.0, Some(4326)))
                    .unwrap()
                    .to_owned(),
            ),
            Some(
                LineString::new(Some(4326))
                    .add_point(Point::new(5.0, 6.0, Some(4326)))
                    .unwrap()
                    .add_point(Point::new(7.0, 8.0, Some(4326)))
                    .unwrap()
                    .to_owned(),
            ),
            Some("FF1FF0102"),
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
            Some("FF1FF0101").into_sql().eq(st_relate::<
                Option<LineString<Point>>,
                LineString<Point>,
            >(
                None,
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
            Some("FF1FF0102").into_sql().eq(st_relate::<
                Option<LineString<Point>>,
                Option<LineString<Point>>,
            >(
                Some(
                    LineString::new(Some(4326))
                        .add_point(Point::new(1.0, 2.0, Some(4326)))
                        .unwrap()
                        .add_point(Point::new(3.0, 4.0, Some(4326)))
                        .unwrap()
                        .to_owned(),
                ),
                Some(
                    LineString::new(Some(4326))
                        .add_point(Point::new(5.0, 6.0, Some(4326)))
                        .unwrap()
                        .add_point(Point::new(7.0, 8.0, Some(4326)))
                        .unwrap()
                        .to_owned(),
                ),
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
        .filter(st_relate_match(Some("101202FFF"), Some("TTTTTTFFF")))
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
            Some(
                LineString::new(Some(4326))
                    .add_point(Point::new(0.0, 0.0, Some(4326)))
                    .unwrap()
                    .add_point(Point::new(1.0, 1.0, Some(4326)))
                    .unwrap()
                    .add_point(Point::new(0.0, 2.0, Some(4326)))
                    .unwrap()
                    .to_owned(),
            ),
            Point::new(1.0, 1.0, Some(4326)),
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(0, found_samples.len());
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_touches(
            Some(
                LineString::new(Some(4326))
                    .add_point(Point::new(0.0, 0.0, Some(4326)))
                    .unwrap()
                    .add_point(Point::new(1.0, 1.0, Some(4326)))
                    .unwrap()
                    .add_point(Point::new(0.0, 2.0, Some(4326)))
                    .unwrap()
                    .to_owned(),
            ),
            Some(Point::new(0.0, 2.0, Some(4326))),
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
            Some(Point::new(3.0, 1.0, Some(4326))),
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
            Some(Point::new(1.0, 1.0, Some(4326))),
            Some(
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
            ),
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
            Some(Point::new(3.0, 3.0, Some(4326))),
            Some(Point::new(4.0, 4.0, Some(4326))),
            1.0,
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(0, found_samples.len());
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_d_within::<Geometry, _, _, _>(
            Point::new(3.0, 3.0, Some(4326)),
            Point::new(4.0, 4.0, Some(4326)),
            Some(2.0),
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(1, found_samples.len());
    for gs in found_samples {
        assert_eq!("topo_rel_test".to_string(), gs.name);
    }
}
