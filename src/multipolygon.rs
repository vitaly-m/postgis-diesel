use crate::write_to_read_from_sql::{ReadFromSql, WriteToSql};
use crate::{
    ewkb::{EwkbSerializable, GeometryType},
    points::Dimension,
    types::*,
};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

impl<P> MultiPolygon<P>
where
    P: PointT,
{
    pub fn new(srid: Option<u32>) -> Self {
        Self::with_capacity(srid, 0)
    }

    pub fn with_capacity(srid: Option<u32>, cap: usize) -> Self {
        MultiPolygon {
            polygons: Vec::with_capacity(cap),
            srid,
        }
    }

    pub fn add_empty_polygon(&mut self) -> &mut Self {
        self.add_empty_polygon_with_capacity(0)
    }

    pub fn add_empty_polygon_with_capacity(&mut self, cap: usize) -> &mut Self {
        self.polygons.push(Polygon {
            rings: Vec::with_capacity(cap),
            srid: self.srid,
        });
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
        if self.polygons.is_empty() {
            self.add_empty_polygon();
        }
        self.polygons.last_mut().unwrap().add_point(point)?;
        Ok(self)
    }

    /// Adds multiple points to the polygon.
    ///
    /// # Arguments
    ///
    /// * `points` - The points to add.
    ///
    /// # Errors
    ///
    /// * `IncompatibleSpatialReferenceSystemIdentifier` - If the point's SRID does not match the polygon's SRID.
    ///
    pub fn add_points(
        &mut self,
        points: impl IntoIterator<Item = P>,
    ) -> Result<&mut Self, crate::errors::Error> {
        if self.polygons.is_empty() {
            self.add_empty_polygon();
        }
        let last = self.polygons.last_mut().unwrap();
        for point in points {
            last.add_point(point)?;
        }
        Ok(self)
    }

    pub fn dimension(&self) -> u32 {
        let mut dimension = Dimension::NONE;
        if let Some(polygon) = self.polygons.first() {
            dimension |= polygon.dimension();
        }
        dimension
    }
}

impl<P> EwkbSerializable for MultiPolygon<P>
where
    P: PointT,
{
    fn expected_geometry_variant(_: u32) -> GeometryType {
        GeometryType::MultiPolygon
    }

    fn geometry_type(&self) -> u32 {
        let mut g_type = GeometryType::MultiPolygon as u32;
        if let Some(polygon) = self.polygons.first() {
            g_type |= polygon.dimension();
        }
        g_type
    }

    fn srid(&self) -> Option<u32> {
        self.srid
    }
}

impl<P> ReadFromSql for MultiPolygon<P>
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
        let polygons_n = reader.read_u32::<Endianness>()?;
        let mut polygon = MultiPolygon::with_capacity(header.srid, polygons_n as usize);

        for _i in 0..polygons_n {
            // skip 1 byte for byte order and 4 bytes for point type
            reader.read_u8()?;
            reader.read_u32::<Endianness>()?;
            polygon
                .polygons
                .push(Polygon::<P>::read_body::<Endianness, Reader>(
                    header, reader,
                )?);
        }
        Ok(polygon)
    }
}

impl<P> WriteToSql for MultiPolygon<P>
where
    P: PointT,
{
    fn write_body<Writer>(&self, out: &mut Writer) -> Result<(), std::io::Error>
    where
        Writer: std::io::Write,
    {
        out.write_u32::<LittleEndian>(self.polygons.len() as u32)?;
        for polygon in self.polygons.iter() {
            polygon.write_to_sql(false, out)?;
        }
        Ok(())
    }
}
