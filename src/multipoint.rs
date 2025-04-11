use std::fmt::Debug;
use std::io::Cursor;

#[cfg(feature = "diesel")]
use crate::{
    ewkb::{read_ewkb_header, write_ewkb_header},
    points::{read_point_coordinates, write_point},
    write_to_read_from_sql::{ReadFromSql, WriteToSql},
};
use crate::{
    ewkb::{EwkbSerializable, GeometryType, BIG_ENDIAN},
    points::Dimension,
    types::*,
};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

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
    fn geometry_type(&self) -> u32 {
        let mut g_type = GeometryType::MultiPoint as u32;
        if let Some(point) = self.points.first() {
            g_type |= point.dimension();
        }
        g_type
    }
}

#[cfg(feature = "diesel")]
impl<P> ReadFromSql for MultiPoint<P>
where
    P: PointT + Debug + Clone,
{
    fn read_from_sql(bytes: &[u8]) -> diesel::deserialize::Result<Self> {
        let mut r = Cursor::new(bytes);
        let end = r.read_u8()?;
        if end == BIG_ENDIAN {
            read_multipoint::<BigEndian, P>(&mut r)
        } else {
            read_multipoint::<LittleEndian, P>(&mut r)
        }
    }
}

#[cfg(feature = "diesel")]
impl<P> WriteToSql for MultiPoint<P>
where
    P: PointT + EwkbSerializable,
{
    fn write_to_sql<W>(&self, out: &mut W) -> diesel::serialize::Result
    where
        W: std::io::Write,
    {
        write_ewkb_header(self, self.srid, out)?;
        // size and points
        out.write_u32::<LittleEndian>(self.points.len() as u32)?;
        for point in self.points.iter() {
            write_point(point, None, out)?;
        }
        Ok(diesel::serialize::IsNull::No)
    }
}

#[cfg(feature = "diesel")]
fn read_multipoint<T, P>(cursor: &mut Cursor<&[u8]>) -> diesel::deserialize::Result<MultiPoint<P>>
where
    T: byteorder::ByteOrder,
    P: PointT + Clone,
{
    let g_header = read_ewkb_header::<T>(cursor)?.expect(GeometryType::MultiPoint)?;
    read_multi_point_body::<T, P>(g_header.g_type, g_header.srid, cursor)
}

#[cfg(feature = "diesel")]
pub fn read_multi_point_body<T, P>(
    g_type: u32,
    srid: Option<u32>,
    cursor: &mut Cursor<&[u8]>,
) -> diesel::deserialize::Result<MultiPoint<P>>
where
    T: byteorder::ByteOrder,
    P: PointT + Clone,
{
    let len = cursor.read_u32::<T>()?;
    let mut mp = MultiPoint::with_capacity(srid, len as usize);
    for _i in 0..len {
        // skip 1 byte for byte order and 4 bytes for point type
        cursor.read_u8()?;
        cursor.read_u32::<T>()?;
        mp.add_point(read_point_coordinates::<T, P>(cursor, g_type, srid)?);
    }
    Ok(mp)
}
