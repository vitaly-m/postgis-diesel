#[macro_use]
extern crate diesel;

use std::env;
use std::sync::Once;

// use diesel::query_dsl::filter_dsl::FilterDsl;
use diesel::pg::PgConnection;
use diesel::Connection;
use diesel::{ExpressionMethods, QueryDsl, QueryResult, RunQueryDsl};
use dotenv::dotenv;

use postgis_diesel::operators::*;
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
}

#[derive(Insertable)]
#[diesel(table_name = distance_samples)]
struct NewDistanceSample {
    name: String,
    point: Point,
    polygon: Polygon<Point>,
}

#[derive(Insertable)]
#[diesel(table_name = point_samples)]
struct NewPointSample {
    name: String,
    point: Point,
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
}

#[derive(Queryable, Debug, PartialEq)]
#[diesel(table_name = distance_samples)]
struct DistanceSample {
    id: i32,
    name: String,
    point: Point,
    polygon: Polygon<Point>,
}

#[derive(Queryable, Debug, PartialEq)]
#[diesel(table_name = point_samples)]
struct PointSample {
    id: i32,
    name: String,
    point: LineString<Point>,
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
    }
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

table! {
    use postgis_diesel::sql_types::*;
    use diesel::sql_types::*;
    point_samples (id) {
        id -> Int4,
        name -> Text,
        point -> Geometry,
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
        let _ = diesel::sql_query("DROP TABLE distance_samples").execute(&mut conn);
        let _ = diesel::sql_query("DROP TABLE geography_samples").execute(&mut conn);
        let _ = diesel::sql_query("DROP TABLE point_samples").execute(&mut conn);

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
    geometrycollection geometry(GeometryCollection,4326) NOT NULL
)",
        )
        .execute(&mut conn);
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
    geometrycollection geography(GeometryCollection,4326) NOT NULL
)",
        )
        .execute(&mut conn);

        let _ = diesel::sql_query(
            "CREATE TABLE point_samples
(
    id                SERIAL PRIMARY KEY,
    name              text,
    point             geometry(Point,4326) NOT NULL
)",
        )
        .execute(&mut conn);
    });
    conn
}

fn new_line(points: Vec<(f64, f64)>) -> LineString<Point> {
    let mut l_points = Vec::with_capacity(points.len());
    for p in points {
        l_points.push(Point {
            x: p.0,
            y: p.1,
            srid: Option::Some(4326),
        });
    }
    //just to check that add_points works
    LineString::new(Option::Some(4326)).add_points(l_points).to_owned()
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

fn new_geometry_collection() -> GeometryCollection<Point> {
    let mut polygon = Polygon::new(Some(4326));
    polygon.add_points([
        new_point(72.0, 64.0),
        new_point(73.0, 65.0),
        new_point(71.0, 62.0),
        new_point(72.0, 64.0),
    ]);
    let mut multiline = MultiLineString::new(Some(4326));
    multiline.add_points([new_point(72.0, 64.0), new_point(73.0, 65.0)]);
    multiline.add_line();
    multiline.add_points([new_point(71.0, 62.0), new_point(72.0, 64.0)]);
    let mut multipolygon = MultiPolygon::new(Some(4326));
    multipolygon
        .add_empty_polygon()
        .add_points([
            new_point(72.0, 64.0),
            new_point(73.0, 65.0),
            new_point(71.0, 62.0),
            new_point(72.0, 64.0),
        ])
        .add_empty_polygon()
        .add_points([
            new_point(75.0, 64.0),
            new_point(74.0, 65.0),
            new_point(74.0, 62.0),
            new_point(75.0, 64.0),
        ]);
    let mut gc = GeometryCollection::new(Some(4326));
    gc.add_geometry(GeometryContainer::Point(new_point(73.0, 64.0)));
    gc.add_geometry(GeometryContainer::LineString(new_line(vec![
        (72.0, 64.0),
        (73.0, 64.0),
    ])));
    gc.add_geometry(GeometryContainer::Polygon(polygon));
    gc.add_geometry(GeometryContainer::MultiPoint(MultiPoint {
        points: vec![new_point(72.0, 64.0), new_point(73.0, 64.0)],
        srid: Some(4326),
    }));
    gc.add_geometries(vec![GeometryContainer::MultiLineString(multiline), GeometryContainer::MultiPolygon(multipolygon)]);
    let mut inner_gc = GeometryCollection::new(Some(4326));
    inner_gc.add_geometry(GeometryContainer::Point(new_point(74.0, 64.0)));
    gc.add_geometry(GeometryContainer::GeometryCollection(inner_gc));
    gc
}

#[test]
fn smoke_test() {
    let mut conn = initialize();
    let mut polygon = Polygon::new(Some(4326));
    polygon.add_points([
        new_point(72.0, 64.0),
        new_point(73.0, 65.0),
        new_point(71.0, 62.0),
        new_point(72.0, 64.0),
    ]);
    let mut multiline = MultiLineString::new(Some(4326));
    multiline.add_points([new_point(72.0, 64.0), new_point(73.0, 65.0)]);
    multiline.add_line();
    multiline.add_points([new_point(71.0, 62.0), new_point(72.0, 64.0)]);
    let mut multipolygon = MultiPolygon::new(Some(4326));
    multipolygon
        .add_empty_polygon()
        .add_points([
            new_point(72.0, 64.0),
            new_point(73.0, 65.0),
            new_point(71.0, 62.0),
            new_point(72.0, 64.0),
        ])
        .add_empty_polygon()
        .add_points([
            new_point(75.0, 64.0),
            new_point(74.0, 65.0),
            new_point(74.0, 62.0),
            new_point(75.0, 64.0),
        ]);
    let sample = NewGeometrySample {
        name: String::from("smoke_test"),
        point: new_point(72.0, 64.0),
        point_z: new_point_z(72.0, 64.0, 10.0),
        point_m: new_point_m(72.0, 64.0, 11.0),
        point_zm: new_point_zm(72.0, 64.0, 10.0, 11.0),
        linestring: new_line(vec![(72.0, 64.0), (73.0, 64.0)]),
        polygon: polygon,
        multipoint: MultiPoint {
            points: vec![new_point(72.0, 64.0), new_point(73.0, 64.0)],
            srid: Some(4326),
        },
        multiline: multiline,
        multipolygon: multipolygon,
        geometrycollection: new_geometry_collection(),
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
    polygon.add_points([
        new_point(72.0, 64.0),
        new_point(73.0, 65.0),
        new_point(71.0, 62.0),
        new_point(72.0, 64.0),
    ]);
    let mut multiline = MultiLineString::new(Some(4326));
    multiline.add_points([new_point(72.0, 64.0), new_point(73.0, 65.0)]);
    multiline.add_line();
    multiline.add_points([new_point(71.0, 62.0), new_point(72.0, 64.0)]);
    let mut multipolygon = MultiPolygon::new(Some(4326));
    multipolygon
        .add_empty_polygon()
        .add_points([
            new_point(72.0, 64.0),
            new_point(73.0, 65.0),
            new_point(71.0, 62.0),
            new_point(72.0, 64.0),
        ])
        .add_empty_polygon()
        .add_points([
            new_point(75.0, 64.0),
            new_point(74.0, 65.0),
            new_point(74.0, 62.0),
            new_point(75.0, 64.0),
        ]);
    let sample = NewGeographySample {
        name: String::from("smoke_test"),
        point: new_point(72.0, 64.0),
        point_z: new_point_z(72.0, 64.0, 10.0),
        point_m: new_point_m(72.0, 64.0, 11.0),
        point_zm: new_point_zm(72.0, 64.0, 10.0, 11.0),
        linestring: new_line(vec![(72.0, 64.0), (73.0, 64.0)]),
        polygon: polygon,
        multipoint: MultiPoint {
            points: vec![new_point(72.0, 64.0), new_point(73.0, 64.0)],
            srid: Some(4326),
        },
        multiline: multiline,
        multipolygon: multipolygon,
        geometrycollection: new_geometry_collection(),
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

#[test]
fn distance_2d_test() {
    let mut conn = initialize();
    let m_point = Point::new(37.618423, 55.751244, Some(4326));
    let v_point = Point::new(39.2088823, 51.6754966, Some(4326));
    let mut m_polygon = Polygon::new(Some(4326));
    m_polygon
        .add_ring()
        .add_point(Point::new(34.22922934177507, 57.98223781625711, Some(4326)))
        .add_point(Point::new(34.22922934177507, 57.08802327022599, Some(4326)))
        .add_point(Point::new(36.18872506469961, 57.08802327022599, Some(4326)))
        .add_point(Point::new(36.18872506469961, 57.98223781625711, Some(4326)))
        .add_point(Point::new(34.22922934177507, 57.98223781625711, Some(4326)));
    let mut v_polygon = Polygon::new(Some(4326));
    v_polygon
        .add_ring()
        .add_point(Point::new(39.732117473214146, 53.3129374517664, Some(4326)))
        .add_point(Point::new(
            39.732117473214146,
            52.658032976474146,
            Some(4326),
        ))
        .add_point(Point::new(
            40.79294127961202,
            52.658032976474146,
            Some(4326),
        ))
        .add_point(Point::new(40.79294127961202, 53.3129374517664, Some(4326)))
        .add_point(Point::new(39.732117473214146, 53.3129374517664, Some(4326)));

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
    assert_eq!(true, r.is_ok(), "can't insert data");

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

#[test]
#[should_panic(expected = "Geometry Point is not a LineString")]
fn unmatched_types_test() {
    let mut conn = initialize();

    let sample = NewPointSample {
        name: String::from("should fail"),
        point: new_point(72.0, 64.0),
    };
    let _point_from_db: PointSample = diesel::insert_into(point_samples::table)
        .values(&sample)
        .get_result(&mut conn)
        .unwrap();
}

macro_rules! operator_test {
    ($t:ident; $f:ident; $find:expr; $not_find:expr) => {
        #[test]
        fn $t() {
            let mut conn = initialize();
            let mut polygon = Polygon::new(Some(4326));
            polygon.add_points([
                new_point(72.0, 64.0),
                new_point(73.0, 65.0),
                new_point(71.0, 62.0),
                new_point(72.0, 64.0),
            ]);
            let mut multiline = MultiLineString::new(Some(4326));
            multiline.add_points([new_point(72.0, 64.0), new_point(73.0, 65.0)]);
            multiline.add_line();
            multiline.add_points([new_point(71.0, 62.0), new_point(72.0, 64.0)]);
            let mut multipolygon = MultiPolygon::new(Some(4326));
            multipolygon
                .add_empty_polygon()
                .add_points([
                    new_point(72.0, 64.0),
                    new_point(73.0, 65.0),
                    new_point(71.0, 62.0),
                    new_point(72.0, 64.0),
                ])
                .add_empty_polygon()
                .add_points([
                    new_point(75.0, 64.0),
                    new_point(74.0, 65.0),
                    new_point(74.0, 62.0),
                    new_point(75.0, 64.0),
                ]);
            let sample = NewGeometrySample {
                name: String::from(stringify!($t)),
                point: new_point(71.0, 63.0),
                point_z: new_point_z(72.0, 64.0, 10.0),
                point_m: new_point_m(72.0, 64.0, 11.0),
                point_zm: new_point_zm(72.0, 64.0, 10.0, 11.0),
                linestring: new_line(vec![(72.0, 64.0), (73.0, 64.0)]),
                polygon: polygon,
                multipoint: MultiPoint {
                    points: vec![new_point(72.0, 64.0), new_point(73.0, 64.0)],
                    srid: Some(4326),
                },
                multiline: multiline,
                multipolygon: multipolygon,
                geometrycollection: new_geometry_collection(),
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
