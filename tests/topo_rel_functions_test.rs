#[macro_use]
extern crate diesel;

use std::env;
use std::sync::Once;

// use diesel::query_dsl::filter_dsl::FilterDsl;
use diesel::pg::PgConnection;
use diesel::Connection;
use diesel::{QueryDsl, RunQueryDsl};
use dotenv::dotenv;

use postgis_diesel::functions::st_3d_intersects;
use postgis_diesel::types::*;

static INIT: Once = Once::new();

#[derive(Insertable)]
#[diesel(table_name = topo_rel_functions)]
struct NewGeometrySample {
    name: String,
    point_z: PointZ,
    linestring: LineString<PointZ>,
}

#[derive(Queryable)]
#[diesel(table_name = topo_rel_functions)]
#[allow(dead_code)]
struct GeometrySample {
    id: i32,
    name: String,
    point_z: PointZ,
    linestring: LineString<PointZ>,
}

table! {
    use postgis_diesel::sql_types::*;
    use diesel::sql_types::*;
    topo_rel_functions (id) {
        id -> Int4,
        name -> Text,
        point_z -> Geometry,
        linestring -> Geometry,
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
    name              text,
    point_z           geometry(PointZ,4326) NOT NULL,
    linestring        geometry(LinestringZ,4326) NOT NULL
)",
        )
        .execute(&mut conn);
    });
    conn
}

#[test]
fn intersect_3d_test() {
    let mut conn = initialize();
    let sample = NewGeometrySample {
        name: "3d_intersects".to_string(),
        point_z: PointZ::new(0.0, 0.0, 2.0, Some(4326)),
        linestring: LineString::new(Some(4326))
            .add_point(PointZ::new(0.0, 0.0, 1.0, Some(4326)))
            .add_point(PointZ::new(0.0, 2.0, 3.0, Some(4326)))
            .to_owned(),
    };
    diesel::insert_into(topo_rel_functions::table)
        .values(&sample)
        .execute(&mut conn)
        .expect("Can't insert 3d_intersect sample");
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_3d_intersects(
            topo_rel_functions::point_z,
            topo_rel_functions::linestring,
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(0, found_samples.len());
    let found_samples: Vec<GeometrySample> = topo_rel_functions::table
        .filter(st_3d_intersects(
            PointZ::new(0.0, 0.0, 1.0, Some(4326)),
            topo_rel_functions::linestring,
        ))
        .get_results(&mut conn)
        .unwrap();
    assert_eq!(1, found_samples.len());
    for gs in found_samples {
        assert_eq!("3d_intersects".to_string(), gs.name);
        assert_eq!(sample.point_z, gs.point_z);
        assert_eq!(sample.linestring, gs.linestring);
    }
}
