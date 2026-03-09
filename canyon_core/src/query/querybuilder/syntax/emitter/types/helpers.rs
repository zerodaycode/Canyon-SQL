//! Standalone functions that shares the same behaviour for different AST kinds

use crate::query::querybuilder::syntax::column::ColumnRef;
use crate::query::querybuilder::syntax::symbol::Symbol::Comma;
use crate::query::querybuilder::syntax::tokens::{SqlTokens, ToSqlTokens};

pub(crate) fn emit_columns<'a>(columns: &Vec<ColumnRef<'a>>, tokens: &mut SqlTokens<'a>) {
    for (i, column) in columns.iter().enumerate() {
        if i > 0 {
            tokens.symbol(Comma);
        }
        tokens.extend(column.to_tokens());
    }
}
