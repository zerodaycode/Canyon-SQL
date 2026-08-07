pub(crate) mod delete;
pub(crate) mod insert;
pub(crate) mod select;
pub(crate) mod update;

use crate::query::querybuilder::syntax::clause::ConditionClause;
use crate::query::querybuilder::syntax::table_metadata::TableMetadata;

/// Query data shared by every statement-specific AST.
///
/// `BaseAst` stores the target table and the ordered collection of conditions
/// used by `SELECT`, `INSERT`, `UPDATE`, and `DELETE` statements.
#[derive(Default)]
pub struct BaseAst<'a> {
    table: TableMetadata<'a>,
    conditions: Vec<ConditionClause<'a>>,
}

impl<'a> BaseAst<'a> {
    /// Creates a base AST from an already constructed [`TableMetadata`].
    pub const fn new_ast(table: TableMetadata<'a>) -> Self {
        Self {
            table,
            conditions: Vec::new(),
        }
    }

    /// Creates a base AST for the provided table.
    pub fn new(table: impl Into<TableMetadata<'a>>) -> Self {
        Self {
            table: table.into(),
            conditions: Vec::new(),
        }
    }

    /// Returns the table targeted by the query.
    #[inline(always)]
    pub const fn table(&self) -> &TableMetadata<'a> {
        &self.table
    }

    /// Returns the conditions registered on the query, in insertion order.
    #[inline(always)]
    pub const fn conditions(&self) -> &[ConditionClause<'a>] {
        self.conditions.as_slice()
    }

    /// Appends a condition to the query.
    pub fn add_condition(&mut self, condition: ConditionClause<'a>) {
        self.conditions.push(condition);
    }
}
