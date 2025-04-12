use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::points::Dimension;
use crate::write_to_read_from_sql::{ReadFromSql, WriteToSql};
use crate::{
    ewkb::{EwkbSerializable, GeometryType},
    types::{LineString, PointT},
};

impl<P> EwkbSerializable for LineString<P>
where
    P: PointT,
{
    fn expected_geometry_variant(_: u32) -> GeometryType {
        GeometryType::LineString
    }

    fn geometry_type(&self) -> u32 {
        GeometryType::LineString as u32 | self.dimension()
    }

    fn srid(&self) -> Option<u32> {
        self.srid
    }
}

impl<P> LineString<P>
where
    P: PointT,
{
    pub fn new(srid: Option<u32>) -> Self {
        Self::with_capacity(srid, 0)
    }

    pub fn with_capacity(srid: Option<u32>, cap: usize) -> Self {
        LineString {
            points: Vec::with_capacity(cap),
            srid,
        }
    }

    /// Adds a point to the linestring.
    ///
    /// # Arguments
    ///
    /// * `point` - The point to add.
    ///
    /// # Errors
    ///
    /// * `IncompatibleSpatialReferenceSystemIdentifier` - If the point's SRID does not match the linestring's SRID.
    ///
    pub fn add_point(&mut self, point: P) -> Result<&mut Self, crate::errors::Error> {
        if point.srid() != self.srid {
            return Err(
                crate::errors::Error::IncompatibleSpatialReferenceSystemIdentifier {
                    expected: self.srid,
                    actual: point.srid(),
                },
            );
        }

        self.points.push(point);
        Ok(self)
    }

    pub fn add_points(&mut self, points: impl IntoIterator<Item = P>) -> Result<&mut Self, crate::errors::Error> {
        for point in points {
            self.add_point(point)?;
        }
        Ok(self)
    }

    pub fn dimension(&self) -> u32 {
        let mut dimension = Dimension::None as u32;
        if let Some(point) = self.points.first() {
            dimension |= point.dimension();
        }
        dimension
    }
}

impl<P> ReadFromSql for LineString<P>
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
        let len = reader.read_u32::<Endianness>()?;
        let mut ls = LineString::with_capacity(header.srid, len as usize);
        for _i in 0..len {
            ls.add_point(P::read_body::<Endianness, Reader>(header, reader)?)
                .unwrap();
        }
        Ok(ls)
    }
}

impl<P> WriteToSql for LineString<P>
where
    P: PointT,
{
    fn write_body<Writer>(&self, out: &mut Writer) -> Result<(), std::io::Error>
    where
        Writer: std::io::Write,
    {
        // size and points
        out.write_u32::<LittleEndian>(self.points.len() as u32)?;
        for point in self.points.iter() {
            point.write_body::<Writer>(out)?;
        }
        Ok(())
    }
}
