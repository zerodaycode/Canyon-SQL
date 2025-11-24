use crate::connection::database_type::DatabaseType;
use crate::query::bounds::{FieldIdentifier, FieldValueIdentifier};
use crate::query::operators::Comp;
use crate::query::parameters::QueryParameter;
use crate::query::query::Query;
use crate::query::querybuilder::{QueryBuilder, QueryBuilderOps, SelectQueryBuilderOps};
use std::error::Error;
use crate::query::querybuilder::syntax::ast::select::SelectAst;
use crate::query::querybuilder::syntax::emitter::QueryEmitter;
use crate::query::querybuilder::syntax::query_kind::QueryKind;
use crate::query::querybuilder::syntax::table_metadata::TableMetadata;

pub struct SelectQueryBuilder<'a> {
    pub(crate) _inner: QueryBuilder<'a, SelectAst<'a>>, // TODO: probably SelectAst mustn't be on the base QueryBuilder
    pub(crate) columns: &'a [String]
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
        let ast_processor = QueryEmitter::new(
            QueryKind::Select
        );
        Ok(Self {
            _inner: QueryBuilder::new(table_schema_data, ast_processor, database_type)?,
            columns: &[],
        })
    }

    pub fn build(self) -> Result<Query<'a>, Box<dyn Error + Send + Sync + 'a>> {
        // let __self = __impl::write_columns_or_select_all(self)?;
        // let __self = __impl::write_from_clause(__self)?;
        self._inner.build()
    }
}

impl<'a> SelectQueryBuilderOps<'a> for SelectQueryBuilder<'a> {
    fn with_columns(mut self, columns: &'a [String]) -> Self {
        self.columns = columns;
        self
    }

    fn left_join(
        mut self,
        join_table: impl TableMetadata<'a>,
        col1: impl FieldIdentifier, // TODO: t_col, not only col
        col2: impl FieldIdentifier,
    ) -> Self {
        self._inner.sql.push_str(&format!(
            " LEFT JOIN {join_table} ON {} = {}", // TODO: this should be avoided
            col1.table_and_column_name(),
            col2.table_and_column_name()
        ));
        self
    }

    fn inner_join(
        mut self,
        join_table: impl TableMetadata<'a>,
        col1: impl FieldIdentifier,
        col2: impl FieldIdentifier,
    ) -> Self {
        self._inner.sql.push_str(&format!(
            " INNER JOIN {join_table} ON {} = {}",
            col1.table_and_column_name(),
            col2.table_and_column_name()
        ));
        self
    }

    fn right_join(
        mut self,
        join_table: impl TableMetadata<'a>,
        col1: impl FieldIdentifier,
        col2: impl FieldIdentifier,
    ) -> Self {
        self._inner.sql.push_str(&format!(
            " RIGHT JOIN {join_table} ON {} = {}",
            col1.table_and_column_name(),
            col2.table_and_column_name()
        ));
        self
    }

    fn full_join(
        mut self,
        join_table: impl TableMetadata<'a>,
        col1: impl FieldIdentifier,
        col2: impl FieldIdentifier,
    ) -> Self {
        self._inner.sql.push_str(&format!(
            " FULL JOIN {join_table} ON {} = {}",
            col1.table_and_column_name(),
            col2.table_and_column_name()
        ));
        self
    }
}

impl<'a> QueryBuilderOps<'a> for SelectQueryBuilder<'a> {
    #[inline]
    fn read_sql(&'a self) -> &'a str {
        self._inner.sql.as_str()
    }

    #[inline(always)]
    fn push_sql(mut self, sql: &str) {
        self._inner.sql.push_str(sql);
    }

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

mod __impl {
    use crate::query::querybuilder::SelectQueryBuilder;
    use std::error::Error;
    use std::fmt::Write;

    // /// Appends to the underlying SQL buffer all the columns passed in by the callee or simply pushes
    // /// a wildcard * for the SELECT * FROM
    // pub(crate) fn write_columns_or_select_all<'a>(
    //     mut _self: SelectQueryBuilder<'a>,
    // ) -> Result<SelectQueryBuilder<'a>, Box<dyn Error + Send + Sync + 'a>> {
    //     if _self.columns.is_empty() {
    //         _self._inner.push_sql_char('*')?;
    //     } else {
    //         for (i, c) in _self.columns.iter().enumerate() {
    //             if i > 0 {
    //                 _self._inner.push_sql(", ")?;
    //             }
    //             _self._inner.sql.push_str(c);
    //         }
    //     }
    //     Ok(_self)
    // }
    //
    // pub(crate) fn write_from_clause<'a>(
    //     mut _self: SelectQueryBuilder<'a>,
    // ) -> Result<SelectQueryBuilder<'a>, Box<dyn Error + Send + Sync + 'a>> {
    //     write!(_self._inner.sql, "FROM {}", _self._inner.meta)?;
    //     Ok(_self)
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_simple() {
        let meta = TableMetadata::new("public", "user");
        let sel = SelectEmitter {
            columns: vec!["id", "name"],
            joins: vec!["other_table"],
        };
        let mut qb = QueryBuilder::new(meta, QueryEmitter::Select(sel), DatabaseType::PostgreSql);
        qb.where_eq("id");
        let q = qb.build().unwrap();
        assert!(q.sql.contains("SELECT"));
        assert!(q.sql.contains("FROM"));
        assert!(q.sql.contains("id"));
        assert!(q.sql.contains("$1")); // placeholder for where
    }
}