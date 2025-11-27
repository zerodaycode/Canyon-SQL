use crate::query::querybuilder::syntax::emitter::{AstProcessor, EmitFrom, EmitKind};
use crate::query::querybuilder::syntax::table_metadata::TableMetadata;
use crate::query::querybuilder::syntax::tokens::{SqlToken, ToSqlTokens};

pub struct DeleteAst {}

impl AstProcessor for DeleteAst {}

impl<'a> EmitKind<'a> for DeleteAst {
    fn emit_kind(&self, out: &mut Vec<SqlToken<'a>>) {
        out.push(SqlToken::new_keyword("DELETE"));
    }
}
impl<'a> EmitFrom<'a> for DeleteAst {
    fn emit_from<'b>(&self, meta: &TableMetadata<'b>, out: &mut Vec<SqlToken<'b>>) {
        out.push(SqlToken::new_keyword("FROM"));
        meta.to_tokens(out);
    }
}

impl Default for DeleteAst {
    fn default() -> Self {
        Self::new()
    }
}

impl DeleteAst {
    pub fn new() -> Self { Self{} }
}
