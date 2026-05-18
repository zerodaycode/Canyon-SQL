use crate::query::operators::Operator;
use crate::query::querybuilder::syntax::column::ColumnRef;
use crate::query::querybuilder::syntax::dialect::SqlDialect;
use crate::query::querybuilder::syntax::keyword::Keyword;
use crate::query::querybuilder::syntax::tokens::{
    PlaceholderKind, SqlToken, SqlTokens, ToSqlTokens,
};

#[derive(Clone)]
pub struct ConditionClause<'a> {
    pub(crate) kind: ConditionClauseKind,
    pub(crate) column_name: ColumnRef<'a>,
    pub(crate) operator: Operator,
    pub(crate) value_indexes: PlaceholderKind,
}
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum ConditionClauseKind {
    Where,
    And,
    In,
    Or,
    AndValuesIn,
    OrValuesIn,
}

impl From<ConditionClauseKind> for Keyword {
    fn from(keyword: ConditionClauseKind) -> Self {
        match keyword {
            ConditionClauseKind::Where => Keyword::Where,
            ConditionClauseKind::And | ConditionClauseKind::AndValuesIn => Keyword::And,
            ConditionClauseKind::Or | ConditionClauseKind::OrValuesIn => Keyword::Or,
            ConditionClauseKind::In => Keyword::In
        }
    }
}

impl<'a, D: SqlDialect> ToSqlTokens<'a, D> for ConditionClause<'a> {
    fn to_tokens(&self) -> impl IntoIterator<Item = SqlToken<'a>> + 'a {
        let mut out = SqlTokens::with_capacity(4);

        // Clause keyword
        out.keyword(self.kind.into());

        // Column
        out.extend(<ColumnRef<'_> as ToSqlTokens<'_, D>>::to_tokens(
            &self.column_name,
        ));

        out.whitespace();

        // Operator
        out.operator(self.operator);

        out.whitespace();

        // Value(s) placeholder(s)
        out.placeholder(self.value_indexes);

        out.whitespace();

        out
    }
}
