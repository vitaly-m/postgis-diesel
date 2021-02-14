#[macro_use]
extern crate diesel;

use std::io::prelude::*;
use std::io::Cursor;

use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, IsNull, Output, ToSql};
use postgis::ewkb::{AsEwkbPoint, EwkbRead, EwkbWrite, AsEwkbLineString, AsEwkbGeometry};
use postgis::*;

use sql_types::*;

pub mod sql_types;

#[derive(Debug, Clone, PartialEq, FromSqlRow, AsExpression)]
#[sql_type = "Geometry"]
pub enum GeometryHolder {
    Point(Point),
    LineString(LineString),
}

// #[derive(Debug, Clone, FromSqlRow, AsExpression)]
// #[sql_type = "Geometry"]
// pub struct GeometryHolder1 {
//     pub geom: ewkb::Geometry
// }
//
// impl FromSql<Geometry, Pg> for GeometryHolder1 {
//     fn from_sql(bytes: Option<&<Pg as Backend>::RawValue>) -> deserialize::Result<Self> {
//         let bytes = not_none!(bytes);
//         let mut r = Cursor::new(bytes);
//         let geom = ewkb::GeometryT::read_ewkb(&mut r)?;
//         return Ok(GeometryHolder1{geom});
//         // Err(format!("Unrecognized or unsupported geometry").into())
//     }
// }
//
// impl ToSql<Geometry, Pg> for GeometryHolder1 {
//     fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
//         self.geom.as_ewkb().write_ewkb(out)?;
//         Ok(IsNull::No)
//     }
// }

impl FromSql<Geometry, Pg> for GeometryHolder {
    fn from_sql(bytes: Option<&<Pg as Backend>::RawValue>) -> deserialize::Result<Self> {
        let bytes = not_none!(bytes);
        let mut r = Cursor::new(bytes);
        let geom = ewkb::GeometryT::read_ewkb(&mut r)?;
        return match geom {
            postgis::ewkb::GeometryT::Point(ref p) => Ok(GeometryHolder::Point(From::from(p))),
            postgis::ewkb::GeometryT::LineString(ref ls) => {
                Ok(GeometryHolder::LineString(From::from(ls)))
            }
            _ => Err(format!("Unrecognized or unsupported geometry").into()),
        };
    }
}



impl ToSql<Geometry, Pg> for GeometryHolder {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        match self {
            GeometryHolder::Point(p) => ewkb::Point::from(p).as_ewkb().write_ewkb(out)?,
            GeometryHolder::LineString(ls) => ewkb::LineString::from(ls).as_ewkb().write_ewkb(out)?,
        };
        Ok(IsNull::No)
    }
}

impl From<&ewkb::Point> for Point {
    fn from(p: &ewkb::Point) -> Self {
        Point {
            x: p.x,
            y: p.y,
            srid: p.srid,
        }
    }
}

impl From<&Point> for ewkb::Point {
    fn from(p: &Point) -> Self {
        ewkb::Point {
            x: p.x,
            y: p.y,
            srid: p.srid,
        }
    }
}

impl From<&ewkb::LineString> for LineString {
    fn from(ls: &ewkb::LineString) -> Self {
        LineString {
            points: ls.points.iter().map(|p| From::from(p)).collect(),
            srid: ls.srid,
        }
    }
}

impl From<&LineString> for ewkb::LineString {
    fn from(ls: &LineString) -> Self {
        ewkb::LineString {
            points: ls.points.iter().map(|p| From::from(p)).collect(),
            srid: ls.srid,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub srid: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LineString {
    pub points: Vec<Point>,
    pub srid: Option<i32>,
}
