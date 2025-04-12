//! Submodule implementing with a macro the `ToSql` and `FromSql` of the Geography variant for several types.

/// Macro implementing the `ToSql` and `FromSql` trait for the Geography variant.
macro_rules! impl_to_sql_geography {
	($($type:ty),+) => {
		$(
			#[cfg(feature = "diesel")]
			impl<B, P> diesel::serialize::ToSql<crate::sql_types::Geography, B> for $type
			where
				P: crate::types::PointT,
				B: diesel::backend::Backend,
				Self: diesel::serialize::ToSql<crate::sql_types::Geometry, B>,
			{
				fn to_sql<'b>(
					&'b self,
					out: &mut diesel::serialize::Output<'b, '_, B>,
				) -> diesel::serialize::Result {
					diesel::serialize::ToSql::<crate::sql_types::Geometry, B>::to_sql(self, out)
				}
			}

			#[cfg(feature = "diesel")]
			impl<B, P> diesel::deserialize::FromSql<crate::sql_types::Geography, B> for $type
			where
				P: crate::types::PointT,
				B: diesel::backend::Backend,
				Self: diesel::deserialize::FromSql<crate::sql_types::Geometry, B>,
			{
				fn from_sql(bytes: B::RawValue<'_>) -> diesel::deserialize::Result<Self> {
					diesel::deserialize::FromSql::<crate::sql_types::Geometry, B>::from_sql(bytes)
				}
			}
		)+
	};
}

/// Macro implementing the `ToSql` and `FromSql` trait for the Geography variant.
macro_rules! impl_point_to_sql_geography {
	($($type:ty),+) => {
		$(
			#[cfg(feature = "diesel")]
			impl<B> diesel::serialize::ToSql<crate::sql_types::Geography, B> for $type
			where
				B: diesel::backend::Backend,
				Self: diesel::serialize::ToSql<crate::sql_types::Geometry, B>,
			{
				fn to_sql<'b>(
					&'b self,
					out: &mut diesel::serialize::Output<'b, '_, B>,
				) -> diesel::serialize::Result {
					diesel::serialize::ToSql::<crate::sql_types::Geometry, B>::to_sql(self, out)
				}
			}

			#[cfg(feature = "diesel")]
			impl<B> diesel::deserialize::FromSql<crate::sql_types::Geography, B> for $type
			where
				B: diesel::backend::Backend,
				Self: diesel::deserialize::FromSql<crate::sql_types::Geometry, B>,
			{
				fn from_sql(bytes: B::RawValue<'_>) -> diesel::deserialize::Result<Self> {
					diesel::deserialize::FromSql::<crate::sql_types::Geometry, B>::from_sql(bytes)
				}
			}
		)+
	};
}

impl_to_sql_geography!(
    crate::types::MultiPoint<P>,
    crate::types::MultiLineString<P>,
    crate::types::MultiPolygon<P>,
    crate::types::GeometryCollection<P>,
    crate::types::GeometryContainer<P>,
    crate::types::LineString<P>,
    crate::types::Polygon<P>
);

impl_point_to_sql_geography!(
    crate::types::Point,
    crate::types::PointZ,
    crate::types::PointM,
    crate::types::PointZM
);
