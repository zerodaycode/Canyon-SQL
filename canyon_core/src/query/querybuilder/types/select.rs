use crate::connection::database_type::DatabaseType;
use crate::query::bounds::{FieldIdentifier, FieldValueIdentifier};
use crate::query::operators::Comp;
use crate::query::parameters::QueryParameter;
use crate::query::query::Query;
use crate::query::querybuilder::{QueryBuilder, QueryBuilderOps, SelectQueryBuilderOps};
use std::error::Error;
use crate::query::querybuilder::syntax::ast::select::SelectAst;
use crate::query::querybuilder::syntax::column::ColumnRef;
use crate::query::querybuilder::syntax::table_metadata::TableMetadata;

pub struct SelectQueryBuilder<'a> {
    pub(crate) _inner: QueryBuilder<'a, SelectAst<'a>>, // TODO: probably SelectAst mustn't be on the base QueryBuilder
}

impl<'a> SelectQueryBuilder<'a> {
    /// The constructor for creating [`QueryBuilder`] instances of type: SELECT
    pub fn new(
        table_schema_data: impl Into<TableMetadata<'a>>,
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'a>>
    {
        SelectQueryBuilder::new_for(table_schema_data, DatabaseType::Deferred)
    }

    /// Same as [`SelectQueryBuilder::new`] but specifying the [`DatabaseType`]
    pub fn new_for(
        table_schema_data: impl Into<TableMetadata<'a>>,
        database_type: DatabaseType,
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'a>>
    {
        Ok(Self {
            _inner: QueryBuilder::new(table_schema_data, SelectAst::new(), database_type)?,
        })
    }

    pub fn build(self) -> Result<Query<'a>, Box<dyn Error + Send + Sync + 'a>> {
        // let __self = __impl::write_columns_or_select_all(self)?;
        // let __self = __impl::write_from_clause(__self)?;
        self._inner.build()
    }
}

impl<'a> SelectQueryBuilderOps<'a> for SelectQueryBuilder<'a> {
    fn with_columns<I: Into<ColumnRef<'a>>>(mut self, columns: Vec<I>) -> Self {
        self._inner.ast.columns = columns.into_iter().map(|e| e.into()).collect();
        self
    }

    fn left_join(
        self,
        join_table: impl crate::query::bounds::TableMetadata<'a>,
        col1: impl FieldIdentifier, // TODO: t_col, not only col
        col2: impl FieldIdentifier,
    ) -> Self {
        // self._inner.sql.push_str(&format!(
        //     " LEFT JOIN {join_table} ON {} = {}", // TODO: this should be avoided
        //     col1.table_and_column_name(),
        //     col2.table_and_column_name()
        // ));
        self
    }

    fn inner_join(
        self,
        join_table: impl crate::query::bounds::TableMetadata<'a>,
        col1: impl FieldIdentifier,
        col2: impl FieldIdentifier,
    ) -> Self {
        // self._inner.sql.push_str(&format!(
        //     " INNER JOIN {join_table} ON {} = {}",
        //     col1.table_and_column_name(),
        //     col2.table_and_column_name()
        // ));
        self
    }

    fn right_join(
        self,
        join_table: impl crate::query::bounds::TableMetadata<'a>,
        col1: impl FieldIdentifier,
        col2: impl FieldIdentifier,
    ) -> Self {
        // self._inner.sql.push_str(&format!(
        //     " RIGHT JOIN {join_table} ON {} = {}",
        //     col1.table_and_column_name(),
        //     col2.table_and_column_name()
        // ));
        self
    }

    fn full_join(
        self,
        join_table: impl crate::query::bounds::TableMetadata<'a>,
        col1: impl FieldIdentifier,
        col2: impl FieldIdentifier,
    ) -> Self {
        // self._inner.sql.push_str(&format!(
        //     " FULL JOIN {join_table} ON {} = {}",
        //     col1.table_and_column_name(),
        //     col2.table_and_column_name()
        // ));
        self
    }
}

impl<'a> QueryBuilderOps<'a> for SelectQueryBuilder<'a> {
    #[inline]
    fn r#where(mut self, column_name: &'a str, operator: Comp, value: &'a dyn QueryParameter) -> Self {
        self._inner.r#where(column_name, operator, value);
        self
    }

    #[inline]
    fn where_value<Z: FieldValueIdentifier>(mut self, r#where: &'a Z, op: Comp) -> Self {
        self._inner.where_value(r#where, op);
        self
    }

    #[inline]
    fn and<Z: FieldValueIdentifier>(mut self, column: &'a Z, op: Comp) -> Self {
        self._inner.and(column, op);
        self
    }

    #[inline]
    fn and_values_in<'b, Z, Q>(mut self, r#and: Z, values: &'a [Q]) -> Result<Self, Box<dyn Error + Send + Sync + 'b>>
    where
        Z: FieldIdentifier,
        Q: QueryParameter,
        Self: Sized
    {
        self._inner.and_values_in(and, values)?;
        Ok(self)
    }

    #[inline]
    fn or_values_in<'b, Z, Q>(mut self, r#and: Z, values: &'a [Q]) -> Result<Self, Box<dyn Error + Send + Sync + 'b>>
    where
        Z: FieldIdentifier,
        Q: QueryParameter,
        Self: Sized,
    {
        self._inner.or_values_in(and, values)?;
        Ok(self)
    }

    #[inline]
    fn or<Z: FieldValueIdentifier>(mut self, column: &'a Z, op: Comp) -> Self {
        self._inner.or(column, op);
        self
    }

    #[inline]
    fn order_by<Z: FieldIdentifier>(mut self, order_by: Z, desc: bool) -> Self {
        self._inner.order_by(order_by, desc);
        self
    }
}
