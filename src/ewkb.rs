use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

#[derive(Debug, PartialEq)]
pub enum GeometryType {
    Point = 1,
    LineString,
    Polygon,
    MultiPoint,
    MultiLineString,
    MultiPolygon,
    GeometryCollection,
}

pub const SRID: u32 = 0x20000000;
pub const LITTLE_ENDIAN: u8 = 1;
pub const BIG_ENDIAN: u8 = 0;

impl From<u32> for GeometryType {
    fn from(t: u32) -> Self {
        if t & 7 == 7 {
            Self::GeometryCollection
        } else if t & 6 == 6 {
            Self::MultiPolygon
        } else if t & 5 == 5 {
            Self::MultiLineString
        } else if t & 4 == 4 {
            Self::MultiPoint
        } else if t & 3 == 3 {
            Self::Polygon
        } else if t & 2 == 2 {
            Self::LineString
        } else {
            Self::Point
        }
    }
}

pub trait EwkbSerializable {
    fn expected_geometry_variant(geometry_type_hint: u32) -> GeometryType;

    fn geometry_type(&self) -> u32;
    fn srid(&self) -> Option<u32>;

    fn write_header<W>(&self, include_srid: bool, out: &mut W) -> Result<(), std::io::Error>
    where
        W: std::io::Write,
    {
        out.write_u8(LITTLE_ENDIAN)?;
        let mut p_type = self.geometry_type();
        if include_srid {
            if let Some(srid) = self.srid() {
                p_type |= SRID;
                out.write_u32::<LittleEndian>(p_type)?;
                out.write_u32::<LittleEndian>(srid)?;
                return Ok(());
            }
        }

        out.write_u32::<LittleEndian>(p_type)
    }

    fn read_header<T, R>(cursor: &mut R) -> Result<EwkbHeader, std::io::Error>
    where
        R: std::io::Read,
        T: byteorder::ByteOrder,
    {
        let mut g_type = cursor.read_u32::<T>()?;
        let mut srid = None;
        // SRID included
        if g_type & SRID == SRID {
            g_type &= !SRID;
            srid = Some(cursor.read_u32::<T>()?);
        }
        Ok(EwkbHeader { g_type, srid })
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct EwkbHeader {
    pub g_type: u32,
    pub srid: Option<u32>,
}

impl EwkbHeader {
    pub fn expect(self, expected_type: GeometryType) -> Result<Self, std::io::Error> {
        if GeometryType::from(self.g_type) != expected_type {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Expected geometry type `{:?}`, but got `{:?}`",
                    GeometryType::from(self.g_type),
                    expected_type,
                ),
            ))
        } else {
            Ok(self)
        }
    }
}

#[cfg(test)]
mod test {
    use super::EwkbSerializable;
    use crate::ewkb::LITTLE_ENDIAN;
    use crate::types::MultiLineString;
    use crate::types::{Point, PointM, PointZ, PointZM};
    use byteorder::LittleEndian;
    use byteorder::ReadBytesExt;
    use std::io::Cursor;

    fn test_write_ewkb_header<T>(geometry: &T) -> Result<(), std::io::Error>
    where
        T: EwkbSerializable,
    {
        let mut buffer = Vec::new();
        geometry.write_header(true, &mut buffer)?;
        let mut cursor = Cursor::new(buffer.as_slice());
        let endianness = cursor.read_u8()?;
        assert_eq!(endianness, LITTLE_ENDIAN);
        let header = T::read_header::<LittleEndian, _>(&mut cursor)?;
        assert_eq!(header.g_type, geometry.geometry_type());
        assert_eq!(header.srid, geometry.srid());
        Ok(())
    }

    #[test]
    fn test_read_write_ewkb_header_point() {
        let point = Point::new(1.0, 2.0, None);
        test_write_ewkb_header(&point).unwrap();
    }

    #[test]
    fn test_read_write_ewkb_header_pointz() {
        let point = PointZ::new(1.0, 2.0, 3.0, None);
        test_write_ewkb_header(&point).unwrap();
    }

    #[test]
    fn test_read_write_ewkb_header_pointm() {
        let point = PointM::new(1.0, 2.0, 3.0, None);
        test_write_ewkb_header(&point).unwrap();
    }

    #[test]
    fn test_read_write_ewkb_header_pointzm() {
        let point = PointZM::new(1.0, 2.0, 3.0, 4.0, None);
        test_write_ewkb_header(&point).unwrap();
    }

    #[test]
    fn test_read_write_ewkb_header_linestring() {
        let point1 = Point::new(1.0, 2.0, None);
        let point2 = Point::new(3.0, 4.0, None);
        let mut linestring = crate::types::LineString::new(None);
        linestring.add_point(point1).unwrap();
        linestring.add_point(point2).unwrap();
        test_write_ewkb_header(&linestring).unwrap();
    }

    #[test]
    fn test_read_write_ewkb_header_empty_multi_line_string() {
        let multiline: MultiLineString<Point> = MultiLineString::new(Some(4326));
        test_write_ewkb_header(&multiline).unwrap();
    }

    #[test]
    fn test_read_write_ewkb_header_multi_line_string() {
        let point1 = Point::new(1.0, 2.0, Some(4326));
        let point2 = Point::new(3.0, 4.0, Some(4326));

        let mut multiline: MultiLineString<Point> = MultiLineString::new(Some(4326));
        multiline.add_points([point1, point2]).unwrap();
        multiline.add_points([point1, point2]).unwrap();

        test_write_ewkb_header(&multiline).unwrap();
    }
}
