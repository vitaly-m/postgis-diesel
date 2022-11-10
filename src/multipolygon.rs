use std::fmt::Debug;
use std::io::Cursor;

use crate::{
    ewkb::{read_ewkb_header, write_ewkb_header, EwkbSerializable, GeometryType, BIG_ENDIAN},
    points::Dimension,
    polygon::{read_polygon_body, write_polygon},
    types::*,
};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use diesel::{
    deserialize::{self, FromSql},
    pg::{self, Pg},
    serialize::{self, IsNull, Output, ToSql},
};

use crate::sql_types::*;

impl<T> MultiPolygon<T>
where
    T: PointT + Clone,
{
    pub fn new(srid: Option<u32>) -> Self {
        MultiPolygon {
            polygons: Vec::new(),
            srid: srid,
        }
    }

    pub fn add_empty_polygon<'a>(&'a mut self) -> &mut Self {
        self.polygons.push(Polygon {
            rings: Vec::new(),
            srid: self.srid,
        });
        self
    }

    pub fn add_point<'a>(&'a mut self, point: T) -> &mut Self {
        if self.polygons.last().is_none() {
            self.add_empty_polygon();
        }
        self.polygons.last_mut().unwrap().add_point(point);
        self
    }

    pub fn add_points<'a>(&'a mut self, points: &[T]) -> &mut Self {
        if self.polygons.last().is_none() {
            self.add_empty_polygon();
        }
        let last = self.polygons.last_mut().unwrap();
        for point in points {
            last.add_point(point.to_owned());
        }
        self
    }

    pub fn dimension(&self) -> u32 {
        let mut dimension = Dimension::None as u32;
        if let Some(polygon) = self.polygons.first() {
            dimension |= polygon.dimension();
        }
        dimension
    }
}

impl<T> EwkbSerializable for MultiPolygon<T>
where
    T: PointT + Clone,
{
    fn geometry_type(&self) -> u32 {
        let mut g_type = GeometryType::MultiPolygon as u32;
        if let Some(polygon) = self.polygons.first() {
            g_type |= polygon.dimension();
        }
        g_type
    }
}

impl<T> ToSql<Geometry, Pg> for MultiPolygon<T>
where
    T: PointT + Debug + PartialEq + Clone + EwkbSerializable,
{
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
        write_multi_polygon(self, self.srid, out)
    }
}

impl<T> ToSql<Geography, Pg> for MultiPolygon<T>
where
    T: PointT + Debug + PartialEq + Clone + EwkbSerializable,
{
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
        write_multi_polygon(self, self.srid, out)
    }
}

impl<T> FromSql<Geometry, Pg> for MultiPolygon<T>
where
    T: PointT + Debug + Clone,
{
    fn from_sql(bytes: pg::PgValue) -> deserialize::Result<Self> {
        let mut r = Cursor::new(bytes.as_bytes());
        let end = r.read_u8()?;
        if end == BIG_ENDIAN {
            read_multi_polygon::<BigEndian, T>(&mut r)
        } else {
            read_multi_polygon::<LittleEndian, T>(&mut r)
        }
    }
}
impl<T> FromSql<Geography, Pg> for MultiPolygon<T>
where
    T: PointT + Debug + Clone,
{
    fn from_sql(bytes: pg::PgValue) -> deserialize::Result<Self> {
        FromSql::<Geometry, Pg>::from_sql(bytes)
    }
}

pub fn write_multi_polygon<T>(
    multipolygon: &MultiPolygon<T>,
    srid: Option<u32>,
    out: &mut Output<Pg>,
) -> serialize::Result
where
    T: PointT + EwkbSerializable + Clone,
{
    write_ewkb_header(multipolygon, srid, out)?;
    // number of polygons
    out.write_u32::<LittleEndian>(multipolygon.polygons.len() as u32)?;
    for polygon in multipolygon.polygons.iter() {
        write_polygon(polygon, None, out)?;
    }
    Ok(IsNull::No)
}

fn read_multi_polygon<T, P>(cursor: &mut Cursor<&[u8]>) -> deserialize::Result<MultiPolygon<P>>
where
    T: byteorder::ByteOrder,
    P: PointT + Clone,
{
    let g_header = read_ewkb_header::<T>(GeometryType::MultiPolygon, cursor)?;
    read_multi_polygon_body::<T, P>(g_header.g_type, g_header.srid, cursor)
}

pub fn read_multi_polygon_body<T, P>(
    g_type: u32,
    srid: Option<u32>,
    cursor: &mut Cursor<&[u8]>,
) -> deserialize::Result<MultiPolygon<P>>
where
    T: byteorder::ByteOrder,
    P: PointT + Clone,
{
    let polygons_n = cursor.read_u32::<T>()?;
    let mut polygon = MultiPolygon::new(srid);

    for _i in 0..polygons_n {
        // skip 1 byte for byte order and 4 bytes for point type
        cursor.read_u8()?;
        cursor.read_u32::<T>()?;
        polygon
            .polygons
            .push(read_polygon_body::<T, P>(g_type, srid, cursor)?);
    }
    Ok(polygon)
}
