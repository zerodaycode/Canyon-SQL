use crate::query::operators::Comp;
use crate::query::querybuilder::syntax::column::ColumnRef;

#[derive(Clone)] pub struct ConditionClause<'a> {
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
    In
}

impl AsRef<str> for ConditionClauseKind {
    fn as_ref(&self) -> &str {
        match self {
            ConditionClauseKind::Where => "WHERE",
            ConditionClauseKind::And => "AND",
            ConditionClauseKind::In => "IN",
            ConditionClauseKind::Or => "OR",
        }
    }
}
