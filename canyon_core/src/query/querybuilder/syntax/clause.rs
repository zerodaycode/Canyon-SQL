use crate::query::operators::Comp;
use crate::query::querybuilder::syntax::column::ColumnRef;
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
#[derive(Eq, PartialEq, Clone)]
pub enum ConditionClauseKind {
    Where,
    And,
    Or,
    In,
}

impl ConditionClauseKind {
    fn as_str(&self) -> &'static str {
        match self {
            ConditionClauseKind::Where => "WHERE",
            ConditionClauseKind::And => "AND",
            ConditionClauseKind::In => "IN",
            ConditionClauseKind::Or => "OR",
        }
    }
}

impl<'a> ToSqlTokens<'a> for ConditionClause<'a> {
    fn to_tokens(&self) -> impl IntoIterator<Item = SqlToken<'a>> + 'a {
        let mut out = SqlTokens::with_capacity(4);

        // Clause keyword
        out.ident(self.kind.as_str()); // NOTE: dubious

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
