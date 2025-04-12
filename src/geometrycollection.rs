use crate::{
    ewkb::{EwkbSerializable, GeometryType},
    points::Dimension,
    types::*,
};

use crate::write_to_read_from_sql::{ReadFromSql, WriteToSql};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

impl<P> GeometryCollection<P>
where
    P: PointT,
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
    P: PointT,
{
    fn expected_geometry_variant(_: u32) -> GeometryType {
        GeometryType::GeometryCollection
    }

    fn geometry_type(&self) -> u32 {
        let mut g_type = GeometryType::GeometryCollection as u32;
        if let Some(polygon) = self.geometries.first() {
            g_type |= polygon.dimension();
        }
        g_type
    }

    fn srid(&self) -> Option<u32> {
        self.srid
    }
}

impl<P> ReadFromSql for GeometryCollection<P>
where
    P: PointT,
{
    fn read_body<Endianness, Reader>(
        header: crate::ewkb::EwkbHeader,
        reader: &mut Reader,
    ) -> Result<Self, std::io::Error>
    where
        Reader: std::io::Read,
        Endianness: byteorder::ByteOrder,
    {
        let geometries_n = reader.read_u32::<Endianness>()?;
        let mut g_collection = GeometryCollection::new(header.srid);
        for _i in 0..geometries_n {
            // skip 1 byte for byte order and 4 bytes for point type
            reader.read_u8()?;
            let geom_type = GeometryType::from(reader.read_u32::<Endianness>()?);
            let g_container = match geom_type {
                GeometryType::Point => {
                    GeometryContainer::Point(P::read_body::<Endianness, Reader>(header, reader)?)
                }
                GeometryType::LineString => {
                    GeometryContainer::LineString(LineString::read_body::<Endianness, Reader>(
                        header, reader,
                    )?)
                }
                GeometryType::Polygon => {
                    GeometryContainer::Polygon(Polygon::read_body::<Endianness, Reader>(
                        header, reader,
                    )?)
                }
                GeometryType::MultiPoint => {
                    GeometryContainer::MultiPoint(MultiPoint::read_body::<Endianness, Reader>(
                        header, reader,
                    )?)
                }
                GeometryType::MultiLineString => {
                    GeometryContainer::MultiLineString(MultiLineString::read_body::<
                        Endianness,
                        Reader,
                    >(header, reader)?)
                }
                GeometryType::MultiPolygon => GeometryContainer::MultiPolygon(
                    MultiPolygon::read_body::<Endianness, Reader>(header, reader)?,
                ),
                GeometryType::GeometryCollection => {
                    GeometryContainer::GeometryCollection(GeometryCollection::read_body::<
                        Endianness,
                        Reader,
                    >(header, reader)?)
                }
            };
            g_collection.geometries.push(g_container);
        }
        Ok(g_collection)
    }
}

impl<P> WriteToSql for GeometryCollection<P>
where
    P: PointT,
{
    fn write_body<Writer>(&self, out: &mut Writer) -> Result<(), std::io::Error>
    where
        Writer: std::io::Write,
    {
        out.write_u32::<LittleEndian>(self.geometries.len() as u32)?;
        for g_container in self.geometries.iter() {
            match g_container {
                GeometryContainer::Point(g) => g.write_to_sql(false, out)?,
                GeometryContainer::LineString(g) => g.write_to_sql(false, out)?,
                GeometryContainer::Polygon(g) => g.write_to_sql(false, out)?,
                GeometryContainer::MultiPoint(g) => g.write_to_sql(false, out)?,
                GeometryContainer::MultiLineString(g) => g.write_to_sql(false, out)?,
                GeometryContainer::MultiPolygon(g) => g.write_to_sql(false, out)?,
                GeometryContainer::GeometryCollection(g) => g.write_to_sql(false, out)?,
            };
        }
        Ok(())
    }
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
                    .add_point(Point::new(0.0, 0.0, None)).unwrap()
                    .to_owned()
            )
            .dimension()
        );
        assert_eq!(
            Dimension::Z as u32,
            GeometryContainer::LineString(
                LineString::new(None)
                    .add_point(PointZ::new(0.0, 0.0, 0.0, None)).unwrap()
                    .to_owned()
            )
            .dimension()
        );
        assert_eq!(
            Dimension::M as u32,
            GeometryContainer::LineString(
                LineString::new(None)
                    .add_point(PointM::new(0.0, 0.0, 0.0, None)).unwrap()
                    .to_owned()
            )
            .dimension()
        );
        assert_eq!(
            Dimension::ZM as u32,
            GeometryContainer::LineString(
                LineString::new(None)
                    .add_point(PointZM::new(0.0, 0.0, 0.0, 0.0, None)).unwrap()
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
                    .add_point(Point::new(0.0, 0.0, None)).unwrap()
                    .to_owned()
            )
            .dimension()
        );
        assert_eq!(
            Dimension::Z as u32,
            GeometryContainer::Polygon(
                Polygon::new(None)
                    .add_point(PointZ::new(0.0, 0.0, 0.0, None)).unwrap()
                    .to_owned()
            )
            .dimension()
        );
        assert_eq!(
            Dimension::M as u32,
            GeometryContainer::Polygon(
                Polygon::new(None)
                    .add_point(PointM::new(0.0, 0.0, 0.0, None)).unwrap()
                    .to_owned()
            )
            .dimension()
        );
        assert_eq!(
            Dimension::ZM as u32,
            GeometryContainer::Polygon(
                Polygon::new(None)
                    .add_point(PointZM::new(0.0, 0.0, 0.0, 0.0, None)).unwrap()
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
                    .add_point(Point::new(0.0, 0.0, None)).unwrap()
                    .to_owned()
            )
            .dimension()
        );
        assert_eq!(
            Dimension::Z as u32,
            GeometryContainer::MultiLineString(
                MultiLineString::new(None)
                    .add_point(PointZ::new(0.0, 0.0, 0.0, None)).unwrap()
                    .to_owned()
            )
            .dimension()
        );
        assert_eq!(
            Dimension::M as u32,
            GeometryContainer::MultiLineString(
                MultiLineString::new(None)
                    .add_point(PointM::new(0.0, 0.0, 0.0, None)).unwrap()
                    .to_owned()
            )
            .dimension()
        );
        assert_eq!(
            Dimension::ZM as u32,
            GeometryContainer::MultiLineString(
                MultiLineString::new(None)
                    .add_point(PointZM::new(0.0, 0.0, 0.0, 0.0, None)).unwrap()
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
                    .add_point(Point::new(0.0, 0.0, None)).unwrap()
                    .to_owned()
            )
            .dimension()
        );
        assert_eq!(
            Dimension::Z as u32,
            GeometryContainer::MultiPolygon(
                MultiPolygon::new(None)
                    .add_point(PointZ::new(0.0, 0.0, 0.0, None)).unwrap()
                    .to_owned()
            )
            .dimension()
        );
        assert_eq!(
            Dimension::M as u32,
            GeometryContainer::MultiPolygon(
                MultiPolygon::new(None)
                    .add_point(PointM::new(0.0, 0.0, 0.0, None)).unwrap()
                    .to_owned()
            )
            .dimension()
        );
        assert_eq!(
            Dimension::ZM as u32,
            GeometryContainer::MultiPolygon(
                MultiPolygon::new(None)
                    .add_point(PointZM::new(0.0, 0.0, 0.0, 0.0, None)).unwrap()
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
