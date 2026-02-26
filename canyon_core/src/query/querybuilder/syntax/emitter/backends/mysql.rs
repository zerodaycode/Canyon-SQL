use crate::query::querybuilder::syntax::ast::BaseAst;
use crate::query::querybuilder::syntax::ast::select::SelectAst;
use crate::query::querybuilder::syntax::dialect::{MySql, PgDialect};
use crate::query::querybuilder::syntax::emitter::SqlEmitter;
use crate::query::querybuilder::syntax::emitter::types::select::EmitSelect;
use crate::query::querybuilder::syntax::tokens::SqlTokens;

#[derive(Default)]
pub struct MySqlEmitter<'a> {
    tokens: SqlTokens<'a>
}

impl<'a> SqlEmitter<'a, SelectAst<'a>> for MySqlEmitter<'a> {
    type Dialect = MySql;

    fn tokens(&mut self) -> &mut SqlTokens<'a> {
        &mut self.tokens
    }

    fn emit(&mut self, ast: &'a SelectAst<'a>, base_ast: &'a BaseAst<'a>) {
        self.emit_select(ast, base_ast)
    }
}
