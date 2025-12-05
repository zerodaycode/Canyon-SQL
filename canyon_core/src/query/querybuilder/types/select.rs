use crate::connection::database_type::DatabaseType;
use crate::query::bounds::{FieldIdentifier, FieldValueIdentifier};
use crate::query::operators::Comp;
use crate::query::parameters::QueryParameter;
use crate::query::query::Query;
use crate::query::querybuilder::syntax::ast::select::SelectAst;
use crate::query::querybuilder::syntax::column::ColumnRef;
use crate::query::querybuilder::syntax::join::JoinKind;
use crate::query::querybuilder::syntax::order::OrderByClause;
use crate::query::querybuilder::syntax::table_metadata::TableMetadata;
use crate::query::querybuilder::{QueryBuilder, QueryBuilderOps, SelectQueryBuilderOps};
use std::error::Error;

pub struct SelectQueryBuilder<'a> {
    pub(crate) _inner: QueryBuilder<'a, SelectAst<'a>>,
}

impl<'a> SelectQueryBuilder<'a> {
    /// The constructor for creating [`QueryBuilder`] instances of type: SELECT
    pub fn new(
        table_schema_data: impl Into<TableMetadata<'a>>,
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'a>> {
        SelectQueryBuilder::new_for(table_schema_data, DatabaseType::Deferred)
    }

    /// Same as [`SelectQueryBuilder::new`] but specifying the [`DatabaseType`]
    pub fn new_for(
        table_schema_data: impl Into<TableMetadata<'a>>,
        database_type: DatabaseType,
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'a>> {
        Ok(Self {
            _inner: QueryBuilder::new(table_schema_data, SelectAst::new(), database_type)?,
        })
    }

    pub fn sql(&self) -> Result<String, Box<dyn Error + Send + Sync + 'a>> {
        self._inner.sql()
    }

    pub fn build(self) -> Result<Query<'a>, Box<dyn Error + Send + Sync + 'a>> {
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
    #[inline]
    fn r#where(mut self, column_name: &'a str, operator: Comp) -> Self {
        self._inner.r#where(column_name, operator);
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
        self._inner.and_values_in(and, values)?;
        Ok(self)
    }

    #[inline]
    fn or_values_in<'b, Z, Q>(
        mut self,
        r#and: Z,
        values: &'a [Q],
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'b>>
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
}

mod __impl {
    use crate::query::operators::Comp;
    use crate::query::querybuilder::syntax::column::ColumnRef;
    use crate::query::querybuilder::syntax::join::JoinKind::Left;
    use crate::query::querybuilder::syntax::join::{JoinClause, JoinKind};
    use crate::query::querybuilder::syntax::table_metadata::TableMetadata;
    use crate::query::querybuilder::SelectQueryBuilder;

    pub(crate) fn build_and_append_join_clause<'a>(
        mut _self: SelectQueryBuilder<'a>,
        join_kind: JoinKind,
        target_table: impl Into<TableMetadata<'a>>,
        left: impl Into<ColumnRef<'a>>,
        right: impl Into<ColumnRef<'a>>,
    ) -> SelectQueryBuilder<'a> {
        let join_clause = build_join_clause(join_kind, target_table, left, right);
        _self._inner.ast.joins.push(join_clause);
        _self
    }

    fn build_join_clause<'a>(
        kind: JoinKind,
        target_table: impl Into<TableMetadata<'a>>,
        left: impl Into<ColumnRef<'a>>,
        right: impl Into<ColumnRef<'a>>,
    ) -> JoinClause<'a> {
        JoinClause {
            kind: Left,
            target_table: target_table.into(),
            left: left.into(),
            operator: Comp::Eq,
            right: right.into(),
        }
    }
}
