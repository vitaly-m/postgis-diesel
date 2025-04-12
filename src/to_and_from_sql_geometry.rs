//! Submodule implementing with a macro the `ToSql` of the Geometry variant for several types.

use std::fmt::Debug;

use crate::types::GeometryContainer;
#[cfg(feature = "diesel")]
use crate::write_to_read_from_sql::{ReadFromSql, WriteToSql};
use crate::{
    ewkb::EwkbSerializable,
    sql_types::Geometry,
    types::{
        GeometryCollection, LineString, MultiLineString, MultiPoint, MultiPolygon, PointT, Polygon,
    },
};

/// Macro implementing the `ToSql` and `FromSql` trait for the Geometry variant.
macro_rules! impl_to_sql_geometry {
	($($type:ty),+) => {
		$(
			#[cfg(feature = "postgres")]
			impl<P> diesel::deserialize::FromSql<Geometry, diesel::pg::Pg> for $type
			where
				P: PointT + Debug + Clone,
			{
				fn from_sql(bytes: diesel::pg::PgValue) -> diesel::deserialize::Result<Self> {
					Self::read_from_sql(bytes.as_bytes())
				}
			}

			#[cfg(feature = "postgres")]
			impl<P> diesel::serialize::ToSql<Geometry, diesel::pg::Pg> for $type
			where
				P: PointT + Debug + PartialEq + Clone + EwkbSerializable,
			{
				fn to_sql(
					&self,
					out: &mut diesel::serialize::Output<diesel::pg::Pg>,
				) -> diesel::serialize::Result {
					let outcome = self.write_to_sql(out);
					println!("outcome: {:?}", out);
					outcome
				}
			}

			#[cfg(feature = "sqlite")]
			impl<P> diesel::deserialize::FromSql<Geometry, diesel::sqlite::Sqlite> for $type
			where
				P: PointT + Debug + Clone,
			{
				fn from_sql(
					mut bytes: diesel::sqlite::SqliteValue<'_, '_, '_>,
				) -> diesel::deserialize::Result<Self> {
					Self::read_from_sql(bytes.read_blob())
				}
			}

			#[cfg(feature = "sqlite")]
			impl<P> diesel::serialize::ToSql<Geometry, diesel::sqlite::Sqlite> for $type
			where
				P: PointT + Debug + PartialEq + Clone + EwkbSerializable,
			{
				fn to_sql(
					&self,
					out: &mut diesel::serialize::Output<diesel::sqlite::Sqlite>,
				) -> diesel::serialize::Result {
					let mut buffer = Vec::new();
					let output = self.write_to_sql(&mut buffer)?;
					let decoded = Self::read_from_sql(buffer.as_ref()).unwrap();
					out.set_value(buffer);
					Ok(output)
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
