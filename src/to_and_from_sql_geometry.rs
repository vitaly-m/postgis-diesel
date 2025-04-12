//! Submodule implementing with a macro the `ToSql` of the Geometry variant for several types.

use crate::types::GeometryContainer;
use crate::write_to_read_from_sql::{ReadFromSql, WriteToSql};
use crate::{
    ewkb::EwkbSerializable,
    sql_types::Geometry,
    types::{
        GeometryCollection, LineString, MultiLineString, MultiPoint, MultiPolygon, Point, PointM,
        PointT, PointZ, PointZM, Polygon,
    },
};

/// Macro implementing the `ToSql` and `FromSql` trait for the Geometry variant.
macro_rules! impl_to_sql_geometry {
	($($type:ty),+) => {
		$(
			#[cfg(feature = "postgres")]
			impl<P> diesel::deserialize::FromSql<Geometry, diesel::pg::Pg> for $type
			where
				P: PointT,
			{
				fn from_sql(bytes: diesel::pg::PgValue) -> diesel::deserialize::Result<Self> {
					Ok(Self::read_from_sql(bytes.as_bytes())?)
				}
			}

			#[cfg(feature = "postgres")]
			impl<P> diesel::serialize::ToSql<Geometry, diesel::pg::Pg> for $type
			where
				P: PointT + PartialEq + Clone + EwkbSerializable + WriteToSql,
			{
				fn to_sql(
					&self,
					out: &mut diesel::serialize::Output<diesel::pg::Pg>,
				) -> diesel::serialize::Result {
					self.write_to_sql(true, out)?;
					Ok(diesel::serialize::IsNull::No)
				}
			}

			#[cfg(feature = "sqlite")]
			impl<P> diesel::deserialize::FromSql<Geometry, diesel::sqlite::Sqlite> for $type
			where
				P: PointT,
			{
				fn from_sql(
					mut bytes: diesel::sqlite::SqliteValue<'_, '_, '_>,
				) -> diesel::deserialize::Result<Self> {
					Ok(Self::read_from_sql(bytes.read_blob())?)
				}
			}

			#[cfg(feature = "sqlite")]
			impl<P> diesel::serialize::ToSql<Geometry, diesel::sqlite::Sqlite> for $type
			where
				P: PointT + PartialEq + Clone + EwkbSerializable + WriteToSql,
			{
				fn to_sql(
					&self,
					out: &mut diesel::serialize::Output<diesel::sqlite::Sqlite>,
				) -> diesel::serialize::Result {
					let mut buffer = Vec::new();
					self.write_to_sql(true, &mut buffer)?;
					out.set_value(buffer);
					Ok(diesel::serialize::IsNull::No)
				}
			}
		)+
	};
}

/// Macro implementing the `ToSql` and `FromSql` trait for the Geometry variant.
macro_rules! impl_point_to_sql_geometry {
	($($type:ty),+) => {
		$(
			#[cfg(feature = "postgres")]
			impl diesel::deserialize::FromSql<Geometry, diesel::pg::Pg> for $type
			{
				fn from_sql(bytes: diesel::pg::PgValue) -> diesel::deserialize::Result<Self> {
					Ok(Self::read_from_sql(bytes.as_bytes())?)
				}
			}

			#[cfg(feature = "postgres")]
			impl diesel::serialize::ToSql<Geometry, diesel::pg::Pg> for $type
			{
				fn to_sql(
					&self,
					out: &mut diesel::serialize::Output<diesel::pg::Pg>,
				) -> diesel::serialize::Result {
					self.write_to_sql(true, out)?;
					Ok(diesel::serialize::IsNull::No)
				}
			}

			#[cfg(feature = "sqlite")]
			impl diesel::deserialize::FromSql<Geometry, diesel::sqlite::Sqlite> for $type
			{
				fn from_sql(
					mut bytes: diesel::sqlite::SqliteValue<'_, '_, '_>,
				) -> diesel::deserialize::Result<Self> {
					Ok(Self::read_from_sql(bytes.read_blob())?)

				}
			}

			#[cfg(feature = "sqlite")]
			impl diesel::serialize::ToSql<Geometry, diesel::sqlite::Sqlite> for $type
			{
				fn to_sql(
					&self,
					out: &mut diesel::serialize::Output<diesel::sqlite::Sqlite>,
				) -> diesel::serialize::Result {
					let mut buffer = Vec::new();
					self.write_to_sql(true, &mut buffer)?;
					out.set_value(buffer);
					Ok(diesel::serialize::IsNull::No)
				}
			}
		)+
	};
}

impl_to_sql_geometry!(
    MultiPoint<P>,
    MultiLineString<P>,
    MultiPolygon<P>,
    GeometryCollection<P>,
    GeometryContainer<P>,
    LineString<P>,
    Polygon<P>
);

impl_point_to_sql_geometry!(Point, PointZ, PointM, PointZM);
