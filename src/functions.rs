use crate::sql_types::*;
use diesel::sql_types::*;

//Topological Relationships****************************************************************

sql_function! {
    /// Tests if two geometries spatially intersect in 3D - only for points, linestrings, polygons, polyhedral surface (area).
    #[sql_name="ST_3DIntersects"]
    fn st_3d_intersects(left: Geometry, right: Geometry) -> Bool;
}
sql_function! {
    /// Tests if no points of B lie in the exterior of A, and A and B have at least one interior point in common.
    #[sql_name="ST_Contains"]
    fn st_contains(left: Geometry, right: Geometry) -> Bool;
}
sql_function! {
    /// Tests if B intersects the interior of A but not the boundary or exterior.
    #[sql_name="ST_ContainsProperly"]
    fn st_contains_properly(left: Geometry, right: Geometry) -> Bool;
}
sql_function! {
    /// Tests if no point in A is outside B
    #[sql_name="ST_CoveredBy"]
    fn st_covered_by<G: GeoType>(left: G, right: G) -> Bool;
}
sql_function! {
    /// Tests if no point in B is outside A
    #[sql_name="ST_Covers"]
    fn st_covers<G: GeoType>(left: G, right: G) -> Bool;
}
sql_function! {
    /// Tests if two geometries have some, but not all, interior points in common.
    #[sql_name="ST_Crosses"]
    fn st_crosses(left: Geometry, right: Geometry) -> Bool;
}
sql_function! {
    /// Tests if two geometries are disjoint (they have no point in common).
    #[sql_name="ST_Disjoint"]
    fn st_disjoint(left: Geometry, right: Geometry) -> Bool;
}
sql_function! {
    /// Tests if two geometries include the same set of points.
    #[sql_name="ST_Equals"]
    fn st_equals(left: Geometry, right: Geometry) -> Bool;
}
sql_function! {
    /// Tests if two geometries intersect (they have at least one point in common).
    #[sql_name="ST_Intersects"]
    fn st_intersects<G: GeoType>(left: G, right: G) -> Bool;
}
sql_function! {
    /// Returns a number indicating the crossing behavior of two LineStrings.
    #[sql_name="ST_LineCrossingDirection"]
    fn st_line_crossing_direction(left: Geometry, right: Geometry) -> Integer;
}
sql_function! {
    /// Tests if two geometries represent the same geometry and have points in the same directional order.
    #[sql_name="ST_OrderingEquals"]
    fn st_ordering_equals(left: Geometry, right: Geometry) -> Bool;
}
sql_function! {
    /// Tests if two geometries intersect and have the same dimension, but are not completely contained by each other.
    #[sql_name="ST_Overlaps"]
    fn st_overlaps(left: Geometry, right: Geometry) -> Bool;
}
sql_function! {
    /// Tests if two geometries have a topological relationship matching an Intersection Matrix pattern.
    #[sql_name="ST_Relate"]
    fn st_relate_check(left: Geometry, right: Geometry, intersection_matrix_mattern: Text) -> Bool;
}
sql_function! {
    /// Computes Intersection Matrix of two geometries.
    #[sql_name="ST_Relate"]
    fn st_relate(left: Geometry, right: Geometry) -> Text;
}
sql_function! {
    /// Computes Intersection Matrix of two geometries. The boundary node rule code is: 1: OGC/MOD2, 2: Endpoint, 3: MultivalentEndpoint, 4: MonovalentEndpoint.
    #[sql_name="ST_Relate"]
    fn st_relate_bnr(left: Geometry, right: Geometry, boundary_node_rule: Integer) -> Text;
}
sql_function! {
    /// Tests if a DE-9IM Intersection Matrix matches an Intersection Matrix pattern
    #[sql_name="ST_RelateMatch"]
    fn st_relate_match(intersection_matrix: Text, intersection_matrix_pattern: Text) -> Bool;
}
sql_function! {
    /// Tests if two geometries have at least one point in common, but their interiors do not intersect.
    #[sql_name="ST_Touches"]
    fn st_touches(left: Geometry, right: Geometry) -> Bool;
}
sql_function! {
    /// Tests if no points of A lie in the exterior of B, and A and B have at least one interior point in common.
    #[sql_name="ST_Within"]
    fn st_within(left: Geometry, right: Geometry) -> Bool;
}
sql_function! {
    /// Tests if A and B are within a given distance.
    #[sql_name="ST_DWithin"]
    fn st_d_within<G: GeoType>(left: G, right: G, distance: Double) -> Bool;
}

pub type St3DIntersects<GeomA, GeomB> = st_3d_intersects::HelperType<GeomA, GeomB>;
pub type StContains<GeomA, GeomB> = st_contains::HelperType<GeomA, GeomB>;
pub type StContainsProperly<GeomA, GeomB> = st_contains_properly::HelperType<GeomA, GeomB>;
pub type StCoveredBy<G, GeomA, GeomB> = st_covered_by::HelperType<G, GeomA, GeomB>;
pub type StCovers<G, GeomA, GeomB> = st_covers::HelperType<G, GeomA, GeomB>;
pub type StCrosses<GeomA, GeomB> = st_crosses::HelperType<GeomA, GeomB>;
pub type StDisjoint<GeomA, GeomB> = st_disjoint::HelperType<GeomA, GeomB>;
pub type StEquals<GeomA, GeomB> = st_equals::HelperType<GeomA, GeomB>;
pub type StIntersects<G, GeomA, GeomB> = st_intersects::HelperType<G, GeomA, GeomB>;
pub type StLineCrossingDirection<GeomA, GeomB> =
    st_line_crossing_direction::HelperType<GeomA, GeomB>;
pub type StOrderingEquals<GeomA, GeomB> = st_ordering_equals::HelperType<GeomA, GeomB>;
pub type StOverlaps<GeomA, GeomB> = st_overlaps::HelperType<GeomA, GeomB>;
pub type StRelateCheck<GeomA, GeomB, Matrix> = st_relate_check::HelperType<GeomA, GeomB, Matrix>;
pub type StRelate<GeomA, GeomB> = st_relate::HelperType<GeomA, GeomB>;
pub type StRelateBnr<GeomA, GeomB, BNRule> = st_relate_bnr::HelperType<GeomA, GeomB, BNRule>;
pub type StRelateMatch<GeomA, GeomB> = st_relate_match::HelperType<GeomA, GeomB>;
pub type StTouches<GeomA, GeomB> = st_touches::HelperType<GeomA, GeomB>;
pub type StWithin<GeomA, GeomB> = st_within::HelperType<GeomA, GeomB>;
pub type StDWithin<G, GeomA, GeomB, Distance> = st_d_within::HelperType<G, GeomA, GeomB, Distance>;
