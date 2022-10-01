use std::fmt::{Debug};
use std::io::Cursor;

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use diesel::{
    deserialize::{self, FromSql},
    pg::{self, Pg},
    serialize::{self, IsNull, Output, ToSql},
};

use crate::points::{PointT, write_point_coordinates, read_point_coordinates};
use crate::sql_types::*;

#[derive(Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
pub struct Polygon<T> {
    pub rings: Vec<Vec<T>>,
    pub srid: Option<u32>,
}

impl<T> Polygon<T>
where T: PointT + Clone {
    pub fn new(srid: Option<u32>) -> Self {
        Polygon { rings: Vec::new(), srid: srid }
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
}

impl<T> ToSql<Geometry, Pg> for Polygon<T>
where
    T: PointT + Debug + PartialEq,
{
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
        if self.rings.len() < 1 {
            return Err(format!(
                "Polygon must contain at least one ring but has {}",
                self.rings.len()
            )
            .into());
        }
        if self.rings.first().unwrap().len() < 4 {
            return Err(format!(
                "The ring with {} elements can't be closed",
                self.rings.len()
            )
            .into());
        }
        out.write_u8(LITTLE_ENDIAN)?;
        // polygon can have points of the same type
        let mut g_type = GeometryType::Polygon as u32;
        let first_point = self.rings.first().unwrap().first().unwrap();
        if self.srid.is_some() {
            g_type |= SRID;
        }
        g_type |= first_point.dimension();
        match self.srid {
            Some(srid) => {
                out.write_u32::<LittleEndian>(g_type)?;
                out.write_u32::<LittleEndian>(srid)?;
            }
            None => out.write_u32::<LittleEndian>(g_type)?,
        }
        // number of rings
        out.write_u32::<LittleEndian>(self.rings.len() as u32)?;
        for (ring_n, ring) in self.rings.iter().enumerate() {
            if ring.len() < 4 {
                return Err(format!(
                    "The ring with {} elements can't be closed",
                    self.rings.len()
                )
                .into());
            }
            if ring.first().unwrap() != ring.last().unwrap() {
                return Err(format!(
                    "The {} ring is not closed",
                    ring_n
                )
                .into());
            }
            //number of points in ring
            out.write_u32::<LittleEndian>(ring.len() as u32)?;
            for point in ring.iter() {
                write_point_coordinates(point, out)?;
            }
        }
        Ok(IsNull::No)
    }
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
    let g_type = cursor.read_u32::<T>()?;
    if GeometryType::from(g_type) != GeometryType::Polygon {
        return Err(format!(
            "Geometry {:?} is not a Polygon",
            GeometryType::from(g_type)
        )
        .into());
    }
    let mut srid = None;
    // SRID included
    if g_type & SRID == SRID {
        srid = Some(cursor.read_u32::<T>()?);
    }
    let rings_n = cursor.read_u32::<T>()?;
    let mut polygon = Polygon::new(srid);

    for _i in 0..rings_n {
        polygon.add_ring();
        let points_n = cursor.read_u32::<T>()?;
        for _p in 0..points_n {
            polygon.add_point(read_point_coordinates::<T, P>(cursor, g_type, srid)?);
        }
    }
    Ok(polygon)
}

