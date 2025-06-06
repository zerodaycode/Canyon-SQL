use crate::query::bounds::{FieldIdentifier, FieldValueIdentifier};
use crate::query::operators::Operator;
use crate::query::parameters::QueryParameter;
use crate::query::querybuilder::contracts::{DeleteQueryBuilderOps, QueryBuilderOps};
use crate::query::querybuilder::types::delete::DeleteQueryBuilder;

impl<'a> DeleteQueryBuilderOps<'a> for DeleteQueryBuilder<'a> {} // NOTE: for now, this is just a type formalism

impl<'a> QueryBuilderOps<'a> for DeleteQueryBuilder<'a> {
    #[inline]
    fn read_sql(&'a self) -> &'a str {
        self._inner.sql.as_str()
    }

    #[inline(always)]
    fn push_sql(mut self, sql: &str) {
        self._inner.sql.push_str(sql);
    }

    #[inline]
    fn r#where<Z: FieldValueIdentifier>(mut self, r#where: &'a Z, op: impl Operator) -> Self {
        self._inner.r#where(r#where, op);
        self
    }

    #[inline]
    fn and<Z: FieldValueIdentifier>(mut self, column: &'a Z, op: impl Operator) -> Self {
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
    fn or_values_in<Z, Q>(mut self, r#or: Z, values: &'a [Q]) -> Self
    where
        Z: FieldIdentifier,
        Q: QueryParameter<'a>,
    {
        self._inner.or_values_in(or, values);
        self
    }

    #[inline]
    fn or<Z: FieldValueIdentifier>(mut self, column: &'a Z, op: impl Operator) -> Self {
        self._inner.or(column, op);
        self
    }

    #[inline]
    fn order_by<Z: FieldIdentifier>(mut self, order_by: Z, desc: bool) -> Self {
        self._inner.order_by(order_by, desc);
        self
    }
}
