use crate::query::operators::Comp;
use crate::query::parameters::QueryParameter;
use crate::query::querybuilder::syntax::column::ColumnRef;
use crate::query::querybuilder::syntax::dialect;
use crate::query::querybuilder::syntax::dialect::SqlDialect;
use crate::query::querybuilder::syntax::keyword::Keyword;
use crate::query::querybuilder::syntax::tokens::{
    PlaceholderKind, SqlToken, SqlTokens, ToSqlTokens,
};

pub struct HavingClause<'a> {
    pub column: ColumnRef<'a>,
    pub operator: Comp,
    pub value_index: PlaceholderKind, // TODO: shouldn't this be a placeholder?
}

impl<'a> HavingClause<'a> {
    pub fn new<I: Into<ColumnRef<'a>>>(column: I, operator: Comp, value_index: usize) -> Self {
        Self {
            column: column.into(),
            operator,
            value_index: PlaceholderKind::Value(value_index),
        }
    }
}

impl<'a> ToSqlTokens<'a> for HavingClause<'a> {
    fn to_tokens(&self) -> impl IntoIterator<Item = SqlToken<'a>> + 'a {
        let mut out = SqlTokens::with_capacity(4);

        out.keyword(Keyword::Having);
        out.extend(self.column.to_tokens());
        out.operator(self.operator);
        out.placeholder(self.value_index);

        out
    }
}
