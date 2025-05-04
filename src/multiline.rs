use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::write_to_read_from_sql::{ReadFromSql, WriteToSql};
use crate::{
    ewkb::{EwkbSerializable, GeometryType},
    points::Dimension,
    types::{LineString, MultiLineString, PointT},
};

impl<P> MultiLineString<P>
where
    P: PointT,
{
    pub fn new(srid: Option<u32>) -> Self {
        Self::with_capacity(srid, 0)
    }

    pub fn with_capacity(srid: Option<u32>, cap: usize) -> Self {
        MultiLineString {
            lines: Vec::with_capacity(cap),
            srid,
        }
    }

    pub fn add_line(&mut self) -> &mut Self {
        self.add_line_with_cap(0)
    }

    pub fn add_line_with_cap(&mut self, cap: usize) -> &mut Self {
        self.lines.push(LineString::with_capacity(self.srid, cap));
        self
    }

    /// Adds a point to the last line of the multiline.
    ///
    /// # Arguments
    ///
    /// * `point` - The point to add.
    ///
    /// # Errors
    ///
    /// * `IncompatibleSpatialReferenceSystemIdentifier` - If the point's SRID does not match the multiline's SRID.
    ///
    pub fn add_point(&mut self, point: P) -> Result<&mut Self, crate::errors::Error> {
        if self.lines.last().is_none() {
            self.add_line();
        }
        self.lines.last_mut().unwrap().add_point(point)?;
        Ok(self)
    }

    /// Adds multiple points to the last line of the multiline.
    ///
    /// # Arguments
    ///
    /// * `points` - The points to add.
    ///
    /// # Errors
    ///
    /// * `IncompatibleSpatialReferenceSystemIdentifier` - If the point's SRID does not match the multiline's SRID.
    ///
    pub fn add_points(
        &mut self,
        points: impl IntoIterator<Item = P>,
    ) -> Result<&mut Self, crate::errors::Error> {
        if self.lines.last().is_none() {
            self.add_line();
        }
        let last = self.lines.last_mut().unwrap();
        for point in points {
            last.add_point(point)?;
        }
        Ok(self)
    }

    pub fn dimension(&self) -> u32 {
        let mut dimension = Dimension::NONE;
        if let Some(line) = self.lines.first() {
            dimension |= line.dimension();
        }
        dimension
    }
}

impl<P> EwkbSerializable for MultiLineString<P>
where
    P: PointT,
{
    fn expected_geometry_variant(_: u32) -> GeometryType {
        GeometryType::MultiLineString
    }

    fn geometry_type(&self) -> u32 {
        let mut g_type = GeometryType::MultiLineString as u32;
        if let Some(line) = self.lines.first() {
            g_type |= line.dimension();
        }
        g_type
    }

    fn srid(&self) -> Option<u32> {
        self.srid
    }
}

impl<P> WriteToSql for MultiLineString<P>
where
    P: PointT,
{
    fn write_body<Writer>(&self, out: &mut Writer) -> Result<(), std::io::Error>
    where
        Writer: std::io::Write,
    {
        // number of lines
        out.write_u32::<LittleEndian>(self.lines.len() as u32)?;
        for line in self.lines.iter() {
            line.write_to_sql(false, out)?;
        }
        Ok(())
    }
}

impl<P> ReadFromSql for MultiLineString<P>
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
        let lines_n = reader.read_u32::<Endianness>().unwrap();
        println!("read lines_n: {:?}", lines_n);
        let mut multiline = MultiLineString::with_capacity(header.srid, lines_n as usize);
        for _i in 0..lines_n {
            println!("read line");
            // skip 1 byte for byte order and 4 bytes for point type
            reader.read_u8().unwrap();
            reader.read_u32::<Endianness>().unwrap();
            let points_n = reader.read_u32::<Endianness>().unwrap();
            println!("read points_n: {:?}", points_n);
            multiline.add_line_with_cap(points_n as usize);
            for _p in 0..points_n {
                let point = P::read_body::<Endianness, Reader>(header, reader)?;
                println!("read point {:?}", point);
                multiline.add_point(point).unwrap();
            }
        }
        Ok(multiline)
    }
}
