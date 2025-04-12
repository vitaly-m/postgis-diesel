use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::write_to_read_from_sql::{ReadFromSql, WriteToSql};
use crate::{
    ewkb::{EwkbSerializable, GeometryType},
    types::{PointT, Polygon},
};

use crate::points::Dimension;

impl<P> Polygon<P>
where
    P: PointT,
{
    pub fn new(srid: Option<u32>) -> Self {
        Self::with_capacity(srid, 0)
    }

    pub fn with_capacity(srid: Option<u32>, cap: usize) -> Self {
        Polygon {
            rings: Vec::with_capacity(cap),
            srid,
        }
    }

    pub fn add_ring(&mut self) -> &mut Self {
        self.add_ring_with_capacity(0)
    }

    pub fn add_ring_with_capacity(&mut self, cap: usize) -> &mut Self {
        self.rings.push(Vec::with_capacity(cap));
        self
    }

    /// Adds a point to the polygon.
    ///
    /// # Arguments
    ///
    /// * `point` - The point to add.
    ///
    /// # Errors
    ///
    /// * `IncompatibleSpatialReferenceSystemIdentifier` - If the point's SRID does not match the polygon's SRID.
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
        if self.rings.last().is_none() {
            self.add_ring();
        }
        self.rings.last_mut().unwrap().push(point);
        Ok(self)
    }

    /// Adds multiple points to the polygon.
    ///
    /// # Arguments
    ///
    /// * `points` - An iterator of points to add.
    ///
    /// # Errors
    ///
    /// * `IncompatibleSpatialReferenceSystemIdentifier` - If any point's SRID does not match the polygon's SRID.
    ///
    pub fn add_points(
        &mut self,
        points: impl IntoIterator<Item = P>,
    ) -> Result<&mut Self, crate::errors::Error> {
        for point in points {
            self.add_point(point)?;
        }
        Ok(self)
    }

    pub fn dimension(&self) -> u32 {
        let mut dimension = Dimension::None as u32;
        if let Some(ring) = self.rings.first() {
            if let Some(point) = ring.first() {
                dimension |= point.dimension();
            }
        }
        dimension
    }
}

impl<P> EwkbSerializable for Polygon<P>
where
    P: PointT,
{
    fn expected_geometry_variant(_: u32) -> GeometryType {
        GeometryType::Polygon
    }

    fn geometry_type(&self) -> u32 {
        GeometryType::Polygon as u32 | self.dimension()
    }

    fn srid(&self) -> Option<u32> {
        self.srid
    }
}

impl<P> WriteToSql for Polygon<P>
where
    P: PointT,
{
    fn write_body<Writer>(&self, out: &mut Writer) -> Result<(), std::io::Error>
    where
        Writer: std::io::Write,
    {
        // number of rings
        out.write_u32::<LittleEndian>(self.rings.len() as u32)?;
        for ring in self.rings.iter() {
            //number of points in ring
            out.write_u32::<LittleEndian>(ring.len() as u32)?;
            for point in ring.iter() {
                point.write_body::<Writer>(out)?;
            }
        }
        Ok(())
    }
}

impl<P> ReadFromSql for Polygon<P>
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
        let rings_n = reader.read_u32::<Endianness>()?;
        let mut polygon = Polygon::with_capacity(header.srid, rings_n as usize);
        for _i in 0..rings_n {
            let points_n = reader.read_u32::<Endianness>()?;
            polygon.add_ring_with_capacity(points_n as usize);
            for _p in 0..points_n {
                polygon
                    .add_point(P::read_body::<Endianness, Reader>(header, reader)?)
                    .unwrap();
            }
        }
        Ok(polygon)
    }
}
