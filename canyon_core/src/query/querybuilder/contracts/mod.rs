//! Contains the elements that makes part of the formal declaration
//! of the behaviour of the Canyon-SQL QueryBuilder

use crate::query::bounds::{FieldIdentifier, FieldValueIdentifier, TableMetadata};
use crate::query::operators::Operator;
use crate::query::parameters::QueryParameter;

pub trait DeleteQueryBuilderOps<'a>: QueryBuilderOps<'a> {}

pub trait UpdateQueryBuilderOps<'a>: QueryBuilderOps<'a> {
    /// Creates an SQL `SET` clause to specify the columns that must be updated in the sentence
    fn set<Z, Q>(self, columns: &'a [(Z, Q)]) -> Self
    where
        Z: FieldIdentifier + Clone,
        Q: QueryParameter<'a>;
}

pub trait SelectQueryBuilderOps<'a>: QueryBuilderOps<'a> {
    /// Adds a *LEFT JOIN* SQL statement to the underlying
    /// `Sql Statement` held by the [`QueryBuilder`], where:
    ///
    /// * `join_table` - The table target of the join operation
    /// * `col1` - The left side of the ON operator for the join
    /// * `col2` - The right side of the ON operator for the join
    ///
    /// > Note: The order on the column parameters is irrelevant
    fn left_join(
        self,
        join_table: impl TableMetadata,
        col1: impl FieldIdentifier,
        col2: impl FieldIdentifier,
    ) -> Self;

    /// Adds a *INNER JOIN* SQL statement to the underlying
    /// `Sql Statement` held by the [`QueryBuilder`], where:
    ///
    /// * `join_table` - The table target of the join operation
    /// * `col1` - The left side of the ON operator for the join
    /// * `col2` - The right side of the ON operator for the join
    ///
    /// > Note: The order on the column parameters is irrelevant
    fn inner_join(
        self,
        join_table: impl TableMetadata,
        col1: impl FieldIdentifier,
        col2: impl FieldIdentifier,
    ) -> Self;

    /// Adds a *RIGHT JOIN* SQL statement to the underlying
    /// `Sql Statement` held by the [`QueryBuilder`], where:
    ///
    /// * `join_table` - The table target of the join operation
    /// * `col1` - The left side of the ON operator for the join
    /// * `col2` - The right side of the ON operator for the join
    ///
    /// > Note: The order on the column parameters is irrelevant
    fn right_join(
        self,
        join_table: impl TableMetadata,
        col1: impl FieldIdentifier,
        col2: impl FieldIdentifier,
    ) -> Self;

    /// Adds a *FULL JOIN* SQL statement to the underlying
    /// `Sql Statement` held by the [`QueryBuilder`], where:
    ///
    /// * `join_table` - The table target of the join operation
    /// * `col1` - The left side of the ON operator for the join
    /// * `col2` - The right side of the ON operator for the join
    ///
    /// > Note: The order on the column parameters is irrelevant
    fn full_join(
        self,
        join_table: impl TableMetadata,
        col1: impl FieldIdentifier,
        col2: impl FieldIdentifier,
    ) -> Self;
}

/// The [`QueryBuilder`] trait is the root of a kind of hierarchy
/// on more specific [`super::QueryBuilder`], that are:
///
/// * [`super::SelectQueryBuilder`]
/// * [`super::UpdateQueryBuilder`]
/// * [`super::DeleteQueryBuilder`]
///
/// This trait provides the formal declaration of the behaviour that the
/// implementors must provide in their public interfaces, grouping
/// the common elements between every element down in that
/// hierarchy.
///
/// For example, the [`super::QueryBuilder`] type holds the data
/// necessary for track the SQL sentence while it's being generated
/// thought the fluent builder, and provides the behaviour of
/// the common elements defined in this trait.
///
/// The more concrete types represents a wrapper over a raw
/// [`super::QueryBuilder`], offering all the elements declared
/// in this trait in its public interface, and which implementation
/// only consists of call the same method on the wrapped
/// [`super::QueryBuilder`].
///
/// This allows us to declare in their public interface their
/// specific operations, like, for example, join operations
/// on the [`super::SelectQueryBuilder`], and the usage
/// of the `SET` clause on a [`super::UpdateQueryBuilder`],
/// without mixing types or polluting everything into
/// just one type.
pub trait QueryBuilderOps<'a> {
    /// Returns a read-only reference to the underlying SQL sentence,
    /// with the same lifetime as self
    fn read_sql(&'a self) -> &'a str;

    /// Public interface for append the content of a slice to the end of
    /// the underlying SQL sentence.
    ///
    /// This mutator will allow the user to wire SQL code to the already
    /// generated one
    ///
    /// * `sql` - The [`&str`] to be wired in the SQL
    fn push_sql(self, sql: &str);

    /// Generates a `WHERE` SQL clause for constraint the query.
    ///
    /// * `column` - A [`FieldValueIdentifier`] that will provide the target
    ///   column name and the value for the filter
    /// * `op` - Any element that implements [`Operator`] for create the comparison
    ///   or equality binary operator
    fn r#where<Z: FieldValueIdentifier<'a>>(self, column: Z, op: impl Operator) -> Self;

    /// Generates an `AND` SQL clause for constraint the query.
    ///
    /// * `column` - A [`FieldValueIdentifier`] that will provide the target
    ///   column name and the value for the filter
    /// * `op` - Any element that implements [`Operator`] for create the comparison
    ///   or equality binary operator
    fn and<Z: FieldValueIdentifier<'a>>(self, column: Z, op: impl Operator) -> Self;

    /// Generates an `AND` SQL clause for constraint the query that's being constructed
    ///
    /// * `column` - A [`FieldIdentifier`] that will provide the target
    ///   column name for the filter, based on the variant that represents
    ///   the field name that maps the targeted column name
    /// * `values` - An array of [`QueryParameter`] with the values to filter
    ///   inside the `IN` operator
    fn and_values_in<Z, Q>(self, column: Z, values: &'a [Q]) -> Self
    where
        Z: FieldIdentifier,
        Q: QueryParameter<'a>;

    /// Generates an `OR` SQL clause for constraint the query that will create
    /// the filter in conjunction with an `IN` operator that will ac
    ///
    /// * `column` - A [`FieldIdentifier`] that will provide the target
    ///   column name for the filter, based on the variant that represents
    ///   the field name that maps the targeted column name
    /// * `values` - An array of [`QueryParameter`] with the values to filter
    ///   inside the `IN` operator
    fn or_values_in<Z, Q>(self, r#or: Z, values: &'a [Q]) -> Self
    where
        Z: FieldIdentifier,
        Q: QueryParameter<'a>;

    /// Generates an `OR` SQL clause for constraint the query.
    ///
    /// * `column` - A [`FieldValueIdentifier`] that will provide the target
    ///   column name and the value for the filter
    /// * `op` - Any element that implements [`Operator`] for create the comparison
    ///   or equality binary operator
    fn or<Z: FieldValueIdentifier<'a>>(self, column: Z, op: impl Operator) -> Self;

    /// Generates a `ORDER BY` SQL clause for constraint the query.
    ///
    /// * `order_by` - A [`FieldIdentifier`] that will provide the target  column name
    /// * `desc` - a boolean indicating if the generated `ORDER_BY` must be in ascending or descending order
    fn order_by<Z: FieldIdentifier>(self, order_by: Z, desc: bool) -> Self;
}
