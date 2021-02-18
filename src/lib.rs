#[macro_use]
extern crate diesel;

use std::fmt::Debug;
use std::io::Cursor;

use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, IsNull, Output, ToSql};
use postgis::ewkb::{AsEwkbLineString, AsEwkbPoint, EwkbRead, EwkbWrite};
use postgis::*;

use sql_types::*;

pub mod sql_types;

#[derive(Copy, Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[sql_type = "Geometry"]
pub struct PointC<T> {
    pub v: T,
}

#[derive(Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[sql_type = "Geometry"]
pub struct LineStringC<T> {
    pub v: T,
}

macro_rules! impl_from_sql {
    ($p:ident, $ps:literal, $s:ident) => {
        fn from_sql(bytes: Option<&<Pg as Backend>::RawValue>) -> deserialize::Result<Self> {
            let bytes = not_none!(bytes);
            let mut r = Cursor::new(bytes);
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
        fn to_sql<W: std::io::Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
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

impl<P> FromSql<Geometry, Pg> for LineStringC<ewkb::LineStringT<P>>
where
    P: postgis::Point + EwkbRead + Debug,
{
    impl_from_sql!(LineString, "LineString", LineStringC);
}

impl<P> ToSql<Geometry, Pg> for LineStringC<ewkb::LineStringT<P>>
where
    P: postgis::Point + for<'a> AsEwkbPoint<'a> + EwkbRead + Debug,
{
    impl_to_sql!();
}
