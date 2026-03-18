use crate::query::querybuilder::syntax::column::ColumnRef;
use crate::query::querybuilder::syntax::keyword::Keyword;
use crate::query::querybuilder::syntax::tokens::{SqlToken, SqlTokens, ToSqlTokens};
use tiberius::ToSql;

#[derive(Debug, Clone, Default)]
pub struct OrderByClause<'a> {
    pub column: ColumnRef<'a>,
    pub descending: bool,
}

impl<'a> OrderByClause<'a> {
    pub fn new<I: Into<ColumnRef<'a>>>(column: I, descending: bool) -> Self {
        Self {
            column: column.into(),
            descending,
        }
    }
}

impl<'a> ToSqlTokens<'a> for OrderByClause<'a> {
    fn to_tokens(&self) -> impl IntoIterator<Item = SqlToken<'a>> + 'a {
        let mut out = SqlTokens::with_capacity(3);

        out.keyword(Keyword::OrderBy);
        out.extend(self.column.to_tokens());
        if self.descending {
            out.keyword(Keyword::Desc);
        }

        out
    }
}
