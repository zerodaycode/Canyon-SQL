pub(crate) mod delete;
pub(crate) mod insert;
pub(crate) mod select;
pub(crate) mod update;

use crate::query::querybuilder::syntax::clause::ConditionClause;
use crate::query::querybuilder::syntax::table_metadata::TableMetadata;

/// Base AST for common parts
#[derive(Default)]
pub struct BaseAst<'a> {
    pub table: TableMetadata<'a>,
    pub conditions: Vec<ConditionClause<'a>>,
}

impl<'a> BaseAst<'a> {
    pub fn new(table: impl Into<TableMetadata<'a>>) -> Self {
        Self {
            table: table.into(),
            conditions: Vec::new(),
        }
    }
}
