//! Defines the operation traits exposed by Canyon-SQL query builders.
//!
//! Each trait groups the operations available for a specific SQL statement,
//! while [`QueryBuilderOps`] contains the behaviour shared by all builders.

use crate::query::bounds::{FieldIdentifier, FieldValueIdentifier};
use crate::query::operators::Operator;
use crate::query::parameters::QueryParameter;
use crate::query::query::Query;
use crate::query::querybuilder::syntax::column::ColumnRef;
use crate::query::querybuilder::syntax::table_metadata::TableMetadata;
use std::error::Error;

/// Operations supported by a delete query builder.
///
/// Delete queries currently require no statement-specific operations beyond
/// those provided by [`QueryBuilderOps`].
pub trait DeleteQueryBuilderOps<'a>: QueryBuilderOps<'a> {}

/// Operations supported by an update query builder.
pub trait UpdateQueryBuilderOps<'a>: QueryBuilderOps<'a> {
    /// Defines the columns assigned by the generated `SET` clause.
    ///
    /// This method only registers column references. It does not collect the
    /// values corresponding to the generated placeholders.
    ///
    /// The caller is therefore responsible for supplying matching parameters
    /// when the query is executed.
    fn set<I: Into<ColumnRef<'a>>>(
        self,
        columns: Vec<I>,
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'a>>
    where
        Self: Sized;

    /// Defines the `SET` clause and collects one update value for each column.
    ///
    /// Each tuple contains the target column identifier and the parameter value
    /// assigned to it.
    fn set_values<Z, Q>(
        self,
        columns: &'a [(Z, Q)],
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'a>>
    where
        Z: FieldIdentifier + Into<ColumnRef<'a>> + Clone,
        Q: QueryParameter,
        Self: Sized;
}

/// Operations supported by an insert query builder.
pub trait InsertQueryBuilderOps<'a>: QueryBuilderOps<'a> {
    /// Defines the columns targeted by the insert statement.
    ///
    /// When omitted, the generated statement does not include an explicit
    /// column list.
    fn with_columns<I: Into<ColumnRef<'a>>>(self, columns: Vec<I>) -> Self;

    /// Collects the values inserted by the statement.
    ///
    /// The generated placeholder count must match the number of configured
    /// insert columns when an explicit column list is present.
    fn with_values<Q>(self, values: &'a [Q]) -> Result<Self, Box<dyn Error + Send + Sync + 'a>>
    where
        Q: QueryParameter,
        Self: Sized;

    /// Defines the columns returned after a successful insert.
    ///
    /// The resulting SQL is emitted according to the target database dialect,
    /// such as `RETURNING` or `OUTPUT INSERTED`.
    fn returning(self, columns: Vec<impl Into<ColumnRef<'a>>>) -> Self;
}

/// Operations supported by a select query builder.
pub trait SelectQueryBuilderOps<'a>: QueryBuilderOps<'a> {
    /// Defines the columns projected by the select statement.
    ///
    /// When omitted, the query projects all columns using `SELECT *`.
    fn with_columns<I: Into<ColumnRef<'a>>>(self, columns: Vec<I>) -> Self;

    /// Marks the select statement as `DISTINCT`.
    fn with_distinct(self) -> Self;

    /// Changes the select projection to a row count.
    fn count(self) -> Self;

    /// Adds a `LEFT JOIN` to the select statement.
    ///
    /// `join_table` identifies the joined table, while `col1` and `col2`
    /// define the two column references used by the join condition.
    ///
    /// The order of the column references does not affect the generated
    /// equality condition.
    fn left_join(
        self,
        join_table: impl Into<TableMetadata<'a>>,
        col1: impl Into<ColumnRef<'a>>,
        col2: impl Into<ColumnRef<'a>>,
    ) -> Self;

    /// Adds an `INNER JOIN` to the select statement.
    ///
    /// `join_table` identifies the joined table, while `col1` and `col2`
    /// define the two column references used by the join condition.
    ///
    /// The order of the column references does not affect the generated
    /// equality condition.
    fn inner_join(
        self,
        join_table: impl Into<TableMetadata<'a>>,
        col1: impl Into<ColumnRef<'a>>,
        col2: impl Into<ColumnRef<'a>>,
    ) -> Self;

    /// Adds a `RIGHT JOIN` to the select statement.
    ///
    /// `join_table` identifies the joined table, while `col1` and `col2`
    /// define the two column references used by the join condition.
    ///
    /// The order of the column references does not affect the generated
    /// equality condition.
    fn right_join(
        self,
        join_table: impl Into<TableMetadata<'a>>,
        col1: impl Into<ColumnRef<'a>>,
        col2: impl Into<ColumnRef<'a>>,
    ) -> Self;

    /// Adds a `FULL JOIN` to the select statement.
    ///
    /// `join_table` identifies the joined table, while `col1` and `col2`
    /// define the two column references used by the join condition.
    ///
    /// The order of the column references does not affect the generated
    /// equality condition.
    fn full_join(
        self,
        join_table: impl Into<TableMetadata<'a>>,
        col1: impl Into<ColumnRef<'a>>,
        col2: impl Into<ColumnRef<'a>>,
    ) -> Self;

    /// Adds an `ORDER BY` clause for the specified column.
    ///
    /// When `desc` is `true`, descending order is used. Otherwise, the
    /// generated ordering is ascending.
    fn order_by<Z: FieldIdentifier + Into<ColumnRef<'a>>>(self, order_by: Z, desc: bool) -> Self;
}

/// Common operations supported by every query builder.
///
/// Statement-specific builders expose this shared filtering and build API,
/// while traits such as [`SelectQueryBuilderOps`], [`InsertQueryBuilderOps`],
/// and [`UpdateQueryBuilderOps`] add operations that only apply to their
/// corresponding SQL statement.
///
/// Implementations collect structured query data and parameters. SQL generation
/// is deferred until [`Self::build`] consumes the builder and emits a [`Query`]
/// for the configured database dialect.
pub trait QueryBuilderOps<'a> {
    /// Consumes the builder and generates the final query.
    ///
    /// The returned [`Query`] contains both the emitted SQL statement and the
    /// parameters collected while constructing it.
    fn build(self) -> Result<Query<'a>, Box<dyn Error + Send + Sync + 'a>>;

    /// Adds a `WHERE` condition without collecting a parameter value.
    ///
    /// `column` identifies the left-hand side of the condition and `op`
    /// defines the comparison operator.
    ///
    /// The condition emits a placeholder whose corresponding value must be
    /// supplied separately.
    fn r#where<I: Into<ColumnRef<'a>>>(self, column: I, op: Operator) -> Self;

    /// Adds a `WHERE` condition and collects its parameter value.
    ///
    /// The [`FieldValueIdentifier`] provides both the target column and the
    /// value bound to the generated placeholder.
    fn where_value<Z: FieldValueIdentifier>(self, column: &'a Z, op: Operator) -> Self;

    /// Adds an `AND` condition and collects its parameter value.
    ///
    /// The [`FieldValueIdentifier`] provides both the target column and the
    /// value bound to the generated placeholder.
    fn and<Z: FieldValueIdentifier>(self, column: &'a Z, op: Operator) -> Self;

    /// Adds an `AND <column> IN (...)` condition.
    ///
    /// One placeholder and one collected query parameter are generated for
    /// every element in `values`.
    fn and_values_in<'b, Z, Q>(
        self,
        column: Z,
        values: &'a [Q],
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'b>>
    where
        Z: FieldIdentifier,
        Q: QueryParameter,
        Self: Sized;

    /// Adds an `OR <column> IN (...)` condition.
    ///
    /// One placeholder and one collected query parameter are generated for
    /// every element in `values`.
    fn or_values_in<'b, Z, Q>(
        self,
        r#or: Z,
        values: &'a [Q],
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'b>>
    where
        Z: FieldIdentifier,
        Q: QueryParameter,
        Self: Sized;

    /// Adds an `OR` condition and collects its parameter value.
    ///
    /// The [`FieldValueIdentifier`] provides both the target column and the
    /// value bound to the generated placeholder.
    fn or<Z: FieldValueIdentifier>(self, column: &'a Z, op: Operator) -> Self;
}