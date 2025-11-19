use crate::query::operators::Comp;
use crate::query::querybuilder::syntax::table_metadata::TableMetadata;

#[derive(Debug, Clone, Copy)]
pub enum JoinKind {
    Inner,
    Left,
    Right,
    Full,
}

impl JoinKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            JoinKind::Inner => "INNER JOIN",
            JoinKind::Left => "LEFT JOIN",
            JoinKind::Right => "RIGHT JOIN",
            JoinKind::Full => "FULL JOIN",
        }
    }
}

pub struct JoinClause<'a> {
    pub kind: JoinKind,
    pub table: TableMetadata, // TODO: this should be the target table, and the origin table
    pub left: &'a str,  // e.g. "t1.id" // TODO: we need to filter and check the syntax
    pub operator: Comp, // usually Eq
    pub right: &'a str, // e.g. "t2.t1_id"
}

impl<'a> JoinClause<'a> {
    pub fn new(kind: JoinKind, table: TableMetadata, left: &'a str, operator: Comp, right: &'a str) -> Self {
        Self { kind, table, left, operator, right }
    }
}
