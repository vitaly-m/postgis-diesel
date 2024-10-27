use crate::sql_types::*;
use diesel::sql_types::*;

//Topological Relationships****************************************************************

define_sql_function! {
    /// Tests if two geometries spatially intersect in 3D - only for points, linestrings, polygons, polyhedral surface (area).
    #[sql_name="ST_3DIntersects"]
    fn st_3d_intersects(left: Geometry, right: Geometry) -> Bool;
}
define_sql_function! {
    /// Tests if no points of B lie in the exterior of A, and A and B have at least one interior point in common.
    #[sql_name="ST_Contains"]
    fn st_contains(left: Geometry, right: Geometry) -> Bool;
}
define_sql_function! {
    /// Tests if B intersects the interior of A but not the boundary or exterior.
    #[sql_name="ST_ContainsProperly"]
    fn st_contains_properly(left: Geometry, right: Geometry) -> Bool;
}
define_sql_function! {
    /// Tests if no point in A is outside B
    #[sql_name="ST_CoveredBy"]
    fn st_covered_by<G: GeoType>(left: G, right: G) -> Bool;
}
define_sql_function! {
    /// Tests if no point in B is outside A
    #[sql_name="ST_Covers"]
    fn st_covers<G: GeoType>(left: G, right: G) -> Bool;
}
define_sql_function! {
    /// Tests if two geometries have some, but not all, interior points in common.
    #[sql_name="ST_Crosses"]
    fn st_crosses(left: Geometry, right: Geometry) -> Bool;
}
define_sql_function! {
    /// Tests if two geometries are disjoint (they have no point in common).
    #[sql_name="ST_Disjoint"]
    fn st_disjoint(left: Geometry, right: Geometry) -> Bool;
}
define_sql_function! {
    /// Tests if two geometries include the same set of points.
    #[sql_name="ST_Equals"]
    fn st_equals(left: Geometry, right: Geometry) -> Bool;
}
define_sql_function! {
    /// Tests if two geometries intersect (they have at least one point in common).
    #[sql_name="ST_Intersects"]
    fn st_intersects<G: GeoType>(left: G, right: G) -> Bool;
}
define_sql_function! {
    /// Returns a number indicating the crossing behavior of two LineStrings.
    #[sql_name="ST_LineCrossingDirection"]
    fn st_line_crossing_direction(left: Geometry, right: Geometry) -> Integer;
}
define_sql_function! {
    /// Tests if two geometries represent the same geometry and have points in the same directional order.
    #[sql_name="ST_OrderingEquals"]
    fn st_ordering_equals(left: Geometry, right: Geometry) -> Bool;
}
define_sql_function! {
    /// Tests if two geometries intersect and have the same dimension, but are not completely contained by each other.
    #[sql_name="ST_Overlaps"]
    fn st_overlaps(left: Geometry, right: Geometry) -> Bool;
}
define_sql_function! {
    /// Tests if two geometries have a topological relationship matching an Intersection Matrix pattern.
    #[sql_name="ST_Relate"]
    fn st_relate_check(left: Geometry, right: Geometry, intersection_matrix_mattern: Text) -> Bool;
}
define_sql_function! {
    /// Computes Intersection Matrix of two geometries.
    #[sql_name="ST_Relate"]
    fn st_relate(left: Geometry, right: Geometry) -> Text;
}
define_sql_function! {
    /// Computes Intersection Matrix of two geometries. The boundary node rule code is: 1: OGC/MOD2, 2: Endpoint, 3: MultivalentEndpoint, 4: MonovalentEndpoint.
    #[sql_name="ST_Relate"]
    fn st_relate_bnr(left: Geometry, right: Geometry, boundary_node_rule: Integer) -> Text;
}
define_sql_function! {
    /// Tests if a DE-9IM Intersection Matrix matches an Intersection Matrix pattern
    #[sql_name="ST_RelateMatch"]
    fn st_relate_match(intersection_matrix: Text, intersection_matrix_pattern: Text) -> Bool;
}
define_sql_function! {
    /// Tests if two geometries have at least one point in common, but their interiors do not intersect.
    #[sql_name="ST_Touches"]
    fn st_touches(left: Geometry, right: Geometry) -> Bool;
}
define_sql_function! {
    /// Tests if no points of A lie in the exterior of B, and A and B have at least one interior point in common.
    #[sql_name="ST_Within"]
    fn st_within(left: Geometry, right: Geometry) -> Bool;
}
define_sql_function! {
    /// Tests if A and B are within a given distance.
    #[sql_name="ST_DWithin"]
    fn st_d_within<G: GeoType>(left: G, right: G, distance: Double) -> Bool;
}
define_sql_function! {
    /// Computes a geometry covering all points within a given distance from a geometry.
    #[sql_name="ST_Buffer"]
    fn st_buffer<G: GeoType>(geometry: G, radius_of_buffer: Double, buffer_style_parameters: Text) -> G;
}
define_sql_function! {
    /// Returns a modified geometry having no segment longer than the given max_segment_length. Distance computation is
    /// performed in 2d only. For geometry, length units are in units of spatial reference. For geography, units are in
    /// meters.
    #[sql_name="ST_Segmentize"]
    fn st_segmentize<G: GeoType>(geometry: G, max_segment_length: Double) -> G;
}
define_sql_function! {
    /// Divides geometry into parts until a part can be represented using no more than max_vertices.
    #[sql_name="ST_Subdivide"]
    fn st_subdivide<G: GeoType>(geometry: G, max_vertices: Integer, grid_size: Float8) -> G;
}

//Geometry Accessors****************************************************************
define_sql_function! {
    /// The inherent dimension of this Geometry object, which must be less than or equal to the coordinate dimension.
    #[sql_name="ST_Dimension"]
    fn st_dimensions(geometry: Geometry) -> Integer;
}
define_sql_function! {
    /// Returns the last point of a LINESTRING geometry as a POINT.
    #[sql_name="ST_EndPoint"]
    fn st_end_point(geometry: Geometry) -> Nullable<Geometry>;
}
define_sql_function! {
    /// Returns a geometry representing the double precision (float8) bounding box of the supplied geometry.
    #[sql_name="ST_Envelope"]
    fn st_envelope(geometry: Geometry) -> Geometry;
}
define_sql_function! {
    /// Returns a line string representing the exterior ring of the POLYGON geometry. Return NULL if the geometry is not a polygon. Will not work with MULTIPOLYGON.
    #[sql_name="ST_ExteriorRing"]
    fn st_exterior_ring(geometry: Geometry) -> Nullable<Geometry>;
}
define_sql_function! {
    /// Return the 1-based Nth geometry if the geometry is a GEOMETRYCOLLECTION, (MULTI)POINT, (MULTI)LINESTRING, MULTICURVE or (MULTI)POLYGON, POLYHEDRALSURFACE Otherwise, return NULL.
    #[sql_name="ST_GeometryN"]
    fn st_geometry_n(geometry: Geometry, n: Integer) -> Nullable<Geometry>;
}
define_sql_function! {
    /// Return the Nth interior linestring ring of the polygon geometry. Return NULL if the geometry is not a polygon or the given N is out of range.
    #[sql_name="ST_InteriorRingN"]
    fn st_interior_ring_n(geometry: Geometry, n: Integer) -> Nullable<Geometry>;
}
define_sql_function! {
    /// Returns TRUE if the LINESTRING's start and end points are coincident. For Polyhedral surface is closed (volumetric). 
    #[sql_name="ST_IsClosed"]
    fn st_is_closed(geometry: Geometry) -> Bool;
}
define_sql_function! {
    /// Returns TRUE if the argument is a collection (MULTI*, GEOMETRYCOLLECTION, ...)
    #[sql_name="ST_IsCollection"]
    fn st_is_collection(geometry: Geometry) -> Bool;
}
define_sql_function! {
    /// Returns true if this Geometry is an empty geometrycollection, polygon, point etc.
    #[sql_name="ST_IsEmpty"]
    fn st_is_empty(geometry: Geometry) -> Bool;
}
define_sql_function! {
    /// Returns TRUE if this LINESTRING is both closed and simple.
    #[sql_name="ST_IsRing"]
    fn st_is_ring(geometry: Geometry) -> Bool;
}
define_sql_function! {
    /// Returns (TRUE) if this Geometry has no anomalous geometric points, such as self intersection or self tangency.
    #[sql_name="ST_IsSimple"]
    fn st_is_simple(geometry: Geometry) -> Bool;
}
define_sql_function! {
    /// Returns true if the ST_Geometry is well formed.
    #[sql_name="ST_IsValid"]
    fn st_is_valid(geometry: Geometry) -> Bool;
}
define_sql_function! {
    /// Returns true if the ST_Geometry is well formed.
    #[sql_name="ST_IsValid"]
    fn st_is_valid_flags(geometry: Geometry, flags: Integer) -> Bool;
}
define_sql_function! {
    /// Returns text stating if a geometry is valid or not and if not valid, a reason why.
    #[sql_name="ST_IsValidReason"]
    fn st_is_valid_reason(geometry: Geometry) -> Text;
}
define_sql_function! {
    /// Returns text stating if a geometry is valid or not and if not valid, a reason why.
    #[sql_name="ST_IsValidReason"]
    fn st_is_valid_reason_flags(geometry: Geometry, flags: Integer) -> Text;
}
define_sql_function! {
    /// Return the M coordinate of the point, or NULL if not available. Input must be a point.
    #[sql_name="ST_M"]
    fn st_m(geometry: Geometry) -> Nullable<Double>;
}
define_sql_function! {
    /// Returns coordinate dimension of the geometry as a small int. Values are: 2,3 or 4.
    #[sql_name="ST_NDims"]
    fn st_n_dims(geometry: Geometry) -> SmallInt;
}
define_sql_function! {
    /// Return the number of points (vertexes) in a geometry. 
    #[sql_name="ST_NPoints"]
    fn st_n_points(geometry: Geometry) -> Integer;
}
define_sql_function! {
    /// If the geometry is a polygon or multi-polygon returns the number of rings.
    #[sql_name="ST_NRings"]
    fn st_n_rings(geometry: Geometry) -> Integer;
}
define_sql_function! {
    /// If geometry is a GEOMETRYCOLLECTION (or MULTI*) return the number of geometries, for single geometries will return 1.
    #[sql_name="ST_NumGeometries"]
    fn st_num_geometries(geometry: Geometry) -> Integer;
}
define_sql_function! {
    /// Return the number of interior rings of the a polygon in the geometry. This will work with POLYGON and return NULL for a MULTIPOLYGON type or any other type.
    #[sql_name="ST_NumInteriorRings"]
    fn st_num_interior_rings(geometry: Geometry) -> Nullable<Integer>;
}
define_sql_function! {
    /// Return the number of interior rings of the first polygon in the geometry. Synonym to ST_NumInteriorRings.
    #[sql_name="ST_NumInteriorRing"]
    fn st_num_interior_ring(geometry: Geometry) -> Nullable<Integer>;
}
define_sql_function! {
    /// Return the number of faces on a Polyhedral Surface. Will return null for non-polyhedral geometries.
    #[sql_name="ST_NumPatches"]
    fn st_num_patches(geometry: Geometry) -> Nullable<Integer>;
}
define_sql_function! {
    /// Return the number of points in an ST_LineString or ST_CircularString value.
    #[sql_name="ST_NumPoints"]
    fn st_num_points(geometry: Geometry) -> Nullable<Integer>;
}
define_sql_function! {
    /// Return the 1-based Nth geometry (face) if the geometry is a POLYHEDRALSURFACE, POLYHEDRALSURFACEM. Otherwise, return NULL.
    #[sql_name="ST_PatchN"]
    fn st_patch_n(geometry: Geometry, n: Integer) -> Nullable<Geometry>;
}
define_sql_function! {
    /// Return the Nth point in the first linestring or circular linestring in the geometry. Return NULL if there is no linestring in the geometry.
    #[sql_name="ST_PointN"]
    fn st_point_n(geometry: Geometry, n: Integer) -> Nullable<Geometry>;
}
define_sql_function! {
    /// Returns the first point of a LINESTRING geometry as a POINT.
    #[sql_name="ST_StartPoint"]
    fn st_start_point(geometry: Geometry) -> Nullable<Geometry>;
}
define_sql_function! {
    /// Returns a text summary of the contents of the geometry.
    #[sql_name="ST_Summary"]
    fn st_summary<G: GeoType>(geometry: G) -> Text;
}
define_sql_function! {
    /// Return the X coordinate of the point, or NULL if not available. Input must be a point.
    #[sql_name="ST_X"]
    fn st_x(geometry: Geometry) -> Double;
}
define_sql_function! {
    /// Return the Y coordinate of the point, or NULL if not available. Input must be a point.
    #[sql_name="ST_Y"]
    fn st_y(geometry: Geometry) -> Double;
}
define_sql_function! {
    /// Return the Z coordinate of the point, or NULL if not available. Input must be a point.
    #[sql_name="ST_Z"]
    fn st_z(geometry: Geometry) -> Nullable<Double>;
}
define_sql_function! {
    /// Returns ZM (dimension semantic) flag of the geometries as a small int. Values are: 0=2d, 1=3dm, 2=3dz, 3=4d.
    #[sql_name="ST_Zmflag"]
    fn st_zmflag(geometry: Geometry) -> SmallInt;
}
