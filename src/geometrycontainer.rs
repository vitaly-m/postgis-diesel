use std::fmt::Debug;
use std::io::Cursor;

use crate::{
    ewkb::{EwkbSerializable, GeometryType, BIG_ENDIAN},
    polygon::*,
    types::*,
};

#[cfg(feature = "diesel")]
use crate::{
    ewkb::read_ewkb_header,
    geometrycollection::read_geometry_collection_body,
    linestring::read_linestring_body,
    multiline::read_multiline_body,
    multipoint::read_multi_point_body,
    multipolygon::read_multi_polygon_body,
    points::{read_point_coordinates, write_point},
    write_to_read_from_sql::{ReadFromSql, WriteToSql},
};

use byteorder::{BigEndian, LittleEndian};

impl<P> GeometryContainer<P>
where
    P: PointT + Clone,
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
impl<P> WriteToSql for GeometryContainer<P>
where
    P: PointT + Clone + EwkbSerializable,
{
    fn write_to_sql<W>(&self, out: &mut W) -> diesel::serialize::Result
    where
        W: std::io::Write,
    {
        match self {
            GeometryContainer::Point(g) => write_point(g, g.get_srid(), out),
            GeometryContainer::MultiPoint(g) => g.write_to_sql(out),
            GeometryContainer::LineString(g) => g.write_to_sql(out),
            GeometryContainer::MultiLineString(g) => g.write_to_sql(out),
            GeometryContainer::Polygon(g) => g.write_to_sql(out),
            GeometryContainer::MultiPolygon(g) => g.write_to_sql(out),
            GeometryContainer::GeometryCollection(g) => g.write_to_sql(out),
        }
    }
}

#[cfg(feature = "diesel")]
fn from_sql_with_endianness<E, P>(
    cursor: &mut Cursor<&[u8]>,
) -> diesel::deserialize::Result<GeometryContainer<P>>
where
    E: byteorder::ByteOrder,
    P: PointT + Clone,
{
    let g_header = read_ewkb_header::<E>(cursor)?;
    Ok(match GeometryType::from(g_header.g_type) {
        GeometryType::Point => GeometryContainer::Point(read_point_coordinates::<E, P>(
            cursor,
            g_header.g_type,
            g_header.srid,
        )?),
        GeometryType::MultiPoint => GeometryContainer::MultiPoint(read_multi_point_body::<E, P>(
            g_header.g_type,
            g_header.srid,
            cursor,
        )?),
        GeometryType::LineString => GeometryContainer::LineString(read_linestring_body::<E, P>(
            g_header.g_type,
            g_header.srid,
            cursor,
        )?),
        GeometryType::MultiLineString => {
            GeometryContainer::MultiLineString(read_multiline_body::<E, P>(
                g_header.g_type,
                g_header.srid,
                cursor,
            )?)
        }
        GeometryType::Polygon => GeometryContainer::Polygon(read_polygon_body::<E, P>(
            g_header.g_type,
            g_header.srid,
            cursor,
        )?),
        GeometryType::MultiPolygon => {
            GeometryContainer::MultiPolygon(read_multi_polygon_body::<E, P>(
                g_header.g_type,
                g_header.srid,
                cursor,
            )?)
        }
        GeometryType::GeometryCollection => GeometryContainer::GeometryCollection(
            read_geometry_collection_body::<E, P>(g_header.g_type, g_header.srid, cursor)?,
        ),
    })
}

#[cfg(feature = "diesel")]
impl<P> ReadFromSql for GeometryContainer<P>
where
    P: PointT + Debug + Clone,
{
    fn read_from_sql(bytes: &[u8]) -> diesel::deserialize::Result<Self> {
        use byteorder::ReadBytesExt;
        let mut cursor = Cursor::new(bytes);
        let end: u8 = cursor.read_u8()?;
        if end == BIG_ENDIAN {
            from_sql_with_endianness::<BigEndian, P>(&mut cursor)
        } else {
            from_sql_with_endianness::<LittleEndian, P>(&mut cursor)
        }
    }
}
