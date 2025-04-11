use std::io::Cursor;

use crate::{
    ewkb::{EwkbSerializable, GeometryType, BIG_ENDIAN},
    types::*,
};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

#[cfg(feature = "diesel")]
use crate::ewkb::{read_ewkb_header, write_ewkb_header};
use crate::sql_types::*;

pub enum Dimension {
    None = 0,
    Z = 0x80000000,
    M = 0x40000000,
    ZM = 0x40000000 | 0x80000000,
}

impl EwkbSerializable for Point {
    fn geometry_type(&self) -> u32 {
        GeometryType::Point as u32
    }
}

impl EwkbSerializable for PointZ {
    fn geometry_type(&self) -> u32 {
        GeometryType::Point as u32 | Dimension::Z as u32
    }
}

impl EwkbSerializable for PointM {
    fn geometry_type(&self) -> u32 {
        GeometryType::Point as u32 | Dimension::M as u32
    }
}

impl EwkbSerializable for PointZM {
    fn geometry_type(&self) -> u32 {
        GeometryType::Point as u32 | Dimension::ZM as u32
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

#[cfg(feature = "diesel")]
/// Deserialize a point from SQL raw bytes.
fn from_sql<P>(bytes: &[u8]) -> diesel::deserialize::Result<P>
where
    P: PointT,
{
    let mut r = Cursor::new(bytes);
    let end = r.read_u8()?;
    if end == BIG_ENDIAN {
        read_point::<BigEndian, P>(&mut r)
    } else {
        read_point::<LittleEndian, P>(&mut r)
    }
}

macro_rules! impl_point_from_to_sql {
    ($g:ident, $p:ident) => {
        #[cfg(feature = "postgres")]
        impl diesel::deserialize::FromSql<$g, diesel::pg::Pg> for $p {
            fn from_sql(bytes: diesel::pg::PgValue) -> diesel::deserialize::Result<Self> {
                from_sql(bytes.as_bytes())
            }
        }

        #[cfg(feature = "sqlite")]
        impl diesel::deserialize::FromSql<$g, diesel::sqlite::Sqlite> for $p {
            fn from_sql(
                mut bytes: diesel::sqlite::SqliteValue<'_, '_, '_>,
            ) -> diesel::deserialize::Result<Self> {
                from_sql(bytes.read_blob())
            }
        }

        #[cfg(feature = "postgres")]
        impl diesel::serialize::ToSql<$g, diesel::pg::Pg> for $p {
            fn to_sql(
                &self,
                out: &mut diesel::serialize::Output<diesel::pg::Pg>,
            ) -> diesel::serialize::Result {
                write_point(self, self.get_srid(), out)?;
                Ok(diesel::serialize::IsNull::No)
            }
        }

        #[cfg(feature = "sqlite")]
        impl diesel::serialize::ToSql<$g, diesel::sqlite::Sqlite> for $p {
            fn to_sql(
                &self,
                out: &mut diesel::serialize::Output<diesel::sqlite::Sqlite>,
            ) -> diesel::serialize::Result {
                let mut bytes = Vec::new();
                write_point(self, self.get_srid(), &mut bytes)?;
                out.set_value(bytes);
                Ok(diesel::serialize::IsNull::No)
            }
        }
    };
}

impl_point_from_to_sql!(Geometry, Point);
impl_point_from_to_sql!(Geometry, PointZ);
impl_point_from_to_sql!(Geometry, PointM);
impl_point_from_to_sql!(Geometry, PointZM);

impl_point_from_to_sql!(Geography, Point);
impl_point_from_to_sql!(Geography, PointZ);
impl_point_from_to_sql!(Geography, PointM);
impl_point_from_to_sql!(Geography, PointZM);

#[cfg(feature = "diesel")]
pub fn write_point<W, P>(point: &P, srid: Option<u32>, out: &mut W) -> diesel::serialize::Result
where
    P: PointT + EwkbSerializable,
    W: WriteBytesExt,
{
    write_ewkb_header(point, srid, out)?;
    write_point_coordinates(point, out)?;
    Ok(diesel::serialize::IsNull::No)
}

#[cfg(feature = "diesel")]
pub fn write_point_coordinates<W, P>(point: &P, out: &mut W) -> diesel::serialize::Result
where
    P: PointT,
    W: std::io::Write,
{
    out.write_f64::<LittleEndian>(point.get_x())?;
    out.write_f64::<LittleEndian>(point.get_y())?;
    if point.get_z().is_some() {
        out.write_f64::<LittleEndian>(point.get_z().unwrap())?;
    }
    if point.get_m().is_some() {
        out.write_f64::<LittleEndian>(point.get_m().unwrap())?;
    }
    Ok(diesel::serialize::IsNull::No)
}

#[cfg(feature = "diesel")]
fn read_point<T, P>(cursor: &mut Cursor<&[u8]>) -> diesel::deserialize::Result<P>
where
    T: byteorder::ByteOrder,
    P: PointT,
{
    let g_header = read_ewkb_header::<T>(cursor)?.expect(GeometryType::Point)?;
    read_point_coordinates::<T, P>(cursor, g_header.g_type, g_header.srid)
}

#[cfg(feature = "diesel")]
pub fn read_point_coordinates<T, P>(
    cursor: &mut Cursor<&[u8]>,
    g_type: u32,
    srid: Option<u32>,
) -> diesel::deserialize::Result<P>
where
    T: byteorder::ByteOrder,
    P: PointT,
{
    let x = cursor.read_f64::<T>()?;
    let y = cursor.read_f64::<T>()?;
    let mut z = None;
    if g_type & Dimension::Z as u32 == Dimension::Z as u32 {
        z = Some(cursor.read_f64::<T>()?);
    }
    let mut m = None;
    if g_type & Dimension::M as u32 == Dimension::M as u32 {
        m = Some(cursor.read_f64::<T>()?);
    }
    Ok(P::new_point(x, y, srid, z, m)?)
}

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
