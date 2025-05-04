//! Submodule implementing with a macro the `ToSql` of the Geometry variant for several types.

/// Macro implementing the `ToSql` and `FromSql` trait for the Geometry variant.
macro_rules! impl_to_sql_geometry {
	($($type:ty),+) => {
		$(
			#[cfg(feature = "postgres")]
			impl<P> diesel::deserialize::FromSql<crate::sql_types::Geometry, diesel::pg::Pg> for $type
			where
				P: crate::types::PointT,
			{
				fn from_sql(bytes: diesel::pg::PgValue) -> diesel::deserialize::Result<Self> {
					use crate::write_to_read_from_sql::ReadFromSql;
					Ok(Self::read_from_sql(bytes.as_bytes())?)
				}
			}

			#[cfg(feature = "postgres")]
			impl<P> diesel::serialize::ToSql<crate::sql_types::Geometry, diesel::pg::Pg> for $type
			where
				P: crate::types::PointT,
			{
				fn to_sql(
					&self,
					out: &mut diesel::serialize::Output<diesel::pg::Pg>,
				) -> diesel::serialize::Result {
					use crate::write_to_read_from_sql::WriteToSql;
					self.write_to_sql(true, out)?;
					Ok(diesel::serialize::IsNull::No)
				}
			}

			#[cfg(feature = "sqlite")]
			impl<P> diesel::deserialize::FromSql<crate::sql_types::Geometry, diesel::sqlite::Sqlite> for $type
			where
				P: crate::types::PointT,
			{
				fn from_sql(
					mut bytes: diesel::sqlite::SqliteValue<'_, '_, '_>,
				) -> diesel::deserialize::Result<Self> {
					use crate::write_to_read_from_sql::ReadFromSql;
					Ok(Self::read_from_sql(bytes.read_blob())?)
				}
			}

			#[cfg(feature = "sqlite")]
			impl<P> diesel::serialize::ToSql<crate::sql_types::Geometry, diesel::sqlite::Sqlite> for $type
			where
				P: crate::types::PointT,
			{
				fn to_sql(
					&self,
					out: &mut diesel::serialize::Output<diesel::sqlite::Sqlite>,
				) -> diesel::serialize::Result {
					use crate::write_to_read_from_sql::WriteToSql;
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
			impl diesel::deserialize::FromSql<crate::sql_types::Geometry, diesel::pg::Pg> for $type
			{
				fn from_sql(bytes: diesel::pg::PgValue) -> diesel::deserialize::Result<Self> {
					use crate::write_to_read_from_sql::ReadFromSql;
					Ok(Self::read_from_sql(bytes.as_bytes())?)
				}
			}

			#[cfg(feature = "postgres")]
			impl diesel::serialize::ToSql<crate::sql_types::Geometry, diesel::pg::Pg> for $type
			{
				fn to_sql(
					&self,
					out: &mut diesel::serialize::Output<diesel::pg::Pg>,
				) -> diesel::serialize::Result {
					use crate::write_to_read_from_sql::WriteToSql;
					self.write_to_sql(true, out)?;
					Ok(diesel::serialize::IsNull::No)
				}
			}

			#[cfg(feature = "sqlite")]
			impl diesel::deserialize::FromSql<crate::sql_types::Geometry, diesel::sqlite::Sqlite> for $type
			{
				fn from_sql(
					mut bytes: diesel::sqlite::SqliteValue<'_, '_, '_>,
				) -> diesel::deserialize::Result<Self> {
					use crate::write_to_read_from_sql::ReadFromSql;
					Ok(Self::read_from_sql(bytes.read_blob())?)

				}
			}

			#[cfg(feature = "sqlite")]
			impl diesel::serialize::ToSql<crate::sql_types::Geometry, diesel::sqlite::Sqlite> for $type
			{
				fn to_sql(
					&self,
					out: &mut diesel::serialize::Output<diesel::sqlite::Sqlite>,
				) -> diesel::serialize::Result {
					use crate::write_to_read_from_sql::WriteToSql;
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
    crate::types::MultiPoint<P>,
    crate::types::MultiLineString<P>,
    crate::types::MultiPolygon<P>,
    crate::types::GeometryCollection<P>,
    crate::types::GeometryContainer<P>,
    crate::types::LineString<P>,
    crate::types::Polygon<P>
);

impl_point_to_sql_geometry!(
    crate::types::Point,
    crate::types::PointZ,
    crate::types::PointM,
    crate::types::PointZM
);
