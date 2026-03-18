//! Standalone functions that shares the same behaviour for different AST kinds

use crate::query::querybuilder::syntax::ast::BaseAst;
use crate::query::querybuilder::syntax::column::ColumnRef;
use crate::query::querybuilder::syntax::dialect::SqlDialect;
use crate::query::querybuilder::syntax::emitter::SqlEmitter;
use crate::query::querybuilder::syntax::symbol::Symbol::Comma;
use crate::query::querybuilder::syntax::tokens::{
    PlaceholderKind, SqlToken, SqlTokens, ToSqlTokens,
};

/// Helper function to push a quoted identifier (like table or column names) into the token stream
pub fn push_quoted_ident<'a, D: SqlDialect>(
    element: &impl ToSqlTokens<'a>,
    tokens: &mut SqlTokens<'a>,
) {
    let q = D::IDENT_QUOTING;
    tokens.ident(q.opening());
    tokens.extend(element.to_tokens());
    tokens.ident(q.closing());
}

/// Helper function to emit a list of columns, separated by commas
pub(crate) fn emit_columns<'a, D: SqlDialect>(
    columns: &Vec<ColumnRef<'a>>,
    tokens: &mut SqlTokens<'a>,
) {
    for (i, column) in columns.iter().enumerate() {
        if i > 0 {
            tokens.symbol(Comma);
        }
        push_quoted_ident::<D>(column, tokens);
    }
}

pub(crate) fn emit_placeholders<'a, E: SqlEmitter<'a>>(
    columns: &Vec<ColumnRef<'a>>,
    base_ast: &mut BaseAst<'a>,
    tokens: &mut SqlTokens<'a>,
) {
    columns.iter().for_each(|_| {
        tokens.placeholder(PlaceholderKind::Value(base_ast.next_placeholder_index()));
    });
}
