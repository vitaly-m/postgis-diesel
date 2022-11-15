use diesel::expression::AsExpression;
use diesel::sql_types::SqlType;
use diesel::Expression;
use crate::sql_types::Geography;

diesel::infix_operator!(BBIntersects2D, " && ");
diesel::infix_operator!(BBOverlapsOrLeft, " &< ");
diesel::infix_operator!(BBOverlapsOrBelow, " &<| ");
diesel::infix_operator!(BBOverlapsOrRight, " &> ");
diesel::infix_operator!(BBStrictlyLeft, " << ");
diesel::infix_operator!(BBStrictlyBelow, " <<| ");
diesel::infix_operator!(GSame, " = ");
diesel::infix_operator!(BBStrictlyRight, " >> ");
diesel::infix_operator!(BBContainedBy, " @ ");
diesel::infix_operator!(BBOverlapsOrAbove, " |&> ");
diesel::infix_operator!(BBStrictlyAbove, " |>> ");
diesel::infix_operator!(BBContains, " ~ ");
diesel::infix_operator!(BBSame, " ~= ");

diesel::infix_operator!(Distance2d, " <-> ");
diesel::infix_operator!(Distance3dTrajectories, " <-> ");
diesel::infix_operator!(Distance2BBs, " <#> ");
diesel::infix_operator!(DistanceNdCentroidsBBs, " <<->> ");
diesel::infix_operator!(DistanceNdBBs, " <<#>> ");

sql_function! {
    /// ST_Intersects returns TRUE iff the intersection of the geometries or geographies is non-empty.
    /// 
    /// It is much faster than computing said intersection.
    fn st_intersects(left: Geography, right: Geography) -> Bool;
}
pub type StIntersects<ExprLeft, ExprRight> = st_intersects::HelperType<ExprLeft, ExprRight>;

/// The @ operator returns TRUE if the bounding box of geometry A is completely contained by the bounding box of geometry B.
pub fn contained_by<T, U>(left: T, right: U) -> BBContainedBy<T, U::Expression>
where
    T: Expression,
    <T as diesel::Expression>::SqlType: SqlType,
    U: AsExpression<T::SqlType>,
{
    BBContainedBy::new(left, right.as_expression())
}

/// The ~ operator returns TRUE if the bounding box of geometry A completely contains the bounding box of geometry B.
pub fn contains<T, U>(left: T, right: U) -> BBContains<T, U::Expression>
where
    T: Expression,
    <T as diesel::Expression>::SqlType: SqlType,
    U: AsExpression<T::SqlType>,
{
    BBContains::new(left, right.as_expression())
}

/// The && operator returns TRUE if the 2D bounding box of geometry A intersects the 2D bounding box of geometry B.
pub fn intersects_2d<T, U>(left: T, right: U) -> BBIntersects2D<T, U::Expression>
where
    T: Expression,
    <T as diesel::Expression>::SqlType: SqlType,
    U: AsExpression<T::SqlType>,
{
    BBIntersects2D::new(left, right.as_expression())
}

/// The &< operator returns TRUE if the bounding box of geometry A overlaps or is to the left of the bounding box of geometry B, or more accurately, overlaps or is NOT to the right of the bounding box of geometry B.
pub fn overlaps_or_left<T, U>(left: T, right: U) -> BBOverlapsOrLeft<T, U::Expression>
where
    T: Expression,
    <T as diesel::Expression>::SqlType: SqlType,
    U: AsExpression<T::SqlType>,
{
    BBOverlapsOrLeft::new(left, right.as_expression())
}

/// The &<| operator returns TRUE if the bounding box of geometry A overlaps or is below of the bounding box of geometry B, or more accurately, overlaps or is NOT above the bounding box of geometry B.
pub fn overlaps_or_below<T, U>(left: T, right: U) -> BBOverlapsOrBelow<T, U::Expression>
where
    T: Expression,
    <T as diesel::Expression>::SqlType: SqlType,
    U: AsExpression<T::SqlType>,
{
    BBOverlapsOrBelow::new(left, right.as_expression())
}

/// The &> operator returns TRUE if the bounding box of geometry A overlaps or is to the right of the bounding box of geometry B, or more accurately, overlaps or is NOT to the left of the bounding box of geometry B.
pub fn overlaps_or_right<T, U>(left: T, right: U) -> BBOverlapsOrRight<T, U::Expression>
where
    T: Expression,
    <T as diesel::Expression>::SqlType: SqlType,
    U: AsExpression<T::SqlType>,
{
    BBOverlapsOrRight::new(left, right.as_expression())
}

/// The |&> operator returns TRUE if the bounding box of geometry A overlaps or is above the bounding box of geometry B, or more accurately, overlaps or is NOT below the bounding box of geometry B.
pub fn overlaps_or_above<T, U>(left: T, right: U) -> BBOverlapsOrAbove<T, U::Expression>
where
    T: Expression,
    <T as diesel::Expression>::SqlType: SqlType,
    U: AsExpression<T::SqlType>,
{
    BBOverlapsOrAbove::new(left, right.as_expression())
}

/// The << operator returns TRUE if the bounding box of geometry A is strictly to the left of the bounding box of geometry B.
pub fn strictly_left<T, U>(left: T, right: U) -> BBStrictlyLeft<T, U::Expression>
where
    T: Expression,
    <T as diesel::Expression>::SqlType: SqlType,
    U: AsExpression<T::SqlType>,
{
    BBStrictlyLeft::new(left, right.as_expression())
}

/// The <<| operator returns TRUE if the bounding box of geometry A is strictly below the bounding box of geometry B.
pub fn strictly_below<T, U>(left: T, right: U) -> BBStrictlyBelow<T, U::Expression>
where
    T: Expression,
    <T as diesel::Expression>::SqlType: SqlType,
    U: AsExpression<T::SqlType>,
{
    BBStrictlyBelow::new(left, right.as_expression())
}

/// The >> operator returns TRUE if the bounding box of geometry A is strictly to the right of the bounding box of geometry B.
pub fn strictly_right<T, U>(left: T, right: U) -> BBStrictlyRight<T, U::Expression>
where
    T: Expression,
    <T as diesel::Expression>::SqlType: SqlType,
    U: AsExpression<T::SqlType>,
{
    BBStrictlyRight::new(left, right.as_expression())
}

/// The |>> operator returns TRUE if the bounding box of geometry A is strictly above the bounding box of geometry B.
pub fn strictly_above<T, U>(left: T, right: U) -> BBStrictlyAbove<T, U::Expression>
where
    T: Expression,
    <T as diesel::Expression>::SqlType: SqlType,
    U: AsExpression<T::SqlType>,
{
    BBStrictlyAbove::new(left, right.as_expression())
}

/// The = operator returns TRUE if the coordinates and coordinate order geometry/geography A are the same as the coordinates and coordinate order of geometry/geography B.
pub fn g_same<T, U>(left: T, right: U) -> GSame<T, U::Expression>
where
    T: Expression,
    <T as diesel::Expression>::SqlType: SqlType,
    U: AsExpression<T::SqlType>,
{
    GSame::new(left, right.as_expression())
}

/// The ~= operator returns TRUE if the bounding box of geometry/geography A is the same as the bounding box of geometry/geography B.
pub fn bb_same<T, U>(left: T, right: U) -> BBSame<T, U::Expression>
where
    T: Expression,
    <T as diesel::Expression>::SqlType: SqlType,
    U: AsExpression<T::SqlType>,
{
    BBSame::new(left, right.as_expression())
}

/// The <-> operator returns the 2D distance between A and B.
pub fn distance_2d<T, U>(left: T, right: U) -> Distance2d<T, U::Expression>
where
    T: Expression,
    <T as diesel::Expression>::SqlType: SqlType,
    U: AsExpression<T::SqlType>,
{
    Distance2d::new(left, right.as_expression())
}

/// The |=| operator returns the 3D distance between two trajectories.
pub fn distance_3d_trajectories<T, U>(left: T, right: U) -> Distance3dTrajectories<T, U::Expression>
where
    T: Expression,
    <T as diesel::Expression>::SqlType: SqlType,
    U: AsExpression<T::SqlType>,
{
    Distance3dTrajectories::new(left, right.as_expression())
}

/// The <#> operator returns the 2D distance between A and B bounding boxes.
pub fn distance_2d_bbs<T, U>(left: T, right: U) -> Distance2BBs<T, U::Expression>
where
    T: Expression,
    <T as diesel::Expression>::SqlType: SqlType,
    U: AsExpression<T::SqlType>,
{
    Distance2BBs::new(left, right.as_expression())
}

/// The <<->> operator returns the n-D distance between the centroids of A and B bounding boxes.
pub fn distance_nd_centroids_bbs<T, U>(
    left: T,
    right: U,
) -> DistanceNdCentroidsBBs<T, U::Expression>
where
    T: Expression,
    <T as diesel::Expression>::SqlType: SqlType,
    U: AsExpression<T::SqlType>,
{
    DistanceNdCentroidsBBs::new(left, right.as_expression())
}

/// The <<#>> operator returns the n-D distance between A and B bounding boxes.
pub fn distance_nd_bbs<T, U>(left: T, right: U) -> DistanceNdBBs<T, U::Expression>
where
    T: Expression,
    <T as diesel::Expression>::SqlType: SqlType,
    U: AsExpression<T::SqlType>,
{
    DistanceNdBBs::new(left, right.as_expression())
}
