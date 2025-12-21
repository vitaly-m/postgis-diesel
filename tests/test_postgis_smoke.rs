#![cfg(feature = "postgres")]
#[macro_use]
extern crate diesel;

use diesel::pg::PgConnection;
use diesel::Connection;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use dotenvy::dotenv;
use std::env;
use std::sync::Once;

use postgis_diesel::types::*;

static INIT: Once = Once::new();

#[derive(Insertable)]
#[diesel(table_name = geometry_samples)]
struct NewGeometrySample {
    name: String,
    point: Point,
    point_z: PointZ,
    point_m: PointM,
    point_zm: PointZM,
    linestring: LineString<Point>,
    polygon: Polygon<Point>,
    multipoint: MultiPoint<Point>,
    multiline: MultiLineString<Point>,
    multipolygon: MultiPolygon<Point>,
    geometrycollection: GeometryCollection<Point>,
    geometrycontainer: GeometryContainer<Point>,
}

#[derive(Insertable)]
#[diesel(table_name = geography_samples)]
struct NewGeographySample {
    name: String,
    point: Point,
    point_z: PointZ,
    point_m: PointM,
    point_zm: PointZM,
    linestring: LineString<Point>,
    polygon: Polygon<Point>,
    multipoint: MultiPoint<Point>,
    multiline: MultiLineString<Point>,
    multipolygon: MultiPolygon<Point>,
    geometrycollection: GeometryCollection<Point>,
    geometrycontainer: GeometryContainer<Point>,
}

#[derive(Queryable, Debug, PartialEq)]
struct GeometrySample {
    id: i32,
    name: String,
    point: Point,
    point_z: PointZ,
    point_m: PointM,
    point_zm: PointZM,
    linestring: LineString<Point>,
    polygon: Polygon<Point>,
    multipoint: MultiPoint<Point>,
    multiline: MultiLineString<Point>,
    multipolygon: MultiPolygon<Point>,
    geometrycollection: GeometryCollection<Point>,
    geometrycontainer: GeometryContainer<Point>,
}

#[derive(Queryable, Debug, PartialEq)]
struct GeographySample {
    id: i32,
    name: String,
    point: Point,
    point_z: PointZ,
    point_m: PointM,
    point_zm: PointZM,
    linestring: LineString<Point>,
    polygon: Polygon<Point>,
    multipoint: MultiPoint<Point>,
    multiline: MultiLineString<Point>,
    multipolygon: MultiPolygon<Point>,
    geometrycollection: GeometryCollection<Point>,
    geometrycontainer: GeometryContainer<Point>,
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
        polygon -> Geometry,
        multipoint -> Geometry,
        multiline -> Geometry,
        multipolygon -> Geometry,
        geometrycollection -> Geometry,
        geometrycontainer -> Geometry,
    }
}

table! {
    use postgis_diesel::sql_types::*;
    use diesel::sql_types::*;
    geography_samples (id) {
        id -> Int4,
        name -> Text,
        point -> Geography,
        point_z -> Geography,
        point_m -> Geography,
        point_zm -> Geography,
        linestring -> Geography,
        polygon -> Geography,
        multipoint -> Geography,
        multiline -> Geography,
        multipolygon -> Geography,
        geometrycollection -> Geography,
        geometrycontainer -> Geography,
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
    point             geometry(Point,4326) NOT NULL,
    point_z           geometry(PointZ,4326) NOT NULL,
    point_m           geometry(PointM,4326) NOT NULL,
    point_zm          geometry(PointZM,4326) NOT NULL,
    linestring        geometry(Linestring,4326) NOT NULL,
    polygon           geometry(Polygon,4326) NOT NULL,
    multipoint        geometry(MultiPoint,4326) NOT NULL,
    multiline         geometry(MultiLineString,4326) NOT NULL,
    multipolygon      geometry(MultiPolygon,4326) NOT NULL,
    geometrycollection geometry(GeometryCollection,4326) NOT NULL,
    geometrycontainer  geometry(Geometry,4326) NOT NULL
)",
        )
        .execute(&mut conn);

        let _ = diesel::sql_query(
            "CREATE TABLE geography_samples
(
    id                SERIAL PRIMARY KEY,
    name              text,
    point             geography(Point,4326) NOT NULL,
    point_z           geography(PointZ,4326) NOT NULL,
    point_m           geography(PointM,4326) NOT NULL,
    point_zm          geography(PointZM,4326) NOT NULL,
    linestring        geography(Linestring,4326) NOT NULL,
    polygon           geography(Polygon,4326) NOT NULL,
    multipoint        geography(MultiPoint,4326) NOT NULL,
    multiline         geography(MultiLineString,4326) NOT NULL,
    multipolygon      geography(MultiPolygon,4326) NOT NULL,
    geometrycollection geography(GeometryCollection,4326) NOT NULL,
    geometrycontainer  geography(Geometry,4326) NOT NULL
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

fn new_line(points: Vec<(f64, f64)>) -> LineString<Point> {
    let points = points
        .into_iter()
        .map(|(x, y)| new_point(x, y))
        .collect::<Vec<Point>>();
    LineString {
        points,
        srid: Some(4326),
    }
}

fn new_geometry_collection() -> GeometryCollection<Point> {
    let mut gc = GeometryCollection::new(Some(4326));
    gc.add_geometry(GeometryContainer::Point(new_point(72.0, 64.0)));
    gc.add_geometry(GeometryContainer::LineString(new_line(vec![
        (72.0, 64.0),
        (73.0, 64.0),
    ])));
    gc
}

#[test]
fn smoke_test() {
    let mut conn = initialize();
    let mut polygon = Polygon::new(Some(4326));
    polygon
        .add_points([
            new_point(72.0, 64.0),
            new_point(73.0, 65.0),
            new_point(71.0, 62.0),
            new_point(72.0, 64.0),
        ])
        .unwrap();
    let mut multiline = MultiLineString::new(Some(4326));
    multiline
        .add_points([new_point(72.0, 64.0), new_point(73.0, 65.0)])
        .unwrap();
    multiline.add_line();
    multiline
        .add_points([new_point(71.0, 62.0), new_point(72.0, 64.0)])
        .unwrap();
    let mut multipolygon = MultiPolygon::new(Some(4326));
    multipolygon
        .add_empty_polygon()
        .add_points([
            new_point(72.0, 64.0),
            new_point(73.0, 65.0),
            new_point(71.0, 62.0),
            new_point(72.0, 64.0),
        ])
        .unwrap()
        .add_empty_polygon()
        .add_points([
            new_point(75.0, 64.0),
            new_point(74.0, 65.0),
            new_point(74.0, 62.0),
            new_point(75.0, 64.0),
        ])
        .unwrap();
    let sample = NewGeometrySample {
        name: String::from("smoke_test"),
        point: new_point(72.0, 64.0),
        point_z: new_point_z(72.0, 64.0, 10.0),
        point_m: new_point_m(72.0, 64.0, 11.0),
        point_zm: new_point_zm(72.0, 64.0, 10.0, 11.0),
        linestring: new_line(vec![(72.0, 64.0), (73.0, 64.0)]),
        polygon: polygon.clone(),
        multipoint: MultiPoint {
            points: vec![new_point(72.0, 64.0), new_point(73.0, 64.0)],
            srid: Some(4326),
        },
        multiline,
        multipolygon,
        geometrycollection: new_geometry_collection(),
        geometrycontainer: GeometryContainer::Polygon(polygon),
    };
    let point_from_db: GeometrySample = diesel::insert_into(geometry_samples::table)
        .values(&sample)
        .get_result(&mut conn)
        .expect("Error saving geometry sample");

    assert_eq!(sample.name, point_from_db.name);
    assert_eq!(sample.point, point_from_db.point);
    assert_eq!(sample.point_z, point_from_db.point_z);
    assert_eq!(sample.point_m, point_from_db.point_m);
    assert_eq!(sample.point_zm, point_from_db.point_zm);
    assert_eq!(sample.linestring, point_from_db.linestring);
    assert_eq!(sample.polygon, point_from_db.polygon);
    assert_eq!(sample.multipoint, point_from_db.multipoint);
    assert_eq!(sample.multiline, point_from_db.multiline);
    assert_eq!(sample.multipolygon, point_from_db.multipolygon);
    assert_eq!(sample.geometrycollection, point_from_db.geometrycollection);

    let _ =
        diesel::delete(geometry_samples::table.filter(geometry_samples::id.eq(point_from_db.id)))
            .execute(&mut conn);
}

#[test]
fn geography_smoke_test() {
    let mut conn = initialize();
    let mut polygon = Polygon::new(Some(4326));
    polygon
        .add_points([
            new_point(72.0, 64.0),
            new_point(73.0, 65.0),
            new_point(71.0, 62.0),
            new_point(72.0, 64.0),
        ])
        .unwrap();
    let mut multiline = MultiLineString::new(Some(4326));
    multiline
        .add_points([new_point(72.0, 64.0), new_point(73.0, 65.0)])
        .unwrap();
    multiline.add_line();
    multiline
        .add_points([new_point(71.0, 62.0), new_point(72.0, 64.0)])
        .unwrap();
    let mut multipolygon = MultiPolygon::new(Some(4326));
    multipolygon
        .add_empty_polygon()
        .add_points([
            new_point(72.0, 64.0),
            new_point(73.0, 65.0),
            new_point(71.0, 62.0),
            new_point(72.0, 64.0),
        ])
        .unwrap()
        .add_empty_polygon()
        .add_points([
            new_point(75.0, 64.0),
            new_point(74.0, 65.0),
            new_point(74.0, 62.0),
            new_point(75.0, 64.0),
        ])
        .unwrap();
    let sample = NewGeographySample {
        name: String::from("smoke_test"),
        point: new_point(72.0, 64.0),
        point_z: new_point_z(72.0, 64.0, 10.0),
        point_m: new_point_m(72.0, 64.0, 11.0),
        point_zm: new_point_zm(72.0, 64.0, 10.0, 11.0),
        linestring: new_line(vec![(72.0, 64.0), (73.0, 64.0)]),
        polygon: polygon.clone(),
        multipoint: MultiPoint {
            points: vec![new_point(72.0, 64.0), new_point(73.0, 64.0)],
            srid: Some(4326),
        },
        multiline,
        multipolygon,
        geometrycollection: new_geometry_collection(),
        geometrycontainer: GeometryContainer::Polygon(polygon),
    };
    let point_from_db: GeographySample = diesel::insert_into(geography_samples::table)
        .values(&sample)
        .get_result(&mut conn)
        .expect("Error saving geometry sample");

    assert_eq!(sample.name, point_from_db.name);
    assert_eq!(sample.point, point_from_db.point);
    assert_eq!(sample.point_z, point_from_db.point_z);
    assert_eq!(sample.point_m, point_from_db.point_m);
    assert_eq!(sample.point_zm, point_from_db.point_zm);
    assert_eq!(sample.linestring, point_from_db.linestring);
    assert_eq!(sample.polygon, point_from_db.polygon);
    assert_eq!(sample.multipoint, point_from_db.multipoint);
    assert_eq!(sample.multiline, point_from_db.multiline);
    assert_eq!(sample.multipolygon, point_from_db.multipolygon);
    assert_eq!(sample.geometrycollection, point_from_db.geometrycollection);

    let _ =
        diesel::delete(geometry_samples::table.filter(geometry_samples::id.eq(point_from_db.id)))
            .execute(&mut conn);
}
