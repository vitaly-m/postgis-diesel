use diesel::expression::AsExpression;
use diesel::Expression;

diesel_infix_operator!(BBIntersects2D, " && ");
diesel_infix_operator!(BBOverlapsOrLeft, " &< ");
diesel_infix_operator!(BBOverlapsOrBelow, " &<| ");
diesel_infix_operator!(BBOverlapsOrRight, " &> ");
diesel_infix_operator!(BBStrictlyLeft, " << ");
diesel_infix_operator!(BBStrictlyBelow, " <<| ");
diesel_infix_operator!(GSame, " = ");
diesel_infix_operator!(BBStrictlyRight, " >> ");
diesel_infix_operator!(BBContainedBy, " @ ");
diesel_infix_operator!(BBOverlapsOrAbove, " |&> ");
diesel_infix_operator!(BBStrictlyAbove, " |>> ");
diesel_infix_operator!(BBContains, " ~ ");
diesel_infix_operator!(BBSame, " ~= ");

/// The @ operator returns TRUE if the bounding box of geometry A is completely contained by the bounding box of geometry B.
pub fn contained_by<T, U>(left: T, right: U) -> BBContainedBy<T, U::Expression>
where
    T: Expression,
    U: AsExpression<T::SqlType>,
{
    BBContainedBy::new(left, right.as_expression())
}

/// The ~ operator returns TRUE if the bounding box of geometry A completely contains the bounding box of geometry B.
pub fn contains<T, U>(left: T, right: U) -> BBContains<T, U::Expression>
where
    T: Expression,
    U: AsExpression<T::SqlType>,
{
    BBContains::new(left, right.as_expression())
}

/// The && operator returns TRUE if the 2D bounding box of geometry A intersects the 2D bounding box of geometry B.
pub fn intersects_2d<T, U>(left: T, right: U) -> BBIntersects2D<T, U::Expression>
where
    T: Expression,
    U: AsExpression<T::SqlType>,
{
    BBIntersects2D::new(left, right.as_expression())
}

/// The &< operator returns TRUE if the bounding box of geometry A overlaps or is to the left of the bounding box of geometry B, or more accurately, overlaps or is NOT to the right of the bounding box of geometry B.
pub fn overlaps_or_left<T, U>(left: T, right: U) -> BBOverlapsOrLeft<T, U::Expression>
where
    T: Expression,
    U: AsExpression<T::SqlType>,
{
    BBOverlapsOrLeft::new(left, right.as_expression())
}

/// The &<| operator returns TRUE if the bounding box of geometry A overlaps or is below of the bounding box of geometry B, or more accurately, overlaps or is NOT above the bounding box of geometry B.
pub fn overlaps_or_below<T, U>(left: T, right: U) -> BBOverlapsOrBelow<T, U::Expression>
where
    T: Expression,
    U: AsExpression<T::SqlType>,
{
    BBOverlapsOrBelow::new(left, right.as_expression())
}

/// The &> operator returns TRUE if the bounding box of geometry A overlaps or is to the right of the bounding box of geometry B, or more accurately, overlaps or is NOT to the left of the bounding box of geometry B.
pub fn overlaps_or_right<T, U>(left: T, right: U) -> BBOverlapsOrRight<T, U::Expression>
where
    T: Expression,
    U: AsExpression<T::SqlType>,
{
    BBOverlapsOrRight::new(left, right.as_expression())
}

/// The |&> operator returns TRUE if the bounding box of geometry A overlaps or is above the bounding box of geometry B, or more accurately, overlaps or is NOT below the bounding box of geometry B.
pub fn overlaps_or_above<T, U>(left: T, right: U) -> BBOverlapsOrAbove<T, U::Expression>
where
    T: Expression,
    U: AsExpression<T::SqlType>,
{
    BBOverlapsOrAbove::new(left, right.as_expression())
}

/// The << operator returns TRUE if the bounding box of geometry A is strictly to the left of the bounding box of geometry B.
pub fn strictly_left<T, U>(left: T, right: U) -> BBStrictlyLeft<T, U::Expression>
where
    T: Expression,
    U: AsExpression<T::SqlType>,
{
    BBStrictlyLeft::new(left, right.as_expression())
}

/// The <<| operator returns TRUE if the bounding box of geometry A is strictly below the bounding box of geometry B.
pub fn strictly_below<T, U>(left: T, right: U) -> BBStrictlyBelow<T, U::Expression>
where
    T: Expression,
    U: AsExpression<T::SqlType>,
{
    BBStrictlyBelow::new(left, right.as_expression())
}

/// The >> operator returns TRUE if the bounding box of geometry A is strictly to the right of the bounding box of geometry B.
pub fn strictly_right<T, U>(left: T, right: U) -> BBStrictlyRight<T, U::Expression>
where
    T: Expression,
    U: AsExpression<T::SqlType>,
{
    BBStrictlyRight::new(left, right.as_expression())
}

/// The |>> operator returns TRUE if the bounding box of geometry A is strictly above the bounding box of geometry B.
pub fn strictly_above<T, U>(left: T, right: U) -> BBStrictlyAbove<T, U::Expression>
where
    T: Expression,
    U: AsExpression<T::SqlType>,
{
    BBStrictlyAbove::new(left, right.as_expression())
}

/// The = operator returns TRUE if the coordinates and coordinate order geometry/geography A are the same as the coordinates and coordinate order of geometry/geography B.
pub fn g_same<T, U>(left: T, right: U) -> GSame<T, U::Expression>
where
    T: Expression,
    U: AsExpression<T::SqlType>,
{
    GSame::new(left, right.as_expression())
}

/// The ~= operator returns TRUE if the bounding box of geometry/geography A is the same as the bounding box of geometry/geography B.
pub fn bb_same<T, U>(left: T, right: U) -> BBSame<T, U::Expression>
where
    T: Expression,
    U: AsExpression<T::SqlType>,
{
    BBSame::new(left, right.as_expression())
}
