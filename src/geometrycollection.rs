use std::fmt::Debug;
use std::io::Cursor;

use crate::{
    ewkb::{read_ewkb_header, write_ewkb_header, EwkbSerializable, GeometryType, BIG_ENDIAN},
    linestring::{read_linestring_body, write_linestring},
    multiline::{read_multiline_body, write_multiline},
    multipoint::{read_multi_point_body, write_multi_point},
    multipolygon::{read_multi_polygon_body, write_multi_polygon},
    points::{read_point_coordinates, write_point, Dimension},
    polygon::*, types::*,
};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use diesel::{
    deserialize::{self, FromSql},
    pg::{self, Pg},
    serialize::{self, IsNull, Output, ToSql},
};

use crate::sql_types::*;

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

impl<T> GeometryCollection<T>
where
    T: PointT + Clone,
{
    pub fn new(srid: Option<u32>) -> Self {
        Self {
            geometries: Vec::new(),
            srid,
        }
    }

    pub fn dimension(&self) -> u32 {
        let mut dimension = Dimension::None as u32;
        if let Some(geometry) = self.geometries.first() {
            dimension |= geometry.dimension();
        }
        dimension
    }
}

impl<T> EwkbSerializable for GeometryCollection<T>
where
    T: PointT + Clone,
{
    fn geometry_type(&self) -> u32 {
        let mut g_type = GeometryType::GeometryCollection as u32;
        if let Some(polygon) = self.geometries.first() {
            g_type |= polygon.dimension();
        }
        g_type
    }
}

impl<T> ToSql<Geometry, Pg> for GeometryCollection<T>
where
    T: PointT + Debug + PartialEq + Clone + EwkbSerializable,
{
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
        write_geometry_collection(self, self.srid, out)
    }
}

impl<T> FromSql<Geometry, Pg> for GeometryCollection<T>
where
    T: PointT + Debug + Clone,
{
    fn from_sql(bytes: pg::PgValue) -> deserialize::Result<Self> {
        let mut r = Cursor::new(bytes.as_bytes());
        let end = r.read_u8()?;
        if end == BIG_ENDIAN {
            read_geometry_collection::<BigEndian, T>(&mut r)
        } else {
            read_geometry_collection::<LittleEndian, T>(&mut r)
        }
    }
}

pub fn write_geometry_collection<T>(
    geometrycollection: &GeometryCollection<T>,
    srid: Option<u32>,
    out: &mut Output<Pg>,
) -> serialize::Result
where
    T: PointT + EwkbSerializable + Clone,
{
    write_ewkb_header(geometrycollection, srid, out)?;
    out.write_u32::<LittleEndian>(geometrycollection.geometries.len() as u32)?;
    for g_container in geometrycollection.geometries.iter() {
        match g_container {
            GeometryContainer::Point(g) => write_point(g, None, out)?,
            GeometryContainer::LineString(g) => write_linestring(g, None, out)?,
            GeometryContainer::Polygon(g) => write_polygon(g, None, out)?,
            GeometryContainer::MultiPoint(g) => write_multi_point(g, None, out)?,
            GeometryContainer::MultiLineString(g) => write_multiline(g, None, out)?,
            GeometryContainer::MultiPolygon(g) => write_multi_polygon(g, None, out)?,
            GeometryContainer::GeometryCollection(g) => write_geometry_collection(g, None, out)?,
        };
    }
    Ok(IsNull::No)
}

fn read_geometry_collection<T, P>(
    cursor: &mut Cursor<&[u8]>,
) -> deserialize::Result<GeometryCollection<P>>
where
    T: byteorder::ByteOrder,
    P: PointT + Clone,
{
    let g_header = read_ewkb_header::<T>(GeometryType::GeometryCollection, cursor)?;
    read_geometry_collection_body::<T, P>(g_header.g_type, g_header.srid, cursor)
}

pub fn read_geometry_collection_body<T, P>(
    g_type: u32,
    srid: Option<u32>,
    cursor: &mut Cursor<&[u8]>,
) -> deserialize::Result<GeometryCollection<P>>
where
    T: byteorder::ByteOrder,
    P: PointT + Clone,
{
    let geometries_n = cursor.read_u32::<T>()?;
    let mut g_collection = GeometryCollection::new(srid);
    for _i in 0..geometries_n {
        // skip 1 byte for byte order and 4 bytes for point type
        cursor.read_u8()?;
        let geom_type = GeometryType::from(cursor.read_u32::<T>()?);
        let g_container = match geom_type {
            GeometryType::Point => GeometryContainer::Point(read_point_coordinates::<T, P>(
                cursor,
                g_type,
                srid,
            )?),
            GeometryType::LineString => GeometryContainer::LineString(
                read_linestring_body::<T, P>(g_type, srid, cursor)?,
            ),
            GeometryType::Polygon => {
                GeometryContainer::Polygon(read_polygon_body::<T, P>(g_type, srid, cursor)?)
            }
            GeometryType::MultiPoint => GeometryContainer::MultiPoint(
                read_multi_point_body::<T, P>(g_type, srid, cursor)?,
            ),
            GeometryType::MultiLineString => GeometryContainer::MultiLineString(
                read_multiline_body::<T, P>(g_type, srid, cursor)?,
            ),
            GeometryType::MultiPolygon => GeometryContainer::MultiPolygon(
                read_multi_polygon_body::<T, P>(g_type, srid, cursor)?,
            ),
            GeometryType::GeometryCollection => GeometryContainer::GeometryCollection(
                read_geometry_collection_body::<T, P>(g_type, srid, cursor)?,
            ),
        };
        g_collection.geometries.push(g_container);
    }
    Ok(g_collection)
}
