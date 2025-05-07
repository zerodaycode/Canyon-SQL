use crate::connection::contracts::DbConnection;
use crate::mapper::RowMapper;
use crate::query::bounds::{FieldIdentifier, FieldValueIdentifier};
use crate::query::operators::Operator;
use crate::query::parameters::QueryParameter;
use crate::query::querybuilder::contracts::{QueryBuilderOps, SelectQueryBuilderOps};
use crate::query::querybuilder::types::select::SelectQueryBuilder;

impl<'a, I: DbConnection + ?Sized, R: RowMapper> SelectQueryBuilderOps<'a>
    for SelectQueryBuilder<'a, I, R>
{
    fn left_join(mut self, join_table: &str, col1: &str, col2: &str) -> Self {
        self._inner
            .sql
            .push_str(&format!(" LEFT JOIN {join_table} ON {col1} = {col2}"));
        self
    }

    fn inner_join(mut self, join_table: &str, col1: &str, col2: &str) -> Self {
        self._inner
            .sql
            .push_str(&format!(" INNER JOIN {join_table} ON {col1} = {col2}"));
        self
    }

    fn right_join(mut self, join_table: &str, col1: &str, col2: &str) -> Self {
        self._inner
            .sql
            .push_str(&format!(" RIGHT JOIN {join_table} ON {col1} = {col2}"));
        self
    }

    fn full_join(mut self, join_table: &str, col1: &str, col2: &str) -> Self {
        self._inner
            .sql
            .push_str(&format!(" FULL JOIN {join_table} ON {col1} = {col2}"));
        self
    }
}

impl<'a, I: DbConnection + ?Sized, R: RowMapper> QueryBuilderOps<'a>
    for SelectQueryBuilder<'a, I, R>
{
    #[inline]
    fn read_sql(&'a self) -> &'a str {
        self._inner.sql.as_str()
    }

    #[inline(always)]
    fn push_sql(mut self, sql: &str) {
        self._inner.sql.push_str(sql);
    }

    #[inline]
    fn r#where<Z: FieldValueIdentifier<'a>>(mut self, r#where: Z, op: impl Operator) -> Self {
        self._inner.r#where(r#where, op);
        self
    }

    #[inline]
    fn and<Z: FieldValueIdentifier<'a>>(mut self, column: Z, op: impl Operator) -> Self {
        self._inner.and(column, op);
        self
    }

    #[inline]
    fn and_values_in<Z, Q>(mut self, r#and: Z, values: &'a [Q]) -> Self
    where
        Z: FieldIdentifier,
        Q: QueryParameter<'a>,
    {
        self._inner.and_values_in(and, values);
        self
    }

    #[inline]
    fn or_values_in<Z, Q>(mut self, r#and: Z, values: &'a [Q]) -> Self
    where
        Z: FieldIdentifier,
        Q: QueryParameter<'a>,
    {
        self._inner.or_values_in(and, values);
        self
    }

    #[inline]
    fn or<Z: FieldValueIdentifier<'a>>(mut self, column: Z, op: impl Operator) -> Self {
        self._inner.or(column, op);
        self
    }

    #[inline]
    fn order_by<Z: FieldIdentifier>(mut self, order_by: Z, desc: bool) -> Self {
        self._inner.order_by(order_by, desc);
        self
    }
}
