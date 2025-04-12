use crate::{
    ewkb::{EwkbSerializable, GeometryType},
    types::*,
};

use crate::write_to_read_from_sql::{ReadFromSql, WriteToSql};

impl<P> GeometryContainer<P>
where
    P: PointT,
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

impl<P> EwkbSerializable for GeometryContainer<P>
where
    P: PointT,
{
    fn expected_geometry_variant(geometry_type_hint: u32) -> GeometryType {
        geometry_type_hint.into()
    }

    fn geometry_type(&self) -> u32 {
        match self {
            GeometryContainer::Point(g) => g.geometry_type(),
            GeometryContainer::LineString(g) => g.geometry_type(),
            GeometryContainer::Polygon(g) => g.geometry_type(),
            GeometryContainer::MultiPoint(g) => g.geometry_type(),
            GeometryContainer::MultiLineString(g) => g.geometry_type(),
            GeometryContainer::MultiPolygon(g) => g.geometry_type(),
            GeometryContainer::GeometryCollection(g) => g.geometry_type(),
        }
    }

    fn srid(&self) -> Option<u32> {
        match self {
            GeometryContainer::Point(g) => g.srid(),
            GeometryContainer::LineString(g) => g.srid(),
            GeometryContainer::Polygon(g) => g.srid(),
            GeometryContainer::MultiPoint(g) => g.srid(),
            GeometryContainer::MultiLineString(g) => g.srid(),
            GeometryContainer::MultiPolygon(g) => g.srid(),
            GeometryContainer::GeometryCollection(g) => g.srid(),
        }
    }
}

impl<P> WriteToSql for GeometryContainer<P>
where
    P: PointT,
{
    fn write_to_sql<W>(&self, _include_srid: bool, out: &mut W) -> Result<(), std::io::Error>
    where
        W: std::io::Write,
    {
        // In this case, we are NOT writing the header, hence we are
        // overwriting the default method of the `WriteToSql` trait
        self.write_body(out)
    }

    fn write_body<Writer>(&self, out: &mut Writer) -> Result<(), std::io::Error>
    where
        Writer: std::io::Write,
    {
        match self {
            GeometryContainer::Point(g) => g.write_to_sql(true, out),
            GeometryContainer::MultiPoint(g) => g.write_to_sql(true, out),
            GeometryContainer::LineString(g) => g.write_to_sql(true, out),
            GeometryContainer::MultiLineString(g) => g.write_to_sql(true, out),
            GeometryContainer::Polygon(g) => g.write_to_sql(true, out),
            GeometryContainer::MultiPolygon(g) => g.write_to_sql(true, out),
            GeometryContainer::GeometryCollection(g) => g.write_to_sql(true, out),
        }
    }
}

impl<P> ReadFromSql for GeometryContainer<P>
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
        Ok(match GeometryType::from(header.g_type) {
            GeometryType::Point => {
                GeometryContainer::Point(P::read_body::<Endianness, Reader>(header, reader)?)
            }
            GeometryType::MultiPoint => GeometryContainer::MultiPoint(
                MultiPoint::<P>::read_body::<Endianness, Reader>(header, reader)?,
            ),
            GeometryType::LineString => GeometryContainer::LineString(
                LineString::<P>::read_body::<Endianness, Reader>(header, reader)?,
            ),
            GeometryType::MultiLineString => {
                GeometryContainer::MultiLineString(MultiLineString::<P>::read_body::<
                    Endianness,
                    Reader,
                >(header, reader)?)
            }
            GeometryType::Polygon => GeometryContainer::Polygon(Polygon::<P>::read_body::<
                Endianness,
                Reader,
            >(header, reader)?),
            GeometryType::MultiPolygon => GeometryContainer::MultiPolygon(
                MultiPolygon::<P>::read_body::<Endianness, Reader>(header, reader)?,
            ),
            GeometryType::GeometryCollection => {
                GeometryContainer::GeometryCollection(GeometryCollection::<P>::read_body::<
                    Endianness,
                    Reader,
                >(header, reader)?)
            }
        })
    }
}
