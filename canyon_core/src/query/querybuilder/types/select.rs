use crate::{
    connection::database_type::DatabaseType,
    query::{
        bounds::{FieldIdentifier, FieldValueIdentifier},
        operators::Operator,
        parameters::QueryParameter,
        query::Query,
        querybuilder::{
            syntax::{
                ast::select::SelectAst,
                column::ColumnRef,
                join::JoinKind,
                order::OrderByClause,
                table_metadata::TableMetadata
            },
            QueryBuilder,
            QueryBuilderOps,
            SelectQueryBuilderOps
        }
    }
};
use std::borrow::Cow;
use std::error::Error;

/// Fluent builder for `SELECT` queries
pub struct SelectQueryBuilder<'a> {
    pub(crate) _inner: QueryBuilder<'a, SelectAst<'a>>,
}

impl<'a> SelectQueryBuilder<'a> {
    /// Creates a builder for the given table and target database.
    pub fn new(
        table_schema_data: impl Into<TableMetadata<'a>>,
        database_type: DatabaseType,
    ) -> Self {
        Self {
            _inner: QueryBuilder::new(table_schema_data, SelectAst::new(), database_type),
        }
    }

    /// Creates a builder from already normalized table metadata.
    ///
    /// This constructor is const-compatible and avoids the conversion performed
    /// by [`Self::new`].
    pub const fn new_querybuilder(
        table_schema_data: TableMetadata<'a>,
        database_type: DatabaseType,
    ) -> Self {
        Self {
            _inner: QueryBuilder::new_querybuilder(
                table_schema_data,
                SelectAst::new(),
                database_type,
            ),
        }
    }

    /// Creates a builder directly from schema and table components.
    pub const fn new_from_parts(
        schema: Option<Cow<'a, str>>,
        table_name: Cow<'a, str>,
        database_type: DatabaseType,
    ) -> Self {
        let table_schema_data = TableMetadata {
            schema,
            name: table_name,
        };

        Self::new_querybuilder(table_schema_data, database_type)
    }

    /// Appends columns that have already been converted into [`ColumnRef`] values.
    ///
    /// This avoids repeating identifier conversion in internal or generated code
    /// that already works with the query syntax types.
    pub fn with_known_columns<I>(mut self, columns: I) -> Self
    where
        I: IntoIterator<Item = ColumnRef<'a>>,
    {
        self._inner.ast.columns.extend(columns);
        self
    }

    /// Appends borrowed column names from the representation produced by the
    /// entity metadata APIs.
    pub fn with_known_column_names<I>(mut self, columns: I) -> Self
    where
        I: IntoIterator<Item = &'a &'a str>,
    {
        self._inner
            .ast
            .columns
            .extend(columns.into_iter().map(Into::into));

        self
    }

    #[inline(always)]
    pub fn build(self) -> Result<Query<'a>, Box<dyn Error + Send + Sync + 'a>> {
        self._inner.build()
    }
}

impl<'a> SelectQueryBuilderOps<'a> for SelectQueryBuilder<'a> {
    fn with_columns<I: Into<ColumnRef<'a>>>(mut self, columns: Vec<I>) -> Self {
        self._inner
            .ast
            .columns
            .extend(columns.into_iter().map(Into::into));

        self
    }

    fn with_distinct(mut self) -> Self {
        self._inner.ast.with_distinct = true;
        self
    }

    fn count(mut self) -> Self {
        self._inner.ast.is_count_query = true;
        self
    }

    fn left_join(
        self,
        join_table: impl Into<TableMetadata<'a>>,
        left: impl Into<ColumnRef<'a>>,
        right: impl Into<ColumnRef<'a>>,
    ) -> Self {
        __impl::build_and_append_join_clause(self, JoinKind::Left, join_table, left, right)
    }

    fn inner_join(
        self,
        join_table: impl Into<TableMetadata<'a>>,
        left: impl Into<ColumnRef<'a>>,
        right: impl Into<ColumnRef<'a>>,
    ) -> Self {
        __impl::build_and_append_join_clause(self, JoinKind::Inner, join_table, left, right)
    }

    fn right_join(
        self,
        join_table: impl Into<TableMetadata<'a>>,
        left: impl Into<ColumnRef<'a>>,
        right: impl Into<ColumnRef<'a>>,
    ) -> Self {
        __impl::build_and_append_join_clause(self, JoinKind::Right, join_table, left, right)
    }

    fn full_join(
        self,
        join_table: impl Into<TableMetadata<'a>>,
        left: impl Into<ColumnRef<'a>>,
        right: impl Into<ColumnRef<'a>>,
    ) -> Self {
        __impl::build_and_append_join_clause(self, JoinKind::Full, join_table, left, right)
    }

    fn order_by<Z: FieldIdentifier + Into<ColumnRef<'a>>>(
        mut self,
        order_by: Z,
        desc: bool,
    ) -> Self {
        self._inner.ast.order_by = Some(OrderByClause::new(order_by, desc));
        self
    }
}

impl<'a> QueryBuilderOps<'a> for SelectQueryBuilder<'a> {
    #[inline(always)]
    fn build(self) -> Result<Query<'a>, Box<dyn Error + Send + Sync + 'a>> {
        self._inner.build()
    }

    #[inline]
    fn r#where<I: Into<ColumnRef<'a>>>(mut self, column_name: I, operator: Operator) -> Self {
        self._inner.r#where(column_name, operator);
        self
    }

    #[inline]
    fn where_value<Z: FieldValueIdentifier>(mut self, r#where: &'a Z, op: Operator) -> Self {
        self._inner.where_value(r#where, op);
        self
    }

    #[inline]
    fn and<Z: FieldValueIdentifier>(mut self, column: &'a Z, op: Operator) -> Self {
        self._inner.and(column, op);
        self
    }

    #[inline]
    fn and_values_in<'b, Z, Q>(
        mut self,
        r#and: Z,
        values: &'a [Q],
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'b>>
    where
        Z: FieldIdentifier,
        Q: QueryParameter,
        Self: Sized,
    {
        self._inner.and_values_in(r#and, values)?;
        Ok(self)
    }

    #[inline]
    fn or_values_in<'b, Z, Q>(
        mut self,
        r#or: Z,
        values: &'a [Q],
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'b>>
    where
        Z: FieldIdentifier,
        Q: QueryParameter,
        Self: Sized,
    {
        self._inner.or_values_in(r#or, values)?;
        Ok(self)
    }

    #[inline]
    fn or<Z: FieldValueIdentifier>(mut self, column: &'a Z, op: Operator) -> Self {
        self._inner.or(column, op);
        self
    }
}

mod __impl {
    use crate::query::operators::Operator;
    use crate::query::querybuilder::syntax::column::ColumnRef;
    use crate::query::querybuilder::syntax::join::{JoinClause, JoinKind};
    use crate::query::querybuilder::syntax::table_metadata::TableMetadata;
    use crate::query::querybuilder::SelectQueryBuilder;

    pub(crate) fn build_and_append_join_clause<'a>(
        mut builder: SelectQueryBuilder<'a>,
        join_kind: JoinKind,
        target_table: impl Into<TableMetadata<'a>>,
        left: impl Into<ColumnRef<'a>>,
        right: impl Into<ColumnRef<'a>>,
    ) -> SelectQueryBuilder<'a> {
        let join_clause = build_join_clause(join_kind, target_table, left, right);
        builder._inner.ast.joins.push(join_clause);
        builder
    }

    fn build_join_clause<'a>(
        kind: JoinKind,
        target_table: impl Into<TableMetadata<'a>>,
        left: impl Into<ColumnRef<'a>>,
        right: impl Into<ColumnRef<'a>>,
    ) -> JoinClause<'a> {
        JoinClause {
            kind,
            target_table: target_table.into(),
            left: left.into(),
            operator: Operator::Eq,
            right: right.into(),
        }
    }
}
