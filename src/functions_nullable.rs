use crate::sql_types::*;
use diesel::sql_types::*;

//Topological Relationships****************************************************************

define_sql_function! {
    /// Tests if two geometries spatially intersect in 3D - only for points, linestrings, polygons, polyhedral surface (area).
    #[sql_name="ST_3DIntersects"]
    fn st_3d_intersects(left: Nullable<Geometry>, right: Nullable<Geometry>) -> Nullable<Bool>;
}
define_sql_function! {
    /// Tests if no points of B lie in the exterior of A, and A and B have at least one interior point in common.
    #[sql_name="ST_Contains"]
    fn st_contains(left: Nullable<Geometry>, right: Nullable<Geometry>) -> Nullable<Bool>;
}
define_sql_function! {
    /// Tests if B intersects the interior of A but not the boundary or exterior.
    #[sql_name="ST_ContainsProperly"]
    fn st_contains_properly(left: Nullable<Geometry>, right: Nullable<Geometry>) -> Nullable<Bool>;
}
define_sql_function! {
    /// Tests if no point in A is outside B
    #[sql_name="ST_CoveredBy"]
    fn st_covered_by<G: GeoType>(left: Nullable<G>, right: Nullable<G>) -> Nullable<Bool>;
}
define_sql_function! {
    /// Tests if no point in B is outside A
    #[sql_name="ST_Covers"]
    fn st_covers<G: GeoType>(left: Nullable<G>, right: Nullable<G>) -> Nullable<Bool>;
}
define_sql_function! {
    /// Tests if two geometries have some, but not all, interior points in common.
    #[sql_name="ST_Crosses"]
    fn st_crosses(left: Nullable<Geometry>, right: Nullable<Geometry>) -> Nullable<Bool>;
}
define_sql_function! {
    /// Tests if two geometries are disjoint (they have no point in common).
    #[sql_name="ST_Disjoint"]
    fn st_disjoint(left: Nullable<Geometry>, right: Nullable<Geometry>) -> Nullable<Bool>;
}
define_sql_function! {
    /// Tests if two geometries include the same set of points.
    #[sql_name="ST_Equals"]
    fn st_equals(left: Nullable<Geometry>, right: Nullable<Geometry>) -> Nullable<Bool>;
}
define_sql_function! {
    /// Tests if two geometries intersect (they have at least one point in common).
    #[sql_name="ST_Intersects"]
    fn st_intersects<G: GeoType>(left: Nullable<G>, right: Nullable<G>) -> Nullable<Bool>;
}
define_sql_function! {
    /// Returns a number indicating the crossing behavior of two LineStrings.
    #[sql_name="ST_LineCrossingDirection"]
    fn st_line_crossing_direction(left: Nullable<Geometry>, right: Nullable<Geometry>) -> Nullable<Integer>;
}
define_sql_function! {
    /// Tests if two geometries represent the same Nullable<Geometry> and have points in the same directional order.
    #[sql_name="ST_OrderingEquals"]
    fn st_ordering_equals(left: Nullable<Geometry>, right: Nullable<Geometry>) -> Nullable<Bool>;
}
define_sql_function! {
    /// Tests if two geometries intersect and have the same dimension, but are not completely contained by each other.
    #[sql_name="ST_Overlaps"]
    fn st_overlaps(left: Nullable<Geometry>, right: Nullable<Geometry>) -> Nullable<Bool>;
}
define_sql_function! {
    /// Tests if two geometries have a topological relationship matching an Intersection Matrix pattern.
    #[sql_name="ST_Relate"]
    fn st_relate_check(left: Nullable<Geometry>, right: Nullable<Geometry>, intersection_matrix_mattern: Nullable<Text>) -> Nullable<Bool>;
}
define_sql_function! {
    /// Computes Intersection Matrix of two geometries.
    #[sql_name="ST_Relate"]
    fn st_relate(left: Nullable<Geometry>, right: Nullable<Geometry>) -> Nullable<Text>;
}
define_sql_function! {
    /// Computes Intersection Matrix of two geometries. The boundary node rule code is: 1: OGC/MOD2, 2: Endpoint, 3: MultivalentEndpoint, 4: MonovalentEndpoint.
    #[sql_name="ST_Relate"]
    fn st_relate_bnr(left: Nullable<Geometry>, right: Nullable<Geometry>, boundary_node_rule: Nullable<Integer>) -> Nullable<Text>;
}
define_sql_function! {
    /// Tests if a DE-9IM Intersection Matrix matches an Intersection Matrix pattern
    #[sql_name="ST_RelateMatch"]
    fn st_relate_match(intersection_matrix: Nullable<Text>, intersection_matrix_pattern: Nullable<Text>) -> Nullable<Bool>;
}
define_sql_function! {
    /// Tests if two geometries have at least one point in common, but their interiors do not intersect.
    #[sql_name="ST_Touches"]
    fn st_touches(left: Nullable<Geometry>, right: Nullable<Geometry>) -> Nullable<Bool>;
}
define_sql_function! {
    /// Tests if no points of A lie in the exterior of B, and A and B have at least one interior point in common.
    #[sql_name="ST_Within"]
    fn st_within(left: Nullable<Geometry>, right: Nullable<Geometry>) -> Nullable<Bool>;
}
define_sql_function! {
    /// Tests if A and B are within a given distance.
    #[sql_name="ST_DWithin"]
    fn st_d_within<G: GeoType>(left: Nullable<G>, right: Nullable<G>, distance: Nullable<Double>) -> Nullable<Bool>;
}
define_sql_function! {
    /// Computes a geometry covering all points within a given distance from a geometry.
    #[sql_name="ST_Buffer"]
    fn st_buffer<G: GeoType>(geometry: Nullable<G>, radius_of_buffer: Nullable<Double>, buffer_style_parameters: Text) -> Nullable<G>;
}
define_sql_function! {
    /// Returns a modified geometry having no segment longer than the given max_segment_length. Distance computation is
    /// performed in 2d only. For geometry, length units are in units of spatial reference. For geography, units are in
    /// meters.
    #[sql_name="ST_Segmentize"]
    fn st_segmentize<G: GeoType>(geometry: Nullable<G>, max_segment_length: Double) -> Nullable<G>;
}
define_sql_function! {
    /// Divides geometry into parts until a part can be represented using no more than max_vertices.
    #[sql_name="ST_Subdivide"]
    fn st_subdivide<G: GeoType>(geometry: Nullable<G>, max_vertices: Integer, grid_size: Float8) -> Nullable<G>;
}

//Geometry Accessors****************************************************************
define_sql_function! {
    /// The inherent dimension of this Geometry object, which must be less than or equal to the coordinate dimension.
    #[sql_name="ST_Dimension"]
    fn st_dimensions(geometry: Nullable<Geometry>) -> Nullable<Integer>;
}
define_sql_function! {
    /// Returns the last point of a LINESTRING geometry as a POINT.
    #[sql_name="ST_EndPoint"]
    fn st_end_point(geometry: Nullable<Geometry>) -> Nullable<Geometry>;
}
define_sql_function! {
    /// Returns a geometry representing the double precision (float8) bounding box of the supplied geometry.
    #[sql_name="ST_Envelope"]
    fn st_envelope(geometry: Nullable<Geometry>) -> Nullable<Geometry>;
}
define_sql_function! {
    /// Returns a line string representing the exterior ring of the POLYGON geometry. Return NULL if the geometry is not a polygon. Will not work with MULTIPOLYGON.
    #[sql_name="ST_ExteriorRing"]
    fn st_exterior_ring(geometry: Nullable<Geometry>) -> Nullable<Geometry>;
}
define_sql_function! {
    /// Return the X coordinate of the point, or NULL if not available. Input must be a point.
    #[sql_name="ST_X"]
    fn st_x(geometry: Nullable<Geometry>) -> Nullable<Double>;
}
define_sql_function! {
    /// Return the Y coordinate of the point, or NULL if not available. Input must be a point.
    #[sql_name="ST_Y"]
    fn st_y(geometry: Nullable<Geometry>) -> Nullable<Double>;
}
define_sql_function! {
    /// Return the Z coordinate of the point, or NULL if not available. Input must be a point.
    #[sql_name="ST_Z"]
    fn st_z(geometry: Nullable<Geometry>) -> Nullable<Double>;
}
