use crate::query::operators::Operator;
use crate::query::querybuilder::syntax::column::ColumnRef;
use crate::query::querybuilder::syntax::dialect::SqlDialect;
use crate::query::querybuilder::syntax::keyword::Keyword;
use crate::query::querybuilder::syntax::tokens::{SqlToken, SqlTokens, ToSqlTokens};

pub struct HavingClause<'a> {
    pub column: ColumnRef<'a>,
    pub operator: Operator,
}

impl<'a> HavingClause<'a> {
    pub fn new<I: Into<ColumnRef<'a>>>(column: I, operator: Operator) -> Self {
        Self {
            column: column.into(),
            operator,
        }
    }

    pub const fn new_const(column: ColumnRef<'a>, operator: Operator) -> Self {
        Self { column, operator }
    }
}

impl<'a, D: SqlDialect> ToSqlTokens<'a, D> for HavingClause<'a> {
    fn to_tokens(&self) -> impl IntoIterator<Item = SqlToken<'a>> + 'a {
        let mut out = SqlTokens::with_capacity(4);

        out.keyword(Keyword::Having);
        <ColumnRef<'_> as ToSqlTokens<'_, D>>::to_tokens(&self.column);
        out.operator(self.operator);
        out.placeholder();

        out
    }
}
