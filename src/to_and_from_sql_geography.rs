//! Submodule implementing with a macro the `ToSql` and `FromSql` of the Geography variant for several types.

use std::fmt::Debug;

use crate::{
    ewkb::EwkbSerializable,
    sql_types::{Geography, Geometry},
    types::{
        GeometryCollection, GeometryContainer, LineString, MultiLineString, MultiPoint,
        MultiPolygon, PointT, Polygon,
    },
};

/// Macro implementing the `ToSql` and `FromSql` trait for the Geography variant.
macro_rules! impl_to_sql_geography {
	($($type:ty),+) => {
		$(
			#[cfg(feature = "diesel")]
			impl<B, P> diesel::serialize::ToSql<Geography, B> for $type
			where
				P: PointT + Debug + PartialEq + Clone + EwkbSerializable,
				B: diesel::backend::Backend,
				Self: diesel::serialize::ToSql<Geometry, B>,
			{
				fn to_sql<'b>(
					&'b self,
					out: &mut diesel::serialize::Output<'b, '_, B>,
				) -> diesel::serialize::Result {
					diesel::serialize::ToSql::<Geometry, B>::to_sql(self, out)
				}
			}

			#[cfg(feature = "diesel")]
			impl<B, P> diesel::deserialize::FromSql<Geography, B> for $type
			where
				P: PointT + Debug + Clone,
				B: diesel::backend::Backend,
				Self: diesel::deserialize::FromSql<Geometry, B>,
			{
				fn from_sql(bytes: B::RawValue<'_>) -> diesel::deserialize::Result<Self> {
					diesel::deserialize::FromSql::<Geometry, B>::from_sql(bytes)
				}
			}
		)+
	};
}

impl_to_sql_geography!(
    MultiPoint<P>,
    MultiLineString<P>,
    MultiPolygon<P>,
    GeometryCollection<P>,
    GeometryContainer<P>,
    LineString<P>,
    Polygon<P>
);
