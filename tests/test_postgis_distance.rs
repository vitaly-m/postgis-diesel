#![cfg(feature = "postgres")]
#[macro_use]
extern crate diesel;

use diesel::pg::PgConnection;
use diesel::Connection;
use diesel::{QueryDsl, RunQueryDsl};
use dotenvy::dotenv;
use std::env;
use std::sync::Once;

use postgis_diesel::operators::*;
use postgis_diesel::types::*;

static INIT: Once = Once::new();

#[derive(Insertable)]
#[diesel(table_name = distance_samples)]
struct NewDistanceSample {
    name: String,
    point: Point,
    polygon: Polygon<Point>,
}

#[derive(Queryable, Debug, PartialEq)]
#[diesel(table_name = distance_samples)]
struct DistanceSample {
    id: i32,
    name: String,
    point: Point,
    polygon: Polygon<Point>,
}

table! {
    use postgis_diesel::sql_types::*;
    use diesel::sql_types::*;
    distance_samples (id) {
        id -> Int4,
        name -> Text,
        point -> Geometry,
        polygon -> Geometry,
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
        let _ = diesel::sql_query("DROP TABLE distance_samples").execute(&mut conn);

        let _ = diesel::sql_query(
            "CREATE TABLE distance_samples
(
    id                SERIAL PRIMARY KEY,
    name              text,
    point             geometry(Point,4326) NOT NULL,
    polygon           geometry(Polygon,4326) NOT NULL
)",
        )
        .execute(&mut conn);
    });
    conn
}

#[test]
fn distance_2d_test() {
    let mut conn = initialize();
    let m_point = Point::new(37.618423, 55.751244, Some(4326));
    let v_point = Point::new(39.2088823, 51.6754966, Some(4326));
    let mut m_polygon = Polygon::new(Some(4326));
    m_polygon
        .add_ring()
        .add_point(Point::new(34.22922934177507, 57.98223781625711, Some(4326)))
        .unwrap()
        .add_point(Point::new(34.22922934177507, 57.08802327022599, Some(4326)))
        .unwrap()
        .add_point(Point::new(36.18872506469961, 57.08802327022599, Some(4326)))
        .unwrap()
        .add_point(Point::new(36.18872506469961, 57.98223781625711, Some(4326)))
        .unwrap()
        .add_point(Point::new(34.22922934177507, 57.98223781625711, Some(4326)))
        .unwrap();
    let mut v_polygon = Polygon::new(Some(4326));
    v_polygon
        .add_ring()
        .add_point(Point::new(39.732117473214146, 53.3129374517664, Some(4326)))
        .unwrap()
        .add_point(Point::new(
            39.732117473214146,
            52.658032976474146,
            Some(4326),
        ))
        .unwrap()
        .add_point(Point::new(
            40.79294127961202,
            52.658032976474146,
            Some(4326),
        ))
        .unwrap()
        .add_point(Point::new(40.79294127961202, 53.3129374517664, Some(4326)))
        .unwrap()
        .add_point(Point::new(39.732117473214146, 53.3129374517664, Some(4326)))
        .unwrap();

    let m_sample = NewDistanceSample {
        name: String::from("Moscow"),
        point: m_point,
        polygon: m_polygon,
    };
    let v_sample = NewDistanceSample {
        name: String::from("Voronezh"),
        point: v_point,
        polygon: v_polygon,
    };
    let records = vec![m_sample, v_sample];
    let r = diesel::insert_into(distance_samples)
        .values(records)
        .execute(&mut conn);
    assert!(r.is_ok(), "can't insert data");

    use self::distance_samples::dsl::*;

    let found_sample: DistanceSample = distance_samples
        .order_by(distance_2d(
            point,
            Point::new(38.495490805803115, 52.62169972015738, Some(4326)),
        ))
        .limit(1)
        .get_result(&mut conn)
        .expect("nothing found");
    assert_eq!("Voronezh", found_sample.name);
    let found_sample: DistanceSample = distance_samples
        .order_by(distance_2d(
            point,
            Point::new(37.184130751959486, 55.988642876744535, Some(4326)),
        ))
        .limit(1)
        .get_result(&mut conn)
        .expect("nothing found");
    assert_eq!("Moscow", found_sample.name);
}
