#[macro_use]
extern crate diesel;

use std::env;
use std::sync::Once;

// use diesel::query_dsl::filter_dsl::FilterDsl;
use diesel::pg::PgConnection;
use diesel::Connection;
use diesel::{QueryDsl, RunQueryDsl};
use dotenv::dotenv;

use postgis_diesel::functions::{st_3d_intersects, st_contains};
use postgis_diesel::types::*;

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
            .add_point(PointZ::new(0.0, 2.0, 3.0, Some(4326)))
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
            .add_point(PointZ::new(0.0, 2.0, 3.0, Some(4326)))
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
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .add_point(Point::new(2.0, 2.0, Some(4326)))
                .add_point(Point::new(2.0, 0.0, Some(4326)))
                .add_point(Point::new(0.0, 0.0, Some(4326)))
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
                .add_point(Point::new(0.0, 2.0, Some(4326)))
                .add_point(Point::new(2.0, 2.0, Some(4326)))
                .add_point(Point::new(2.0, 0.0, Some(4326)))
                .add_point(Point::new(0.0, 0.0, Some(4326)))
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
