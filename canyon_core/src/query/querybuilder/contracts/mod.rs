//! Contains the elements that makes part of the formal declaration
//! of the behaviour of the Canyon-SQL QueryBuilder

use crate::query::bounds::{FieldIdentifier, FieldValueIdentifier};
use crate::query::operators::Operator;
use crate::query::parameters::QueryParameter;
use crate::query::querybuilder::syntax::column::ColumnRef;
use crate::query::querybuilder::syntax::table_metadata::TableMetadata;
use std::error::Error;
use crate::query::query::Query;

pub trait DeleteQueryBuilderOps<'a>: QueryBuilderOps<'a> {}

pub trait UpdateQueryBuilderOps<'a>: QueryBuilderOps<'a> {
    /// Creates an SQL `SET` clause by specifying the columns that must be updated in the sentence,
    /// but without adding any [`QueryParameter`] value to the internal querybuilder.
    ///
    /// Is it the responsibility of the callee to pass the query values that will match the generated
    /// sql placeholders
    ///
    /// Note: If there's values already on the querybuilder, and the only placeholders api is called,
    /// UB (provisionally) will occur, since we're refactoring the API's and these are subject to change
    /// at any time while in the v0.x.x
    fn set<I: Into<ColumnRef<'a>>>(
        self,
        columns: Vec<I>,
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'a>>
    where
        Self: Sized;

    /// Similar to [`Self::set`] but storing the underlying update values for each column in the
    /// internal values collection of the [`crate::query::querybuilder::QueryBuilder`]
    fn set_values<Z, Q>(
        self,
        columns: &'a [(Z, Q)],
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'a>>
    where
        Z: FieldIdentifier + Into<ColumnRef<'a>> + Clone,
        Q: QueryParameter,
        Self: Sized;
}

pub trait SelectQueryBuilderOps<'a>: QueryBuilderOps<'a> {
    /// Adds the column names that must be added to the query in order to retrieve the correct mapped fields
    /// If this method isn't invoked, the querybuilder will create a SELECT * FROM query
    fn with_columns<I: Into<ColumnRef<'a>>>(self, columns: Vec<I>) -> Self;

    /// Adds a `DISTINCT` SQL statement to the underlying `Sql Statement` held by the [`QueryBuilder`]
    fn with_distinct(self) -> Self;

    /// Adds a `COUNT` SQL statement to the underlying `Sql Statement` held by the [`QueryBuilder`]
    fn count(self) -> Self;

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
        join_table: impl Into<TableMetadata<'a>>,
        col1: impl Into<ColumnRef<'a>>,
        col2: impl Into<ColumnRef<'a>>,
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
        join_table: impl Into<TableMetadata<'a>>,
        col1: impl Into<ColumnRef<'a>>,
        col2: impl Into<ColumnRef<'a>>,
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
        join_table: impl Into<TableMetadata<'a>>,
        col1: impl Into<ColumnRef<'a>>,
        col2: impl Into<ColumnRef<'a>>,
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
        join_table: impl Into<TableMetadata<'a>>,
        col1: impl Into<ColumnRef<'a>>,
        col2: impl Into<ColumnRef<'a>>,
    ) -> Self;

    /// Generates a `ORDER BY` SQL clause for constraint the query.
    ///
    /// * `order_by` - A [`FieldIdentifier`] that will provide the target  column name
    /// * `desc` - a boolean indicating if the generated `ORDER_BY` must be in ascending or descending order
    fn order_by<Z: FieldIdentifier + Into<ColumnRef<'a>>>(self, order_by: Z, desc: bool) -> Self;
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
    /// Builds the final [`Query`] by consuming the querybuilder, and returning the generated SQL statement and the collected parameters as a tuple.
    fn build(self) -> Result<Query<'a>, Box<dyn Error + Send + Sync + 'a>>;

    /// Generates a `WHERE` SQL clause for constraint the query.
    ///
    /// * `column` - An [`&str`] that will provide the target column name
    /// * `op` - Any element that implements [`Operator`] for create the comparison
    ///   or equality binary operator
    ///
    ///  It will generate a SQL statement with the where constraint value generated as a placeholder,
    /// depending on the underlying database driver and the number of elements already added to the
    /// querybuilder
    fn r#where<I: Into<ColumnRef<'a>>>(self, column: I, op: Operator) -> Self;

    /// Generates a `WHERE` SQL clause for constraint the query.
    ///
    /// * `column` - A [`FieldValueIdentifier`] that will provide the target
    ///   column name and the value for the filter
    /// * `op` - Any element that implements [`Operator`] for create the comparison
    ///   or equality binary operator
    fn where_value<Z: FieldValueIdentifier>(self, column: &'a Z, op: Operator) -> Self;

    /// Generates an `AND` SQL clause for constraint the query.
    ///
    /// * `column` - A [`FieldValueIdentifier`] that will provide the target
    ///   column name and the value for the filter
    /// * `op` - Any element that implements [`Operator`] for create the comparison
    ///   or equality binary operator
    fn and<Z: FieldValueIdentifier>(self, column: &'a Z, op: Operator) -> Self;

    /// Generates an `AND` SQL clause for constraint the query that's being constructed
    ///
    /// * `column` - A [`FieldIdentifier`] that will provide the target
    ///   column name for the filter, based on the variant that represents
    ///   the field name that maps the targeted column name
    /// * `values` - An array of [`QueryParameter`] with the values to filter
    ///   inside the `IN` operator
    fn and_values_in<'b, Z, Q>(
        self,
        column: Z,
        values: &'a [Q],
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'b>>
    where
        Z: FieldIdentifier,
        Q: QueryParameter,
        Self: Sized;

    /// Generates an `OR` SQL clause for constraint the query that will create
    /// the filter in conjunction with an `IN` operator that will ac
    ///
    /// * `column` - A [`FieldIdentifier`] that will provide the target
    ///   column name for the filter, based on the variant that represents
    ///   the field name that maps the targeted column name
    /// * `values` - An array of [`QueryParameter`] with the values to filter
    ///   inside the `IN` operator
    fn or_values_in<'b, Z, Q>(
        self,
        r#or: Z,
        values: &'a [Q],
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'b>>
    where
        Z: FieldIdentifier,
        Q: QueryParameter,
        Self: Sized;

    /// Generates an `OR` SQL clause for constraint the query.
    ///
    /// * `column` - A [`FieldValueIdentifier`] that will provide the target
    ///   column name and the value for the filter
    /// * `op` - Any element that implements [`Operator`] for create the comparison
    ///   or equality binary operator
    fn or<Z: FieldValueIdentifier>(self, column: &'a Z, op: Operator) -> Self;
}
