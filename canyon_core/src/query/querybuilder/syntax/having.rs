use crate::query::operators::Operator;
use crate::query::querybuilder::syntax::column::ColumnRef;
use crate::query::querybuilder::syntax::dialect::SqlDialect;
use crate::query::querybuilder::syntax::keyword::Keyword;
use crate::query::querybuilder::syntax::tokens::{
    PlaceholderKind, SqlToken, SqlTokens, ToSqlTokens,
};

pub struct HavingClause<'a> {
    pub column: ColumnRef<'a>,
    pub operator: Operator,
    pub value_index: PlaceholderKind, // TODO: shouldn't this be a placeholder?
}

impl<'a> HavingClause<'a> {
    pub fn _new<I: Into<ColumnRef<'a>>>(column: I, operator: Operator, value_index: usize) -> Self {
        Self {
            column: column.into(),
            operator,
            value_index: PlaceholderKind::Value(value_index),
        }
    }

    pub const fn _new_const(column: ColumnRef<'a>, operator: Operator, value_index: usize) -> Self {
        Self {
            column,
            operator,
            value_index: PlaceholderKind::Value(value_index),
        }
    }
}

impl<'a, D: SqlDialect> ToSqlTokens<'a, D> for HavingClause<'a> {
    fn to_tokens(&self) -> impl IntoIterator<Item = SqlToken<'a>> + 'a {
        let mut out = SqlTokens::with_capacity(4);

        out.keyword(Keyword::Having);
        <ColumnRef<'_> as ToSqlTokens<'_, D>>::to_tokens(&self.column);
        out.operator(self.operator);
        out.placeholder(self.value_index);

        out
    }
}
