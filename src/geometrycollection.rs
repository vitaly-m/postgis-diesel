use std::fmt::Debug;
use std::io::Cursor;

use crate::{
    ewkb::{EwkbSerializable, GeometryType, BIG_ENDIAN},
    points::Dimension,
    polygon::*,
    types::*,
};

#[cfg(feature = "diesel")]
use crate::{
    ewkb::{read_ewkb_header, write_ewkb_header},
    linestring::{read_linestring_body, write_linestring},
    multiline::{read_multiline_body, write_multiline},
    multipoint::{read_multi_point_body, write_multi_point},
    multipolygon::{read_multi_polygon_body, write_multi_polygon},
    points::{read_point_coordinates, write_point},
};

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::sql_types::*;

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

    pub fn add_geometry(&mut self, geometry: GeometryContainer<T>) -> &mut Self {
        self.geometries.push(geometry);
        self
    }

    pub fn add_geometries(
        &mut self,
        geometries: impl IntoIterator<Item = GeometryContainer<T>>,
    ) -> &mut Self {
        for gc in geometries {
            self.geometries.push(gc);
        }
        self
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

#[cfg(feature = "diesel")]
impl<T> diesel::serialize::ToSql<Geometry, diesel::pg::Pg> for GeometryCollection<T>
where
    T: PointT + Debug + PartialEq + Clone + EwkbSerializable,
{
    fn to_sql(
        &self,
        out: &mut diesel::serialize::Output<diesel::pg::Pg>,
    ) -> diesel::serialize::Result {
        write_geometry_collection(self, self.srid, out)
    }
}

#[cfg(feature = "diesel")]
impl<T> diesel::serialize::ToSql<Geography, diesel::pg::Pg> for GeometryCollection<T>
where
    T: PointT + Debug + PartialEq + Clone + EwkbSerializable,
{
    fn to_sql(
        &self,
        out: &mut diesel::serialize::Output<diesel::pg::Pg>,
    ) -> diesel::serialize::Result {
        write_geometry_collection(self, self.srid, out)
    }
}

#[cfg(feature = "diesel")]
impl<T> diesel::deserialize::FromSql<Geometry, diesel::pg::Pg> for GeometryCollection<T>
where
    T: PointT + Debug + Clone,
{
    fn from_sql(bytes: diesel::pg::PgValue) -> diesel::deserialize::Result<Self> {
        let mut r = Cursor::new(bytes.as_bytes());
        let end = r.read_u8()?;
        if end == BIG_ENDIAN {
            read_geometry_collection::<BigEndian, T>(&mut r)
        } else {
            read_geometry_collection::<LittleEndian, T>(&mut r)
        }
    }
}

#[cfg(feature = "diesel")]
impl<T> diesel::deserialize::FromSql<Geography, diesel::pg::Pg> for GeometryCollection<T>
where
    T: PointT + Debug + Clone,
{
    fn from_sql(bytes: diesel::pg::PgValue) -> diesel::deserialize::Result<Self> {
        diesel::deserialize::FromSql::<Geometry, diesel::pg::Pg>::from_sql(bytes)
    }
}

#[cfg(feature = "diesel")]
pub fn write_geometry_collection<T>(
    geometrycollection: &GeometryCollection<T>,
    srid: Option<u32>,
    out: &mut diesel::serialize::Output<diesel::pg::Pg>,
) -> diesel::serialize::Result
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
    Ok(diesel::serialize::IsNull::No)
}

#[cfg(feature = "diesel")]
fn read_geometry_collection<T, P>(
    cursor: &mut Cursor<&[u8]>,
) -> diesel::deserialize::Result<GeometryCollection<P>>
where
    T: byteorder::ByteOrder,
    P: PointT + Clone,
{
    let g_header = read_ewkb_header::<T>(cursor)?.expect(GeometryType::GeometryCollection)?;
    read_geometry_collection_body::<T, P>(g_header.g_type, g_header.srid, cursor)
}

#[cfg(feature = "diesel")]
pub fn read_geometry_collection_body<T, P>(
    g_type: u32,
    srid: Option<u32>,
    cursor: &mut Cursor<&[u8]>,
) -> diesel::deserialize::Result<GeometryCollection<P>>
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
            GeometryType::Point => {
                GeometryContainer::Point(read_point_coordinates::<T, P>(cursor, g_type, srid)?)
            }
            GeometryType::LineString => {
                GeometryContainer::LineString(read_linestring_body::<T, P>(g_type, srid, cursor)?)
            }
            GeometryType::Polygon => {
                GeometryContainer::Polygon(read_polygon_body::<T, P>(g_type, srid, cursor)?)
            }
            GeometryType::MultiPoint => {
                GeometryContainer::MultiPoint(read_multi_point_body::<T, P>(g_type, srid, cursor)?)
            }
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

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_dimensions_point() {
        assert_eq!(
            Dimension::None as u32,
            GeometryContainer::Point(Point::new(0.0, 0.0, None)).dimension()
        );
        assert_eq!(
            Dimension::Z as u32,
            GeometryContainer::Point(PointZ::new(0.0, 0.0, 0.0, None)).dimension()
        );
        assert_eq!(
            Dimension::M as u32,
            GeometryContainer::Point(PointM::new(0.0, 0.0, 0.0, None)).dimension()
        );
        assert_eq!(
            Dimension::ZM as u32,
            GeometryContainer::Point(PointZM::new(0.0, 0.0, 0.0, 0.0, None)).dimension()
        );
    }

    #[test]
    fn test_dimensions_line_string() {
        assert_eq!(
            Dimension::None as u32,
            GeometryContainer::LineString(
                LineString::new(None)
                    .add_point(Point::new(0.0, 0.0, None))
                    .to_owned()
            )
            .dimension()
        );
        assert_eq!(
            Dimension::Z as u32,
            GeometryContainer::LineString(
                LineString::new(None)
                    .add_point(PointZ::new(0.0, 0.0, 0.0, None))
                    .to_owned()
            )
            .dimension()
        );
        assert_eq!(
            Dimension::M as u32,
            GeometryContainer::LineString(
                LineString::new(None)
                    .add_point(PointM::new(0.0, 0.0, 0.0, None))
                    .to_owned()
            )
            .dimension()
        );
        assert_eq!(
            Dimension::ZM as u32,
            GeometryContainer::LineString(
                LineString::new(None)
                    .add_point(PointZM::new(0.0, 0.0, 0.0, 0.0, None))
                    .to_owned()
            )
            .dimension()
        );
    }

    #[test]
    fn test_dimensions_polygon() {
        assert_eq!(
            Dimension::None as u32,
            GeometryContainer::Polygon(
                Polygon::new(None)
                    .add_point(Point::new(0.0, 0.0, None))
                    .to_owned()
            )
            .dimension()
        );
        assert_eq!(
            Dimension::Z as u32,
            GeometryContainer::Polygon(
                Polygon::new(None)
                    .add_point(PointZ::new(0.0, 0.0, 0.0, None))
                    .to_owned()
            )
            .dimension()
        );
        assert_eq!(
            Dimension::M as u32,
            GeometryContainer::Polygon(
                Polygon::new(None)
                    .add_point(PointM::new(0.0, 0.0, 0.0, None))
                    .to_owned()
            )
            .dimension()
        );
        assert_eq!(
            Dimension::ZM as u32,
            GeometryContainer::Polygon(
                Polygon::new(None)
                    .add_point(PointZM::new(0.0, 0.0, 0.0, 0.0, None))
                    .to_owned()
            )
            .dimension()
        );
    }

    #[test]
    fn test_dimensions_multi_point() {
        assert_eq!(
            Dimension::None as u32,
            GeometryContainer::MultiPoint(
                MultiPoint::new(None)
                    .add_point(Point::new(0.0, 0.0, None))
                    .to_owned()
            )
            .dimension()
        );
        assert_eq!(
            Dimension::Z as u32,
            GeometryContainer::MultiPoint(
                MultiPoint::new(None)
                    .add_point(PointZ::new(0.0, 0.0, 0.0, None))
                    .to_owned()
            )
            .dimension()
        );
        assert_eq!(
            Dimension::M as u32,
            GeometryContainer::MultiPoint(
                MultiPoint::new(None)
                    .add_point(PointM::new(0.0, 0.0, 0.0, None))
                    .to_owned()
            )
            .dimension()
        );
        assert_eq!(
            Dimension::ZM as u32,
            GeometryContainer::MultiPoint(
                MultiPoint::new(None)
                    .add_point(PointZM::new(0.0, 0.0, 0.0, 0.0, None))
                    .to_owned()
            )
            .dimension()
        );
    }

    #[test]
    fn test_dimensions_multi_line_string() {
        assert_eq!(
            Dimension::None as u32,
            GeometryContainer::MultiLineString(
                MultiLineString::new(None)
                    .add_point(Point::new(0.0, 0.0, None))
                    .to_owned()
            )
            .dimension()
        );
        assert_eq!(
            Dimension::Z as u32,
            GeometryContainer::MultiLineString(
                MultiLineString::new(None)
                    .add_point(PointZ::new(0.0, 0.0, 0.0, None))
                    .to_owned()
            )
            .dimension()
        );
        assert_eq!(
            Dimension::M as u32,
            GeometryContainer::MultiLineString(
                MultiLineString::new(None)
                    .add_point(PointM::new(0.0, 0.0, 0.0, None))
                    .to_owned()
            )
            .dimension()
        );
        assert_eq!(
            Dimension::ZM as u32,
            GeometryContainer::MultiLineString(
                MultiLineString::new(None)
                    .add_point(PointZM::new(0.0, 0.0, 0.0, 0.0, None))
                    .to_owned()
            )
            .dimension()
        );
    }

    #[test]
    fn test_dimensions_multi_polygon() {
        assert_eq!(
            Dimension::None as u32,
            GeometryContainer::MultiPolygon(
                MultiPolygon::new(None)
                    .add_point(Point::new(0.0, 0.0, None))
                    .to_owned()
            )
            .dimension()
        );
        assert_eq!(
            Dimension::Z as u32,
            GeometryContainer::MultiPolygon(
                MultiPolygon::new(None)
                    .add_point(PointZ::new(0.0, 0.0, 0.0, None))
                    .to_owned()
            )
            .dimension()
        );
        assert_eq!(
            Dimension::M as u32,
            GeometryContainer::MultiPolygon(
                MultiPolygon::new(None)
                    .add_point(PointM::new(0.0, 0.0, 0.0, None))
                    .to_owned()
            )
            .dimension()
        );
        assert_eq!(
            Dimension::ZM as u32,
            GeometryContainer::MultiPolygon(
                MultiPolygon::new(None)
                    .add_point(PointZM::new(0.0, 0.0, 0.0, 0.0, None))
                    .to_owned()
            )
            .dimension()
        );
    }

    #[test]
    fn test_dimensions_geometry_collection() {
        assert_eq!(
            Dimension::None as u32,
            GeometryContainer::GeometryCollection(
                GeometryCollection::new(None)
                    .add_geometry(GeometryContainer::Point(Point::new(0.0, 0.0, None)))
                    .to_owned()
            )
            .dimension()
        );
        assert_eq!(
            Dimension::Z as u32,
            GeometryContainer::GeometryCollection(
                GeometryCollection::new(None)
                    .add_geometry(GeometryContainer::Point(PointZ::new(0.0, 0.0, 0.0, None)))
                    .to_owned()
            )
            .dimension()
        );
        assert_eq!(
            Dimension::M as u32,
            GeometryContainer::GeometryCollection(
                GeometryCollection::new(None)
                    .add_geometry(GeometryContainer::Point(PointM::new(0.0, 0.0, 0.0, None)))
                    .to_owned()
            )
            .dimension()
        );
        assert_eq!(
            Dimension::ZM as u32,
            GeometryContainer::GeometryCollection(
                GeometryCollection::new(None)
                    .add_geometry(GeometryContainer::Point(PointZM::new(
                        0.0, 0.0, 0.0, 0.0, None
                    )))
                    .to_owned()
            )
            .dimension()
        );
    }
}
