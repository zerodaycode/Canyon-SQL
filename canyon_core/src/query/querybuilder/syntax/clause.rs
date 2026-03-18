use crate::query::operators::Comp;
use crate::query::querybuilder::syntax::column::ColumnRef;
use crate::query::querybuilder::syntax::keyword::Keyword;
use crate::query::querybuilder::syntax::tokens::{
    PlaceholderKind, SqlToken, SqlTokens, ToSqlTokens,
};

#[derive(Clone)]
pub struct ConditionClause<'a> {
    pub(crate) kind: ConditionClauseKind,
    pub(crate) column_name: ColumnRef<'a>,
    pub(crate) operator: Comp,
    pub(crate) value_index: usize,
}
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum ConditionClauseKind {
    Where,
    And,
    Or,
    In,
}

impl From<ConditionClauseKind> for Keyword {
    fn from(keyword: ConditionClauseKind) -> Self {
        match keyword {
            ConditionClauseKind::Where => Keyword::Where,
            ConditionClauseKind::And => Keyword::And,
            ConditionClauseKind::Or => Keyword::Or,
            ConditionClauseKind::In => Keyword::In,
        }
    }
}

impl<'a> ToSqlTokens<'a> for ConditionClause<'a> {
    fn to_tokens(&self) -> impl IntoIterator<Item = SqlToken<'a>> + 'a {
        let mut out = SqlTokens::with_capacity(4);

        // Clause keyword
        out.keyword(self.kind.into());

        // Column
        out.extend(self.column_name.to_tokens());

        // Operator
        out.operator(self.operator);

        // Value placeholder
        out.placeholder(match self.operator {
            Comp::Like(kind) => PlaceholderKind::Like(kind, self.value_index),
            _ => PlaceholderKind::Value(self.value_index),
        });

        out
    }
}
