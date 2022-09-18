#[macro_use]
extern crate diesel;

use std::fmt::Debug;
use std::io::Cursor;

use diesel::deserialize::{self, FromSql};
use diesel::pg;
use diesel::pg::Pg;
use diesel::serialize::{self, IsNull, Output, ToSql};
use postgis::ewkb::{
    AsEwkbGeometryCollection, AsEwkbLineString, AsEwkbMultiPolygon, AsEwkbPoint, AsEwkbPolygon,
    EwkbRead, EwkbWrite,
};
use postgis::*;

use sql_types::*;

pub mod operators;
pub mod sql_types;

/// Container for a `postgis::ewkb::Point`, use that structure in `Insertable` or `Queryable` struct.
/// ```
/// #[macro_use] extern crate diesel;
/// use postgis_diesel::PointC;
/// use postgis::ewkb::Point;
/// #[derive(Queryable)]
/// struct PointExample {
///     id: i32,
///     point: PointC<Point>,
/// }
/// ```
#[derive(Copy, Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
pub struct PointC<T> {
    pub v: T,
}

/// Container for a `postgis::ewkb::LineStringT`, use that structure in `Insertable` or `Queryable` struct.
/// ```
/// #[macro_use] extern crate diesel;
/// use postgis_diesel::LineStringC;
/// use postgis::ewkb::{Point, LineStringT};
/// #[derive(Queryable)]
/// struct PointExample {
///     id: i32,
///     point: LineStringC<LineStringT<Point>>,
/// }
/// ```
#[derive(Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
pub struct LineStringC<T> {
    pub v: T,
}

/// Container for a `postgis::ewkb::PolygonT`, use that structure in `Insertable` or `Queryable` struct.
/// ```
/// #[macro_use] extern crate diesel;
/// use postgis_diesel::PolygonC;
/// use postgis::ewkb::{Point, PolygonT};
/// #[derive(Queryable)]
/// struct PointExample {
///     id: i32,
///     point: PolygonC<PolygonT<Point>>,
/// }
/// ```
#[derive(Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
pub struct PolygonC<T> {
    pub v: T,
}

/// Container for a `postgis::ewkb::MultiPointT`, use that structure in `Insertable` or `Queryable` struct.
/// ```
/// #[macro_use] extern crate diesel;
/// use postgis_diesel::MultiPointC;
/// use postgis::ewkb::{Point, MultiPointT};
/// #[derive(Queryable)]
/// struct PointExample {
///     id: i32,
///     point: MultiPointC<MultiPointT<Point>>,
/// }
/// ```
#[derive(Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
pub struct MultiPointC<T> {
    pub v: T,
}

/// Container for a `postgis::ewkb::MultiLineStringT`, use that structure in `Insertable` or `Queryable` struct.
/// ```
/// #[macro_use] extern crate diesel;
/// use postgis_diesel::MultiLineStringC;
/// use postgis::ewkb::{Point, MultiLineStringT};
/// #[derive(Queryable)]
/// struct PointExample {
///     id: i32,
///     point: MultiLineStringC<MultiLineStringT<Point>>,
/// }
/// ```
#[derive(Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
pub struct MultiLineStringC<T> {
    pub v: T,
}

/// Container for a `postgis::ewkb::MultiPolygonT`, use that structure in `Insertable` or `Queryable` struct.
/// ```
/// #[macro_use] extern crate diesel;
/// use postgis_diesel::MultiPolygonC;
/// use postgis::ewkb::{Point, MultiPolygonT};
/// #[derive(Queryable)]
/// struct PointExample {
///     id: i32,
///     point: MultiPolygonC<MultiPolygonT<Point>>,
/// }
/// ```
#[derive(Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
pub struct MultiPolygonC<T> {
    pub v: T,
}

/// Container for a `postgis::ewkb::GeometryCollectionT`, use that structure in `Insertable` or `Queryable` struct.
/// ```
/// #[macro_use] extern crate diesel;
/// use postgis_diesel::GeometryCollectionC;
/// use postgis::ewkb::{Point, GeometryCollectionT};
/// #[derive(Queryable)]
/// struct PointExample {
///     id: i32,
///     point: GeometryCollectionC<GeometryCollectionT<Point>>,
/// }
/// ```
#[derive(Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
pub struct GeometryCollectionC<T> {
    pub v: T,
}

macro_rules! impl_from_sql {
    ($p:ident, $ps:literal, $s:ident) => {
        fn from_sql(bytes: pg::PgValue) -> deserialize::Result<Self> {
            let mut r = Cursor::new(bytes.as_bytes());
            let geom = ewkb::GeometryT::read_ewkb(&mut r)?;
            return match geom {
                postgis::ewkb::GeometryT::$p(v) => Ok($s { v }),
                _ => Err(format!("Geometry is not a {}", $ps).into()),
            };
        }
    };
}

macro_rules! impl_to_sql {
    () => {
        fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
            self.v.as_ewkb().write_ewkb(out)?;
            Ok(IsNull::No)
        }
    };
}

impl<P> FromSql<Geometry, Pg> for PointC<P>
where
    P: postgis::Point + EwkbRead + Debug,
{
    impl_from_sql!(Point, "Point", PointC);
}

impl<P> ToSql<Geometry, Pg> for PointC<P>
where
    P: postgis::Point + for<'a> AsEwkbPoint<'a> + Debug,
{
    impl_to_sql!();
}

macro_rules! impl_from_sql_trait {
    ($p:ident, $pb:ident, $ps:literal, $s:ident) => {
        impl<P> FromSql<Geometry, Pg> for $s<ewkb::$pb<P>>
        where
            P: postgis::Point + EwkbRead + Debug,
        {
            impl_from_sql!($p, $ps, $s);
        }
    };
}

macro_rules! impl_to_sql_trait {
    ($pb:ident, $s:ident) => {
        impl<P> ToSql<Geometry, Pg> for $s<ewkb::$pb<P>>
        where
            P: postgis::Point + for<'a> AsEwkbPoint<'a> + EwkbRead + Debug,
        {
            impl_to_sql!();
        }
    };
}

impl_to_sql_trait!(PolygonT, PolygonC);
impl_to_sql_trait!(LineStringT, LineStringC);
impl_to_sql_trait!(MultiPolygonT, MultiPolygonC);
impl_to_sql_trait!(GeometryCollectionT, GeometryCollectionC);

impl_from_sql_trait!(LineString, LineStringT, "LineString", LineStringC);
impl_from_sql_trait!(Polygon, PolygonT, "Polygon", PolygonC);
impl_from_sql_trait!(MultiPoint, MultiPointT, "MultiPoint", MultiPointC);
impl_from_sql_trait!(
    MultiLineString,
    MultiLineStringT,
    "MultiLineString",
    MultiLineStringC
);
impl_from_sql_trait!(MultiPolygon, MultiPolygonT, "MultiPolygon", MultiPolygonC);
impl_from_sql_trait!(
    GeometryCollection,
    GeometryCollectionT,
    "GeometryCollection",
    GeometryCollectionC
);
