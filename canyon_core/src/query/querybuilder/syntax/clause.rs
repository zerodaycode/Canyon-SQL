use crate::query::operators::Comp;
use crate::query::parameters::QueryParameter;
use crate::query::querybuilder::syntax::tokens::SqlToken;

pub struct ConditionClause<'a> {
    // TODO: where are missing complex where usages, like in joins, so we should consider to add the table
    // to the column like where table.column = ...
    pub(crate) kind: ConditionClauseKind,
    pub(crate) column_name: SqlToken<'a>,
    pub(crate) operator: Comp,
    pub(crate) value: &'a dyn QueryParameter
}
#[derive(Eq, PartialEq)]
pub enum ConditionClauseKind {
    Where,
    And,
    Or,
    In
}

impl<'a> AsRef<str> for ConditionClauseKind {
    fn as_ref(&self) -> &str {
        match self {
            ConditionClauseKind::Where => "WHERE",
            ConditionClauseKind::And => "AND",
            ConditionClauseKind::In => "IN",
            ConditionClauseKind::Or => "OR",
        }
    }
}
