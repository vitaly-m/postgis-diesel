use std::fmt::Debug;
use std::io::Cursor;

use crate::{
    ewkb::{EwkbSerializable, GeometryType, BIG_ENDIAN},
    points::Dimension,
    polygon::*,
    types::*,
    write_to_read_from_sql::{ReadFromSql, WriteToSql},
};

#[cfg(feature = "diesel")]
use crate::{
    ewkb::{read_ewkb_header, write_ewkb_header},
    linestring::read_linestring_body,
    multiline::read_multiline_body,
    multipoint::read_multi_point_body,
    multipolygon::read_multi_polygon_body,
    points::{read_point_coordinates, write_point},
};

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

impl<P> GeometryCollection<P>
where
    P: PointT + Clone,
{
    pub fn new(srid: Option<u32>) -> Self {
        Self {
            geometries: Vec::new(),
            srid,
        }
    }

    pub fn add_geometry(&mut self, geometry: GeometryContainer<P>) -> &mut Self {
        self.geometries.push(geometry);
        self
    }

    pub fn add_geometries(
        &mut self,
        geometries: impl IntoIterator<Item = GeometryContainer<P>>,
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

impl<P> EwkbSerializable for GeometryCollection<P>
where
    P: PointT + Clone,
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
impl<P> ReadFromSql for GeometryCollection<P>
where
    P: PointT + Debug + Clone,
{
    fn read_from_sql(bytes: &[u8]) -> diesel::deserialize::Result<Self> {
        let mut r = Cursor::new(bytes);
        let end = r.read_u8()?;
        if end == BIG_ENDIAN {
            read_geometry_collection::<BigEndian, P>(&mut r)
        } else {
            read_geometry_collection::<LittleEndian, P>(&mut r)
        }
    }
}

#[cfg(feature = "diesel")]
impl<P> WriteToSql for GeometryCollection<P>
where
    P: PointT + Clone + EwkbSerializable,
{
    fn write_to_sql<W>(&self, out: &mut W) -> diesel::serialize::Result
    where
        W: std::io::Write,
    {
        write_ewkb_header(self, self.srid, out)?;
        out.write_u32::<LittleEndian>(self.geometries.len() as u32)?;
        for g_container in self.geometries.iter() {
            match g_container {
                GeometryContainer::Point(g) => write_point(g, None, out)?,
                GeometryContainer::LineString(g) => g.write_to_sql(out)?,
                GeometryContainer::Polygon(g) => g.write_to_sql(out)?,
                GeometryContainer::MultiPoint(g) => g.write_to_sql(out)?,
                GeometryContainer::MultiLineString(g) => g.write_to_sql(out)?,
                GeometryContainer::MultiPolygon(g) => g.write_to_sql(out)?,
                GeometryContainer::GeometryCollection(g) => g.write_to_sql(out)?,
            };
        }
        Ok(diesel::serialize::IsNull::No)
    }
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
