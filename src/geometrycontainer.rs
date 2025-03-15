use std::fmt::Debug;
use std::io::Cursor;

use crate::{
    ewkb::{EwkbSerializable, GeometryType, BIG_ENDIAN},
    polygon::*,
    sql_types::{Geography, Geometry},
    types::*,
};

#[cfg(feature = "diesel")]
use crate::{
    ewkb::read_ewkb_header,
    geometrycollection::{read_geometry_collection_body, write_geometry_collection},
    linestring::{read_linestring_body, write_linestring},
    multiline::{read_multiline_body, write_multiline},
    multipoint::{read_multi_point_body, write_multi_point},
    multipolygon::{read_multi_polygon_body, write_multi_polygon},
    points::{read_point_coordinates, write_point},
};

use byteorder::{BigEndian, LittleEndian};

impl<T> GeometryContainer<T>
where
    T: PointT + Clone,
{
    pub fn dimension(&self) -> u32 {
        match self {
            GeometryContainer::Point(g) => g.dimension(),
            GeometryContainer::LineString(g) => g.dimension(),
            GeometryContainer::Polygon(g) => g.dimension(),
            GeometryContainer::MultiPoint(g) => g.dimension(),
            GeometryContainer::MultiLineString(g) => g.dimension(),
            GeometryContainer::MultiPolygon(g) => g.dimension(),
            GeometryContainer::GeometryCollection(g) => g.dimension(),
        }
    }
}

#[cfg(feature = "diesel")]
impl<T> diesel::serialize::ToSql<Geometry, diesel::pg::Pg> for GeometryContainer<T>
where
    T: PointT
        + Debug
        + PartialEq
        + Clone
        + EwkbSerializable
        + diesel::serialize::ToSql<Geometry, diesel::pg::Pg>,
{
    fn to_sql(
        &self,
        out: &mut diesel::serialize::Output<diesel::pg::Pg>,
    ) -> diesel::serialize::Result {
        match self {
            GeometryContainer::Point(g) => write_point(g, g.get_srid(), out),
            GeometryContainer::MultiPoint(g) => write_multi_point(g, g.srid, out),
            GeometryContainer::LineString(g) => write_linestring(g, g.srid, out),
            GeometryContainer::MultiLineString(g) => write_multiline(g, g.srid, out),
            GeometryContainer::Polygon(g) => write_polygon(g, g.srid, out),
            GeometryContainer::MultiPolygon(g) => write_multi_polygon(g, g.srid, out),
            GeometryContainer::GeometryCollection(g) => write_geometry_collection(g, g.srid, out),
        }
    }
}

#[cfg(feature = "diesel")]
impl<T> diesel::serialize::ToSql<Geography, diesel::pg::Pg> for GeometryContainer<T>
where
    T: PointT
        + Debug
        + PartialEq
        + Clone
        + EwkbSerializable
        + diesel::serialize::ToSql<Geometry, diesel::pg::Pg>,
{
    fn to_sql(
        &self,
        out: &mut diesel::serialize::Output<diesel::pg::Pg>,
    ) -> diesel::serialize::Result {
        match self {
            GeometryContainer::Point(g) => write_point(g, g.get_srid(), out),
            GeometryContainer::MultiPoint(g) => write_multi_point(g, g.srid, out),
            GeometryContainer::LineString(g) => write_linestring(g, g.srid, out),
            GeometryContainer::MultiLineString(g) => write_multiline(g, g.srid, out),
            GeometryContainer::Polygon(g) => write_polygon(g, g.srid, out),
            GeometryContainer::MultiPolygon(g) => write_multi_polygon(g, g.srid, out),
            GeometryContainer::GeometryCollection(g) => write_geometry_collection(g, g.srid, out),
        }
    }
}

#[cfg(feature = "diesel")]
impl<T> diesel::deserialize::FromSql<Geometry, diesel::pg::Pg> for GeometryContainer<T>
where
    T: PointT + Debug + Clone + diesel::deserialize::FromSql<Geometry, diesel::pg::Pg>,
{
    fn from_sql(bytes: diesel::pg::PgValue) -> diesel::deserialize::Result<Self> {
        use byteorder::ReadBytesExt;

        let mut cursor = Cursor::new(bytes.as_bytes());
        let end = cursor.read_u8()?;
        let container = if end == BIG_ENDIAN {
            let g_header = read_ewkb_header::<BigEndian>(&mut cursor)?;
            match GeometryType::from(g_header.g_type) {
                GeometryType::Point => {
                    GeometryContainer::Point(read_point_coordinates::<BigEndian, T>(
                        &mut cursor,
                        g_header.g_type,
                        g_header.srid,
                    )?)
                }
                GeometryType::MultiPoint => {
                    GeometryContainer::MultiPoint(read_multi_point_body::<BigEndian, T>(
                        g_header.g_type,
                        g_header.srid,
                        &mut cursor,
                    )?)
                }
                GeometryType::LineString => {
                    GeometryContainer::LineString(read_linestring_body::<BigEndian, T>(
                        g_header.g_type,
                        g_header.srid,
                        &mut cursor,
                    )?)
                }
                GeometryType::MultiLineString => {
                    GeometryContainer::MultiLineString(read_multiline_body::<BigEndian, T>(
                        g_header.g_type,
                        g_header.srid,
                        &mut cursor,
                    )?)
                }
                GeometryType::Polygon => {
                    GeometryContainer::Polygon(read_polygon_body::<BigEndian, T>(
                        g_header.g_type,
                        g_header.srid,
                        &mut cursor,
                    )?)
                }
                GeometryType::MultiPolygon => {
                    GeometryContainer::MultiPolygon(read_multi_polygon_body::<BigEndian, T>(
                        g_header.g_type,
                        g_header.srid,
                        &mut cursor,
                    )?)
                }
                GeometryType::GeometryCollection => {
                    GeometryContainer::GeometryCollection(read_geometry_collection_body::<
                        BigEndian,
                        T,
                    >(
                        g_header.g_type,
                        g_header.srid,
                        &mut cursor,
                    )?)
                }
            }
        } else {
            let g_header = read_ewkb_header::<LittleEndian>(&mut cursor)?;
            match GeometryType::from(g_header.g_type) {
                GeometryType::Point => {
                    GeometryContainer::Point(read_point_coordinates::<LittleEndian, T>(
                        &mut cursor,
                        g_header.g_type,
                        g_header.srid,
                    )?)
                }
                GeometryType::MultiPoint => {
                    GeometryContainer::MultiPoint(read_multi_point_body::<LittleEndian, T>(
                        g_header.g_type,
                        g_header.srid,
                        &mut cursor,
                    )?)
                }
                GeometryType::LineString => {
                    GeometryContainer::LineString(read_linestring_body::<LittleEndian, T>(
                        g_header.g_type,
                        g_header.srid,
                        &mut cursor,
                    )?)
                }
                GeometryType::MultiLineString => {
                    GeometryContainer::MultiLineString(read_multiline_body::<LittleEndian, T>(
                        g_header.g_type,
                        g_header.srid,
                        &mut cursor,
                    )?)
                }
                GeometryType::Polygon => {
                    GeometryContainer::Polygon(read_polygon_body::<LittleEndian, T>(
                        g_header.g_type,
                        g_header.srid,
                        &mut cursor,
                    )?)
                }
                GeometryType::MultiPolygon => {
                    GeometryContainer::MultiPolygon(read_multi_polygon_body::<LittleEndian, T>(
                        g_header.g_type,
                        g_header.srid,
                        &mut cursor,
                    )?)
                }
                GeometryType::GeometryCollection => {
                    GeometryContainer::GeometryCollection(read_geometry_collection_body::<
                        LittleEndian,
                        T,
                    >(
                        g_header.g_type,
                        g_header.srid,
                        &mut cursor,
                    )?)
                }
            }
        };
        Ok(container)
    }
}

#[cfg(feature = "diesel")]
impl<T> diesel::deserialize::FromSql<Geography, diesel::pg::Pg> for GeometryContainer<T>
where
    T: PointT + Debug + Clone + diesel::deserialize::FromSql<Geometry, diesel::pg::Pg>,
{
    fn from_sql(bytes: diesel::pg::PgValue) -> diesel::deserialize::Result<Self> {
        diesel::deserialize::FromSql::<Geometry, diesel::pg::Pg>::from_sql(bytes)
    }
}
