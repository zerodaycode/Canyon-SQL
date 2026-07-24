pub(crate) mod delete;
pub(crate) mod insert;
pub(crate) mod select;
pub(crate) mod update;

use crate::query::querybuilder::syntax::clause::ConditionClause;
use crate::query::querybuilder::syntax::table_metadata::TableMetadata;

/// Base AST for common parts
#[derive(Default)]
pub struct BaseAst<'a> {
    table: TableMetadata<'a>,
    conditions: Vec<ConditionClause<'a>>,
}

impl<'a> BaseAst<'a> {
    pub const fn new_ast(table: TableMetadata<'a>) -> Self {
        Self {
            table,
            conditions: Vec::new(),
        }
    }
    pub fn new(table: impl Into<TableMetadata<'a>>) -> Self {
        Self {
            table: table.into(),
            conditions: Vec::new(),
        }
    }

    #[inline(always)]
    pub const fn table(&self) -> &TableMetadata<'a> {
        &self.table
    }

    #[inline(always)]
    pub const fn conditions(&self) -> &Vec<ConditionClause<'a>> {
        &self.conditions
    }

    pub fn add_condition(&mut self, condition: ConditionClause<'a>) {
        self.conditions.push(condition);
    }
}
