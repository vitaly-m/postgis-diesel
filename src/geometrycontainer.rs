use std::fmt::Debug;
use std::io::Cursor;

use crate::{
    ewkb::{read_ewkb_header, EwkbSerializable, GeometryType, BIG_ENDIAN},
    geometrycollection::{read_geometry_collection_body, write_geometry_collection},
    linestring::{read_linestring_body, write_linestring},
    multiline::{read_multiline_body, write_multiline},
    multipoint::{read_multi_point_body, write_multi_point},
    multipolygon::{read_multi_polygon_body, write_multi_polygon},
    points::{read_point_coordinates, write_point},
    polygon::*,
    sql_types::{Geography, Geometry},
    types::*,
};
use byteorder::{BigEndian, LittleEndian};
use diesel::{
    deserialize::{self, FromSql},
    pg::{self, Pg},
    serialize::{self, Output, ToSql},
};

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

impl<T> ToSql<Geometry, Pg> for GeometryContainer<T>
where
    T: PointT + Debug + PartialEq + Clone + EwkbSerializable + ToSql<Geometry, Pg>,
{
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
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

impl<T> ToSql<Geography, Pg> for GeometryContainer<T>
where
    T: PointT + Debug + PartialEq + Clone + EwkbSerializable + ToSql<Geometry, Pg>,
{
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
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

impl<T> FromSql<Geometry, Pg> for GeometryContainer<T>
where
    T: PointT + Debug + Clone + FromSql<Geometry, Pg>,
{
    fn from_sql(bytes: pg::PgValue) -> deserialize::Result<Self> {
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

impl<T> FromSql<Geography, Pg> for GeometryContainer<T>
where
    T: PointT + Debug + Clone + FromSql<Geometry, Pg>,
{
    fn from_sql(bytes: pg::PgValue) -> deserialize::Result<Self> {
        FromSql::<Geometry, Pg>::from_sql(bytes)
    }
}
