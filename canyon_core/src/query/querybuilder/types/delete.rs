use std::error::Error;

use crate::{
    connection::database_type::DatabaseType,
    query::{
        ColumnRef,
        bounds::{FieldIdentifier, FieldValueIdentifier},
        operators::Operator,
        parameters::QueryParameter,
        query::Query,
        querybuilder::{
            DeleteQueryBuilderOps, QueryBuilder, QueryBuilderOps, syntax::ast::delete::DeleteAst,
            types::TableMetadata,
        },
    },
};

/// Fluent builder for `DELETE` statements
pub struct DeleteQueryBuilder<'a> {
    pub(crate) _inner: QueryBuilder<'a, DeleteAst>,
}

impl<'a> DeleteQueryBuilder<'a> {
    /// Creates a delete builder whose database dialect will be resolved later.
    pub fn new(table_schema_data: impl Into<TableMetadata<'a>>) -> Self {
        Self::new_for(table_schema_data, DatabaseType::Deferred)
    }

    /// Creates a delete builder for a specific database dialect.
    pub fn new_for(
        table_schema_data: impl Into<TableMetadata<'a>>,
        database_type: DatabaseType,
    ) -> Self {
        Self {
            _inner: QueryBuilder::new(table_schema_data, DeleteAst::new(), database_type),
        }
    }

    #[inline(always)]
    pub fn build(self) -> Result<Query<'a>, Box<dyn Error + Send + Sync + 'a>> {
        self._inner.build()
    }
}

impl<'a> DeleteQueryBuilderOps<'a> for DeleteQueryBuilder<'a> {}

impl<'a> QueryBuilderOps<'a> for DeleteQueryBuilder<'a> {
    #[inline(always)]
    fn build(self) -> Result<Query<'a>, Box<dyn Error + Send + Sync + 'a>> {
        self._inner.build()
    }

    #[inline]
    fn r#where<I: Into<ColumnRef<'a>>>(mut self, column: I, op: Operator) -> Self {
        self._inner.r#where(column, op);
        self
    }

    #[inline]
    fn where_value<Z: FieldValueIdentifier>(mut self, column: &'a Z, op: Operator) -> Self {
        self._inner.where_value(column, op);
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
        column: Z,
        values: &'a [Q],
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'b>>
    where
        Z: FieldIdentifier,
        Q: QueryParameter,
    {
        self._inner.and_values_in(column, values)?;
        Ok(self)
    }

    #[inline]
    fn or_values_in<'b, Z, Q>(
        mut self,
        column: Z,
        values: &'a [Q],
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'b>>
    where
        Z: FieldIdentifier,
        Q: QueryParameter,
    {
        self._inner.or_values_in(column, values)?;
        Ok(self)
    }

    #[inline]
    fn or<Z: FieldValueIdentifier>(mut self, column: &'a Z, op: Operator) -> Self {
        self._inner.or(column, op);
        self
    }
}
