#[macro_use]
extern crate diesel;

use std::env;

use diesel::query_dsl::filter_dsl::FilterDsl;
use diesel::{Connection, ExpressionMethods, PgConnection, RunQueryDsl};
use dotenv::dotenv;

use postgis_diesel::*;

#[derive(Insertable)]
#[table_name = "geometry_samples"]
struct NewGeometrySample {
    point: postgis_diesel::GeometryHolder,
    linestring: postgis_diesel::GeometryHolder,
}

#[derive(Queryable)]
struct GeometrySample {
    id: i32,
    point: postgis_diesel::GeometryHolder,
    linestring: postgis_diesel::GeometryHolder,
}

table! {
    use postgis_diesel::sql_types::*;
    use diesel::sql_types::*;
    geometry_samples (id) {
        id -> Int4,
        point -> Geometry,
        linestring -> Geometry,
    }
}

fn establish_connection() -> PgConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");

    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

#[test]
fn geometry_test() {
    let conn = establish_connection();
    let _ = diesel::sql_query("CREATE EXTENSION postgis").execute(&conn);
    let _ = diesel::sql_query(
        "CREATE TABLE geometry_samples
(
    id         SERIAL PRIMARY KEY,
    point      geometry(Point,4326)    NOT NULL,
    linestring geometry(Linestring,4326)    NOT NULL
)",
    )
    .execute(&conn);
    let new_track = NewGeometrySample {
        point: GeometryHolder::Point(Point {
            x: 75.0,
            y: 34.0,
            srid: Option::Some(4326),
        }),
        linestring: GeometryHolder::LineString(LineString {
            points: vec![
                Point {
                    x: 75.0,
                    y: 34.0,
                    srid: Option::Some(4326),
                },
                Point {
                    x: 76.0,
                    y: 35.0,
                    srid: Option::Some(4326),
                },
            ],
            srid: Option::Some(4326),
        }),
    };
    let point_from_db: GeometrySample = diesel::insert_into(geometry_samples::table)
        .values(&new_track)
        .get_result(&conn)
        .expect("Error saving geometry sample");

    assert_eq!(new_track.point, point_from_db.point);
    assert_eq!(new_track.linestring, point_from_db.linestring);

    let _ =
        diesel::delete(geometry_samples::table.filter(geometry_samples::id.eq(point_from_db.id)))
            .execute(&conn);
}
