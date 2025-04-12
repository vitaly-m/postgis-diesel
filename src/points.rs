use crate::ewkb::EwkbHeader;
use crate::{
    ewkb::{EwkbSerializable, GeometryType},
    types::*,
    write_to_read_from_sql::{ReadFromSql, WriteToSql},
};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

pub enum Dimension {
    None = 0,
    Z = 0x80000000,
    M = 0x40000000,
    ZM = 0x40000000 | 0x80000000,
}

impl EwkbSerializable for Point {
    fn expected_geometry_variant(_: u32) -> GeometryType {
        GeometryType::Point
    }

    fn geometry_type(&self) -> u32 {
        GeometryType::Point as u32
    }

    fn srid(&self) -> Option<u32> {
        self.srid
    }
}

impl EwkbSerializable for PointZ {
    fn expected_geometry_variant(_: u32) -> GeometryType {
        GeometryType::Point
    }

    fn geometry_type(&self) -> u32 {
        GeometryType::Point as u32 | Dimension::Z as u32
    }

    fn srid(&self) -> Option<u32> {
        self.srid
    }
}

impl EwkbSerializable for PointM {
    fn expected_geometry_variant(_: u32) -> GeometryType {
        GeometryType::Point
    }

    fn geometry_type(&self) -> u32 {
        GeometryType::Point as u32 | Dimension::M as u32
    }

    fn srid(&self) -> Option<u32> {
        self.srid
    }
}

impl EwkbSerializable for PointZM {
    fn expected_geometry_variant(_: u32) -> GeometryType {
        GeometryType::Point
    }

    fn geometry_type(&self) -> u32 {
        GeometryType::Point as u32 | Dimension::ZM as u32
    }
    fn srid(&self) -> Option<u32> {
        self.srid
    }
}

impl Point {
    pub fn new(x: f64, y: f64, srid: Option<u32>) -> Self {
        Point::new_point(x, y, srid, None, None).unwrap()
    }
}

impl PointZ {
    pub fn new(x: f64, y: f64, z: f64, srid: Option<u32>) -> Self {
        PointZ::new_point(x, y, srid, Some(z), None).unwrap()
    }
}

impl PointM {
    pub fn new(x: f64, y: f64, m: f64, srid: Option<u32>) -> Self {
        PointM::new_point(x, y, srid, None, Some(m)).unwrap()
    }
}

impl PointZM {
    pub fn new(x: f64, y: f64, z: f64, m: f64, srid: Option<u32>) -> Self {
        PointZM::new_point(x, y, srid, Some(z), Some(m)).unwrap()
    }
}

impl PointT for Point {
    fn get_x(&self) -> f64 {
        self.x
    }

    fn get_y(&self) -> f64 {
        self.y
    }

    fn get_srid(&self) -> Option<u32> {
        self.srid
    }

    fn get_z(&self) -> Option<f64> {
        None
    }

    fn get_m(&self) -> Option<f64> {
        None
    }

    fn dimension(&self) -> u32 {
        0
    }

    fn new_point(
        x: f64,
        y: f64,
        srid: Option<u32>,
        z: Option<f64>,
        m: Option<f64>,
    ) -> Result<Self, PointConstructorError> {
        if z.is_some() || m.is_some() {
            return Err(PointConstructorError {
                reason: format!("unexpectedly defined Z {:?} or M {:?} for Point", z, m)
                    .to_string(),
            });
        }
        Ok(Point { x, y, srid })
    }
}

impl PointT for PointZ {
    fn get_x(&self) -> f64 {
        self.x
    }

    fn get_y(&self) -> f64 {
        self.y
    }

    fn get_srid(&self) -> Option<u32> {
        self.srid
    }

    fn get_z(&self) -> Option<f64> {
        Some(self.z)
    }

    fn get_m(&self) -> Option<f64> {
        None
    }

    fn dimension(&self) -> u32 {
        Dimension::Z as u32
    }

    fn new_point(
        x: f64,
        y: f64,
        srid: Option<u32>,
        z: Option<f64>,
        m: Option<f64>,
    ) -> Result<Self, PointConstructorError> {
        if z.is_none() {
            return Err(PointConstructorError {
                reason: "Z is not defined, but mandatory for PointZ".to_string(),
            });
        }
        if m.is_some() {
            return Err(PointConstructorError {
                reason: format!("unexpectedly defined M {:?} for PointZ", m).to_string(),
            });
        }
        Ok(PointZ {
            x,
            y,
            z: z.unwrap(),
            srid,
        })
    }
}

impl PointT for PointM {
    fn get_x(&self) -> f64 {
        self.x
    }

    fn get_y(&self) -> f64 {
        self.y
    }

    fn get_srid(&self) -> Option<u32> {
        self.srid
    }

    fn get_z(&self) -> Option<f64> {
        None
    }

    fn get_m(&self) -> Option<f64> {
        Some(self.m)
    }

    fn dimension(&self) -> u32 {
        Dimension::M as u32
    }

    fn new_point(
        x: f64,
        y: f64,
        srid: Option<u32>,
        z: Option<f64>,
        m: Option<f64>,
    ) -> Result<Self, PointConstructorError> {
        if m.is_none() {
            return Err(PointConstructorError {
                reason: "M is not defined, but mandatory for PointM".to_string(),
            });
        }
        if z.is_some() {
            return Err(PointConstructorError {
                reason: format!("unexpectedly defined Z {:?} for PointM", z).to_string(),
            });
        }
        Ok(PointM {
            x,
            y,
            m: m.unwrap(),
            srid,
        })
    }
}

impl PointT for PointZM {
    fn get_x(&self) -> f64 {
        self.x
    }

    fn get_y(&self) -> f64 {
        self.y
    }

    fn get_srid(&self) -> Option<u32> {
        self.srid
    }

    fn get_z(&self) -> Option<f64> {
        Some(self.z)
    }

    fn get_m(&self) -> Option<f64> {
        Some(self.m)
    }

    fn dimension(&self) -> u32 {
        Dimension::ZM as u32
    }

    fn new_point(
        x: f64,
        y: f64,
        srid: Option<u32>,
        z: Option<f64>,
        m: Option<f64>,
    ) -> Result<Self, PointConstructorError> {
        if z.is_none() {
            return Err(PointConstructorError {
                reason: "Z is not defined, but mandatory for PointZM".to_string(),
            });
        }
        if m.is_none() {
            return Err(PointConstructorError {
                reason: "M is not defined, but mandatory for PointZM".to_string(),
            });
        }
        Ok(PointZM {
            x,
            y,
            z: z.unwrap(),
            m: m.unwrap(),
            srid,
        })
    }
}

fn write_body<Writer, P>(point: &P, writer: &mut Writer) -> Result<(), std::io::Error>
where
    Writer: std::io::Write,
    P: PointT,
{
    writer.write_f64::<LittleEndian>(point.get_x())?;
    writer.write_f64::<LittleEndian>(point.get_y())?;
    if point.get_z().is_some() {
        writer.write_f64::<LittleEndian>(point.get_z().unwrap())?;
    }
    if point.get_m().is_some() {
        writer.write_f64::<LittleEndian>(point.get_m().unwrap())?;
    }
    Ok(())
}

fn read_body<Endianness, Reader, P>(
    header: EwkbHeader,
    reader: &mut Reader,
) -> Result<P, std::io::Error>
where
    Reader: std::io::Read,
    Endianness: byteorder::ByteOrder,
    P: PointT,
{
    let x = reader.read_f64::<Endianness>()?;
    let y = reader.read_f64::<Endianness>()?;
    let mut z = None;
    if header.g_type & Dimension::Z as u32 == Dimension::Z as u32 {
        z = Some(reader.read_f64::<Endianness>()?);
    }
    let mut m = None;
    if header.g_type & Dimension::M as u32 == Dimension::M as u32 {
        m = Some(reader.read_f64::<Endianness>()?);
    }
    Ok(P::new_point(x, y, header.srid, z, m)?)
}

macro_rules! impl_point_read_write {
    ($($point:ty),+) => {
        $(
            impl WriteToSql for $point {
                fn write_body<Writer>(&self, out: &mut Writer) -> Result<(), std::io::Error>
                where
                    Writer: std::io::Write {
                        write_body(self, out)
                    }
            }

            impl ReadFromSql for $point {
                fn read_body<Endianness, Reader>(header: EwkbHeader, reader: &mut Reader) -> Result<Self, std::io::Error>
                where
                    Reader: std::io::Read,
                    Endianness: byteorder::ByteOrder {
                        read_body::<Endianness, Reader, Self>(header, reader)
                    }
            }
        )+
    };
}

impl_point_read_write!(Point, PointZ, PointM, PointZM);

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_point_dimensions() {
        assert_eq!(
            Dimension::None as u32,
            Point::new(0.0, 0.0, None).dimension()
        );
        assert_eq!(
            Dimension::Z as u32,
            PointZ::new(0.0, 0.0, 0.0, None).dimension()
        );
        assert_eq!(
            Dimension::M as u32,
            PointM::new(0.0, 0.0, 0.0, None).dimension()
        );
        assert_eq!(
            Dimension::ZM as u32,
            PointZM::new(0.0, 0.0, 0.0, 0.0, None).dimension()
        );
    }

    #[test]
    #[should_panic(expected = "unexpectedly defined Z Some(1.0) or M Some(1.0) for Point")]
    fn test_new_point_err() {
        Point::new_point(72.0, 64.0, None, Some(1.0), Some(1.0)).unwrap();
    }

    #[test]
    #[should_panic(expected = "Z is not defined, but mandatory for PointZ")]
    fn test_new_point_z_not_def_err() {
        PointZ::new_point(72.0, 64.0, None, None, None).unwrap();
    }

    #[test]
    #[should_panic(expected = "unexpectedly defined M Some(1.0) for PointZ")]
    fn test_new_point_z_m_def_err() {
        PointZ::new_point(72.0, 64.0, None, Some(1.0), Some(1.0)).unwrap();
    }

    #[test]
    #[should_panic(expected = "M is not defined, but mandatory for PointM")]
    fn test_new_point_m_not_def_err() {
        PointM::new_point(72.0, 64.0, None, None, None).unwrap();
    }

    #[test]
    #[should_panic(expected = "unexpectedly defined Z Some(1.0) for PointM")]
    fn test_new_point_m_z_def_err() {
        PointM::new_point(72.0, 64.0, None, Some(1.0), Some(1.0)).unwrap();
    }

    #[test]
    #[should_panic(expected = "Z is not defined, but mandatory for PointZM")]
    fn test_new_point_zm_z_not_def_err() {
        PointZM::new_point(72.0, 64.0, None, None, Some(1.0)).unwrap();
    }

    #[test]
    #[should_panic(expected = "M is not defined, but mandatory for PointZM")]
    fn test_new_point_zm_m_not_def_err() {
        PointZM::new_point(72.0, 64.0, None, Some(1.0), None).unwrap();
    }
}
