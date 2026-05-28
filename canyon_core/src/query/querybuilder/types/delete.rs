use crate::connection::database_type::DatabaseType;
use crate::query::bounds::{FieldIdentifier, FieldValueIdentifier};
use crate::query::operators::Operator;
use crate::query::parameters::QueryParameter;
use crate::query::query::Query;
use crate::query::querybuilder::syntax::ast::delete::DeleteAst;
use crate::query::querybuilder::types::TableMetadata;
use crate::query::querybuilder::{DeleteQueryBuilderOps, QueryBuilder, QueryBuilderOps};
use std::error::Error;

/// Contains the specific database operations associated with the
/// *DELETE* SQL statements.
///
/// * `set` - To construct a new `SET` clause to determine the columns to
///   update with the provided values
pub struct DeleteQueryBuilder<'a> {
    pub(crate) _inner: QueryBuilder<'a, DeleteAst>,
}

impl<'a> DeleteQueryBuilder<'a> {
    /// Generates a new public instance of the [`DeleteQueryBuilder`]
    pub fn new(table_schema_data: impl Into<TableMetadata<'a>>) -> Self {
        Self::new_for(table_schema_data, DatabaseType::Deferred)
    }

    pub fn new_for(
        table_schema_data: impl Into<TableMetadata<'a>>,
        database_type: DatabaseType,
    ) -> Self {
        Self {
            _inner: QueryBuilder::new(table_schema_data, DeleteAst::new(), database_type),
        }
    }

    pub fn build(self) -> Result<Query<'a>, Box<dyn Error + Send + Sync + 'a>> {
        self._inner.build()
    }
}

impl<'a> DeleteQueryBuilderOps<'a> for DeleteQueryBuilder<'a> {} // NOTE: for now, this is just a type formalism

impl<'a> QueryBuilderOps<'a> for DeleteQueryBuilder<'a> {
    #[inline]
    fn r#where(mut self, column_name: &'a str, operator: Operator) -> Self {
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
    {
        self._inner.and_values_in(and, values)?;
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
    {
        self._inner.or_values_in(or, values)?;
        Ok(self)
    }

    #[inline]
    fn or<Z: FieldValueIdentifier>(mut self, column: &'a Z, op: Operator) -> Self {
        self._inner.or(column, op);
        self
    }
}
