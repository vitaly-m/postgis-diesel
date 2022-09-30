#[macro_use]
extern crate diesel;

use std::env;
use std::sync::Once;

// use diesel::query_dsl::filter_dsl::FilterDsl;
use diesel::pg::PgConnection;
use diesel::Connection;
use diesel::{ExpressionMethods, QueryDsl, QueryResult, RunQueryDsl};
use dotenv::dotenv;
use postgis::ewkb::{self, LineStringT};

use postgis_diesel::operators::*;
use postgis_diesel::types::{Point, PointM, PointZ, PointZM};
use postgis_diesel::*;

static INIT: Once = Once::new();

#[derive(Insertable)]
#[table_name = "geometry_samples"]
struct NewGeometrySample {
    name: String,
    point: Point,
    point_z: PointZ,
    point_m: PointM,
    point_zm: PointZM,
    linestring: LineStringC<LineStringT<ewkb::Point>>,
}

#[derive(Queryable, Debug, PartialEq)]
struct GeometrySample {
    id: i32,
    name: String,
    point: Point,
    point_z: PointZ,
    point_m: PointM,
    point_zm: PointZM,
    linestring: LineStringC<LineStringT<ewkb::Point>>,
}

table! {
    use postgis_diesel::sql_types::*;
    use diesel::sql_types::*;
    geometry_samples (id) {
        id -> Int4,
        name -> Text,
        point -> Geometry,
        point_z -> Geometry,
        point_m -> Geometry,
        point_zm -> Geometry,
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
        let _ = diesel::sql_query("DROP TABLE geometry_samples").execute(&mut conn);
        let _ = diesel::sql_query(
            "CREATE TABLE geometry_samples
(
    id         SERIAL PRIMARY KEY,
    name       text,
    point      geometry(Point,4326) NOT NULL,
    point_z    geometry(PointZ,4326) NOT NULL,
    point_m    geometry(PointM,4326) NOT NULL,
    point_zm   geometry(PointZM,4326) NOT NULL,
    linestring geometry(Linestring,4326) NOT NULL
)",
        )
        .execute(&mut conn);
    });
    conn
}

fn new_line(points: Vec<(f64, f64)>) -> LineStringC<LineStringT<ewkb::Point>> {
    let mut ls = LineStringT::new();
    for p in points {
        ls.points
            .push(ewkb::Point::new(p.0, p.1, Option::Some(4326)));
    }
    ls.srid = Option::Some(4326);
    LineStringC { v: ls }
}

fn new_point(x: f64, y: f64) -> Point {
    Point {
        x,
        y,
        srid: Some(4326),
    }
}

fn new_point_z(x: f64, y: f64, z: f64) -> PointZ {
    PointZ {
        x,
        y,
        z,
        srid: Some(4326),
    }
}

fn new_point_m(x: f64, y: f64, m: f64) -> PointM {
    PointM {
        x,
        y,
        m,
        srid: Some(4326),
    }
}

fn new_point_zm(x: f64, y: f64, z: f64, m: f64) -> PointZM {
    PointZM {
        x,
        y,
        z,
        m,
        srid: Some(4326),
    }
}

#[test]
fn smoke_test() {
    let mut conn = initialize();
    let sample = NewGeometrySample {
        name: String::from("smoke_test"),
        point: new_point(72.0, 64.0),
        point_z: new_point_z(72.0, 64.0, 10.0),
        point_m: new_point_m(72.0, 64.0, 11.0),
        point_zm: new_point_zm(72.0, 64.0, 10.0, 11.0),
        linestring: new_line(vec![(72.0, 64.0), (73.0, 64.0)]),
    };
    let point_from_db: GeometrySample = diesel::insert_into(geometry_samples::table)
        .values(&sample)
        .get_result(&mut conn)
        .expect("Error saving geometry sample");

    assert_eq!(sample.name, point_from_db.name);
    assert_eq!(sample.point, point_from_db.point);
    assert_eq!(sample.linestring, point_from_db.linestring);

    let _ =
        diesel::delete(geometry_samples::table.filter(geometry_samples::id.eq(point_from_db.id)))
            .execute(&mut conn);
}

macro_rules! operator_test {
    ($t:ident; $f:ident; $find:expr; $not_find:expr) => {
        #[test]
        fn $t() {
            let mut conn = initialize();
            let sample = NewGeometrySample {
                name: String::from(stringify!($t)),
                point: new_point(71.0, 63.0),
                point_z: new_point_z(72.0, 64.0, 10.0),
                point_m: new_point_m(72.0, 64.0, 11.0),
                point_zm: new_point_zm(72.0, 64.0, 10.0, 11.0),
                linestring: new_line(vec![(72.0, 64.0), (73.0, 64.0)]),
            };
            let _ = diesel::insert_into(geometry_samples::table)
                .values(&sample)
                .get_result::<GeometrySample>(&mut conn)
                .expect("Error saving geometry sample");
            let found = geometry_samples::table
                .filter($f(geometry_samples::linestring, $find))
                .filter(geometry_samples::name.eq(stringify!($t)))
                .get_result::<GeometrySample>(&mut conn)
                .expect("Error getting geometry");

            assert_eq!(sample.point, found.point);
            assert_eq!(sample.linestring, found.linestring);

            let not_found: QueryResult<GeometrySample> = geometry_samples::table
                .filter($f(geometry_samples::linestring, $not_find))
                .filter(geometry_samples::name.eq(stringify!($t)))
                .get_result(&mut conn);
            assert_eq!(not_found, Err(diesel::result::Error::NotFound));
        }
    };
}

// line (72.0, 64.0) --> (73.0, 64.0)
operator_test!(intersects_2d_test; intersects_2d; new_line(vec![(72.0, 63.0), (72.0, 65.0)]); new_line(vec![(71.0, 63.0), (71.0, 65.0)]));
operator_test!(overlap_or_left_test; overlaps_or_left; new_line(vec![(74.0, 63.0), (74.0, 65.0)]); new_line(vec![(71.0, 63.0), (71.0, 65.0)]));
operator_test!(overlap_or_left_overlaps_test; overlaps_or_left; new_line(vec![(72.5, 64.0), (74.0, 64.0)]); new_line(vec![(71.0, 63.0), (71.0, 65.0)]));
operator_test!(overlap_or_below_test; overlaps_or_below; new_line(vec![(72.0, 65.0), (73.0, 65.0)]); new_line(vec![(71.0, 62.0), (71.0, 62.0)]));
operator_test!(overlap_or_below_overlaps_test; overlaps_or_below; new_line(vec![(72.5, 64.0), (74.0, 64.0)]); new_line(vec![(71.0, 62.0), (71.0, 62.0)]));
operator_test!(overlap_or_right_test; overlaps_or_right; new_line(vec![(70.0, 64.0), (71.0, 64.0)]); new_line(vec![(74.0, 62.0), (75.0, 62.0)]));
operator_test!(overlap_or_right_overlaps_test; overlaps_or_right; new_line(vec![(71.0, 64.0), (73.0, 64.0)]); new_line(vec![(74.0, 62.0), (75.0, 62.0)]));
operator_test!(stricly_left_test; strictly_left; new_line(vec![(74.0, 63.0), (74.0, 65.0)]); new_line(vec![(71.0, 63.0), (71.0, 65.0)]));
operator_test!(stricly_below_test; strictly_below; new_line(vec![(72.0, 65.0), (73.0, 65.0)]); new_line(vec![(71.0, 62.0), (71.0, 62.0)]));
operator_test!(g_same_test; g_same; new_line(vec![(72.0, 64.0), (73.0, 64.0)]); new_line(vec![(73.0, 64.0), (72.0, 64.0)]));
operator_test!(strictly_right_test; strictly_right; new_line(vec![(70.0, 64.0), (71.0, 64.0)]); new_line(vec![(74.0, 62.0), (75.0, 62.0)]));
operator_test!(contained_by_test; contained_by; new_line(vec![(71.0, 64.0), (74.0, 64.0)]); new_line(vec![(74.0, 62.0), (75.0, 62.0)]));
operator_test!(overlap_or_above_test; overlaps_or_above; new_line(vec![(72.0, 63.0), (73.0, 63.0)]); new_line(vec![(71.0, 65.0), (71.0, 65.0)]));
operator_test!(overlap_or_above_overlaps_test; overlaps_or_above; new_line(vec![(72.5, 64.0), (74.0, 64.0)]); new_line(vec![(71.0, 65.0), (71.0, 65.0)]));
operator_test!(strictly_above_test; strictly_above; new_line(vec![(72.0, 63.0), (73.0, 63.0)]); new_line(vec![(71.0, 65.0), (71.0, 65.0)]));
operator_test!(contains_test; contains; new_line(vec![(72.1, 64.0), (72.9, 64.0)]); new_line(vec![(71.0, 64.0), (75.0, 64.0)]));
operator_test!(bb_same_test; bb_same; new_line(vec![(73.0, 64.0), (72.0, 64.0)]); new_line(vec![(71.0, 64.0), (75.0, 64.0)]));
