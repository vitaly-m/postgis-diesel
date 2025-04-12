//! Submodule defining the WriteToSql ReadFromSql private traits.

use byteorder::{BigEndian, LittleEndian};

use crate::ewkb::{EwkbHeader, EwkbSerializable, BIG_ENDIAN};

#[cfg(feature = "diesel")]
pub trait WriteToSql: EwkbSerializable {
    fn write_to_sql<Writer>(
        &self,
        include_srid: bool,
        out: &mut Writer,
    ) -> Result<(), std::io::Error>
    where
        Writer: std::io::Write,
    {
        self.write_header(include_srid, out)?;
        self.write_body(out)
    }

    fn write_body<Writer>(&self, out: &mut Writer) -> Result<(), std::io::Error>
    where
        Writer: std::io::Write;
}

pub trait ReadFromSql: Sized + EwkbSerializable {
    fn read_from_sql(bytes: &[u8]) -> Result<Self, std::io::Error> {
        use byteorder::ReadBytesExt;
        use std::io::Cursor;
        let mut cursor = Cursor::new(bytes);
        let endianness = cursor.read_u8()?;
        if endianness == BIG_ENDIAN {
            let header = Self::read_header::<BigEndian, _>(&mut cursor)?;
            Self::read_body::<BigEndian, _>(
                header.expect(Self::expected_geometry_variant(header.g_type))?,
                &mut cursor,
            )
        } else {
            let header = Self::read_header::<LittleEndian, _>(&mut cursor)?;
            Self::read_body::<LittleEndian, _>(
                header.expect(Self::expected_geometry_variant(header.g_type))?,
                &mut cursor,
            )
        }
    }

    fn read_body<Endianness, Reader>(
        header: EwkbHeader,
        reader: &mut Reader,
    ) -> Result<Self, std::io::Error>
    where
        Reader: std::io::Read,
        Endianness: byteorder::ByteOrder;
}

#[cfg(test)]
mod test {
    use super::ReadFromSql;
    use super::WriteToSql;
    use crate::types::MultiLineString;
    use crate::types::{Point, PointM, PointZ, PointZM};

    #[test]
    fn test_encode_decode_point() {
        let point = Point::new(1.0, 2.0, None);
        let mut buffer = Vec::new();
        point.write_to_sql(true, &mut buffer).unwrap();
        let decoded_point = Point::read_from_sql(&buffer).unwrap();
        assert_eq!(point, decoded_point);
    }

    #[test]
    fn test_encode_decode_pointz() {
        let point = PointZ::new(1.0, 2.0, 3.0, None);
        let mut buffer = Vec::new();
        point.write_to_sql(true, &mut buffer).unwrap();
        let decoded_point = PointZ::read_from_sql(&buffer).unwrap();
        assert_eq!(point, decoded_point);
    }

    #[test]
    fn test_encode_decode_pointm() {
        let point = PointM::new(1.0, 2.0, 3.0, None);
        let mut buffer = Vec::new();
        point.write_to_sql(true, &mut buffer).unwrap();
        let decoded_point = PointM::read_from_sql(&buffer).unwrap();
        assert_eq!(point, decoded_point);
    }

    #[test]
    fn test_encode_decode_pointzm() {
        let point = PointZM::new(1.0, 2.0, 3.0, 4.0, None);
        let mut buffer = Vec::new();
        point.write_to_sql(true, &mut buffer).unwrap();
        let decoded_point = PointZM::read_from_sql(&buffer).unwrap();
        assert_eq!(point, decoded_point);
    }

    #[test]
    fn test_encode_decode_linestring() {
        let point1 = Point::new(1.0, 2.0, None);
        let point2 = Point::new(3.0, 4.0, None);
        let mut linestring = crate::types::LineString::new(None);
        linestring.add_point(point1).unwrap();
        linestring.add_point(point2).unwrap();
        let mut buffer = Vec::new();
        linestring.write_to_sql(true, &mut buffer).unwrap();
        let decoded_linestring = crate::types::LineString::read_from_sql(&buffer).unwrap();
        assert_eq!(linestring, decoded_linestring);
    }

    #[test]
    fn test_encode_decode_empty_multi_line_string() {
        let multiline: MultiLineString<Point> = MultiLineString::new(Some(4326));
        let mut buffer = Vec::new();
        multiline.write_to_sql(true, &mut buffer).unwrap();
        let decoded_multiline = crate::types::MultiLineString::read_from_sql(&buffer).unwrap();
        assert_eq!(multiline, decoded_multiline);
    }

    #[test]
    fn test_encode_decode_multi_line_string() {
        let point1 = Point::new(1.0, 2.0, Some(4326));
        let point2 = Point::new(3.0, 4.0, Some(4326));

        let mut multiline: MultiLineString<Point> = MultiLineString::new(Some(4326));
        multiline.add_points([point1, point2]).unwrap();
        multiline.add_points([point1, point2]).unwrap();

        let mut buffer = Vec::new();
        multiline.write_to_sql(true, &mut buffer).unwrap();
        let decoded_multiline = crate::types::MultiLineString::read_from_sql(&buffer).unwrap();
        assert_eq!(multiline, decoded_multiline);
    }
}
