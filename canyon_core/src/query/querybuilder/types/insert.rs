use std::borrow::Cow;
use std::error::Error;

use crate::connection::database_type::DatabaseType;
use crate::query::bounds::{FieldIdentifier, FieldValueIdentifier};
use crate::query::operators::Operator;
use crate::query::parameters::QueryParameter;
use crate::query::query::Query;
use crate::query::querybuilder::syntax::ast::insert::InsertAst;
use crate::query::querybuilder::syntax::column::ColumnRef;
use crate::query::querybuilder::syntax::table_metadata::TableMetadata;
use crate::query::querybuilder::{InsertQueryBuilderOps, QueryBuilder, QueryBuilderOps};

pub struct InsertQueryBuilder<'a> {
    pub(crate) _inner: QueryBuilder<'a, InsertAst<'a>>,
}

impl<'a> InsertQueryBuilder<'a> {
    pub fn new(
        table_schema_data: impl Into<TableMetadata<'a>>,
        database_type: DatabaseType,
    ) -> Self {
        Self {
            _inner: QueryBuilder::new(table_schema_data, InsertAst::new(), database_type),
        }
    }

    pub const fn new_querybuilder(
        table_schema_data: TableMetadata<'a>,
        database_type: DatabaseType,
    ) -> Self {
        Self {
            _inner: QueryBuilder::new_querybuilder(
                table_schema_data,
                InsertAst::new(),
                database_type,
            ),
        }
    }

    pub const fn new_from_parts(
        schema: Option<Cow<'a, str>>,
        table_name: Cow<'a, str>,
        database_type: DatabaseType,
    ) -> Self {
        let table_schema_data = TableMetadata {
            schema,
            name: table_name,
        };
        Self {
            _inner: QueryBuilder::new_querybuilder(
                table_schema_data,
                InsertAst::new(),
                database_type,
            ),
        }
    }

    // TODO: this should be abstracted into a trait for all query builders, but for now it's only implemented for InsertQueryBuilder and SelectQueryBuilder
    pub fn with_known_columns<I>(mut self, columns: I) -> Self
    where
        I: IntoIterator<Item = ColumnRef<'a>>,
    {
        self._inner.ast.columns.extend(columns);
        self
    }

    pub fn returning_columns<I>(mut self, columns: I) -> Self
    where
        I: IntoIterator<Item = ColumnRef<'a>>,
    {
        self._inner.ast.returning_columns.extend(columns);
        self
    }

    #[inline(always)]
    pub fn build(self) -> Result<Query<'a>, Box<dyn Error + Send + Sync + 'a>> {
        self._inner.build()
    }
}

impl<'a> QueryBuilderOps<'a> for InsertQueryBuilder<'a> {
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
        Self: Sized,
    {
        self._inner.and_values_in(column, values)?;
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

impl<'a> InsertQueryBuilderOps<'a> for InsertQueryBuilder<'a> {
    fn with_columns<I: Into<ColumnRef<'a>>>(mut self, columns: Vec<I>) -> Self {
        self._inner.ast.columns = columns.into_iter().map(|e| e.into()).collect();
        self
    }

    fn with_values<Q>(mut self, values: &'a [Q]) -> Result<Self, Box<dyn Error + Send + Sync + 'a>>
    where
        Q: QueryParameter,
    {
        for value in values {
            self._inner.params.push(value);
        }
        Ok(self)
    }

    fn returning(mut self, columns: Vec<impl Into<ColumnRef<'a>>>) -> Self {
        self._inner.ast.returning_columns = columns.into_iter().map(|e| e.into()).collect();
        self
    }
}
