use std::fmt::Debug;
use std::io::Cursor;

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use diesel::{
    deserialize::{self, FromSql},
    pg::{self, Pg},
    serialize::{self, IsNull, Output, ToSql},
};

use crate::{ewkb::{
    read_ewkb_header, write_ewkb_header, EwkbSerializable, GeometryType, BIG_ENDIAN,
}, types::{Polygon, PointT}};

use crate::points::{read_point_coordinates, write_point_coordinates, Dimension};
use crate::sql_types::*;

impl<T> Polygon<T>
where
    T: PointT + Clone,
{
    pub fn new(srid: Option<u32>) -> Self {
        Polygon {
            rings: Vec::new(),
            srid: srid,
        }
    }

    pub fn add_ring<'a>(&'a mut self) {
        self.rings.push(Vec::new());
    }

    pub fn add_point<'a>(&'a mut self, point: T) {
        if self.rings.last().is_none() {
            self.add_ring();
        }
        self.rings.last_mut().unwrap().push(point);
    }

    pub fn add_points<'a>(&'a mut self, points: &[T]) {
        if self.rings.last().is_none() {
            self.add_ring();
        }
        let last = self.rings.last_mut().unwrap();
        for point in points {
            last.push(point.to_owned());
        }
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

impl<T> EwkbSerializable for Polygon<T>
where
    T: PointT + Clone,
{
    fn geometry_type(&self) -> u32 {
        GeometryType::Polygon as u32 | self.dimension()
    }
}

impl<T> ToSql<Geometry, Pg> for Polygon<T>
where
    T: PointT + Debug + PartialEq + Clone + EwkbSerializable,
{
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
        write_polygon(self, self.srid, out)
    }
}

pub fn write_polygon<T>(
    polygon: &Polygon<T>,
    srid: Option<u32>,
    out: &mut Output<Pg>,
) -> serialize::Result
where
    T: PointT + EwkbSerializable + Clone,
{
    write_ewkb_header(polygon, srid, out)?;
    // number of rings
    out.write_u32::<LittleEndian>(polygon.rings.len() as u32)?;
    for ring in polygon.rings.iter() {
        //number of points in ring
        out.write_u32::<LittleEndian>(ring.len() as u32)?;
        for point in ring.iter() {
            write_point_coordinates(point, out)?;
        }
    }
    Ok(IsNull::No)
}

impl<T> FromSql<Geometry, Pg> for Polygon<T>
where
    T: PointT + Debug + Clone,
{
    fn from_sql(bytes: pg::PgValue) -> deserialize::Result<Self> {
        let mut r = Cursor::new(bytes.as_bytes());
        let end = r.read_u8()?;
        if end == BIG_ENDIAN {
            read_polygon::<BigEndian, T>(&mut r)
        } else {
            read_polygon::<LittleEndian, T>(&mut r)
        }
    }
}

fn read_polygon<T, P>(cursor: &mut Cursor<&[u8]>) -> deserialize::Result<Polygon<P>>
where
    T: byteorder::ByteOrder,
    P: PointT + Clone,
{
    let g_header = read_ewkb_header::<T>(GeometryType::Polygon, cursor)?;
    read_polygon_body::<T, P>(g_header.g_type, g_header.srid, cursor)
}

pub fn read_polygon_body<T, P>(g_type: u32, srid: Option<u32>, cursor: &mut Cursor<&[u8]>) -> deserialize::Result<Polygon<P>>
where
    T: byteorder::ByteOrder,
    P: PointT + Clone,
{
    let rings_n = cursor.read_u32::<T>()?;
    let mut polygon = Polygon::new(srid);
    for _i in 0..rings_n {
        polygon.add_ring();
        let points_n = cursor.read_u32::<T>()?;
        for _p in 0..points_n {
            polygon.add_point(read_point_coordinates::<T, P>(
                cursor,
                g_type,
                srid,
            )?);
        }
    }
    Ok(polygon)
}