#[macro_use]
extern crate diesel;

use std::env;

// use diesel::query_dsl::filter_dsl::FilterDsl;
use diesel::{Connection, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use dotenv::dotenv;
use postgis::ewkb::{LineStringT, Point, PolygonT};

use postgis_diesel::*;

#[derive(Insertable)]
#[table_name = "geometry_samples"]
struct NewGeometrySample {
    point: PointC<Point>,
    linestring: LineStringC<LineStringT<Point>>,
}

#[derive(Queryable)]
struct GeometrySample {
    id: i32,
    point: PointC<Point>,
    linestring: LineStringC<LineStringT<Point>>,
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
    let _ = diesel::sql_query("CREATE EXTENSION IF NOT EXISTS postgis").execute(&conn);
    let _ = diesel::sql_query("DROP TABLE geometry_samples").execute(&conn);
    let _ = diesel::sql_query(
        "CREATE TABLE geometry_samples
(
    id         SERIAL PRIMARY KEY,
    point      geometry(Point,4326) NOT NULL,
    linestring geometry(Linestring,4326) NOT NULL
)",
    )
    .execute(&conn);
    let mut ls = LineStringT::new();
    ls.points.push(Point::new(72.0, 64.0, Option::Some(4326)));
    ls.points.push(Point::new(73.0, 64.0, Option::Some(4326)));
    ls.srid = Option::Some(4326);
    let sample = NewGeometrySample {
        point: PointC {
            v: Point::new(72.0, 64.0, Option::Some(4326)),
        },
        linestring: LineStringC { v: ls },
    };
    let point_from_db: GeometrySample = diesel::insert_into(geometry_samples::table)
        .values(&sample)
        .get_result(&conn)
        .expect("Error saving geometry sample");

    assert_eq!(sample.point, point_from_db.point);
    assert_eq!(sample.linestring, point_from_db.linestring);

    let pol = PolygonC {
        v: PolygonT {
            rings: vec![
                LineStringT {
                    points: vec![
                        Point::new(71.0, 63.0, Option::Some(4326)),
                        Point::new(71.0, 65.0, Option::Some(4326)),
                        Point::new(73.0, 65.0, Option::Some(4326)),
                        Point::new(73.0, 63.0, Option::Some(4326)),
                        Point::new(71.0, 63.0, Option::Some(4326)),
                    ],
                    srid: Option::Some(4326),
                },
            ],
            srid: Option::Some(4326),
        },
    };
    let r:GeometrySample = geometry_samples::table
        .filter(contained_by(geometry_samples::point, pol))
        .get_result::<GeometrySample>(&conn)
        .expect("Error getting geometry sample");

    assert_eq!(sample.point, r.point);
    assert_eq!(sample.linestring, r.linestring);
    let _ =
        diesel::delete(geometry_samples::table.filter(geometry_samples::id.eq(point_from_db.id)))
            .execute(&conn);
}
