use crate::write_to_read_from_sql::{ReadFromSql, WriteToSql};
use crate::{
    ewkb::{EwkbSerializable, GeometryType},
    points::Dimension,
    types::*,
};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

impl<P> MultiPoint<P>
where
    P: PointT,
{
    pub fn new(srid: Option<u32>) -> Self {
        Self::with_capacity(srid, 0)
    }

    pub fn with_capacity(srid: Option<u32>, cap: usize) -> Self {
        MultiPoint {
            points: Vec::with_capacity(cap),
            srid,
        }
    }

    pub fn add_point(&mut self, point: P) -> &mut Self {
        self.points.push(point);
        self
    }

    pub fn add_points(&mut self, points: impl IntoIterator<Item = P>) -> &mut Self {
        for point in points {
            self.points.push(point);
        }
        self
    }

    pub fn dimension(&self) -> u32 {
        let mut dimension = Dimension::None as u32;
        if let Some(point) = self.points.first() {
            dimension |= point.dimension();
        }
        dimension
    }
}

impl<P> EwkbSerializable for MultiPoint<P>
where
    P: PointT,
{
    fn expected_geometry_variant(_: u32) -> GeometryType {
        GeometryType::MultiPoint
    }

    fn geometry_type(&self) -> u32 {
        let mut g_type = GeometryType::MultiPoint as u32;
        if let Some(point) = self.points.first() {
            g_type |= point.dimension();
        }
        g_type
    }

    fn srid(&self) -> Option<u32> {
        self.srid
    }
}

impl<P> ReadFromSql for MultiPoint<P>
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
        let mut mp = MultiPoint::with_capacity(header.srid, len as usize);
        for _i in 0..len {
            // skip 1 byte for byte order and 4 bytes for point type
            reader.read_u8()?;
            reader.read_u32::<Endianness>()?;
            mp.add_point(P::read_body::<Endianness, Reader>(header, reader)?);
        }
        Ok(mp)
    }
}

impl<P> WriteToSql for MultiPoint<P>
where
    P: PointT,
{
    fn write_body<Writer>(&self, out: &mut Writer) -> Result<(), std::io::Error>
    where
        Writer: std::io::Write,
    {
        out.write_u32::<LittleEndian>(self.points.len() as u32)?;
        for point in self.points.iter() {
            point.write_to_sql::<Writer>(false, out)?;
        }
        Ok(())
    }
}
