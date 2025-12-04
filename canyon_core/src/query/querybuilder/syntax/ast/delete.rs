use crate::query::querybuilder::syntax::emitter::{
    AsEmitBody, AsEmitFrom, AsEmitKind, AstProcessor, EmitBody, EmitFrom, EmitKind, ToSql,
};
use crate::query::querybuilder::syntax::table_metadata::TableMetadata;
use crate::query::querybuilder::syntax::tokens::{SqlToken, ToSqlTokens};

pub struct DeleteAst {}

impl AstProcessor for DeleteAst {}
impl<'a> ToSql<'a> for DeleteAst {}

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
    pub fn new() -> Self {
        Self {}
    }
}

impl<'a> AsEmitKind<'a> for DeleteAst {
    fn as_emit_kind(&self) -> Option<&dyn EmitKind<'a>> {
        Some(self as &dyn EmitKind<'a>)
    }
}
impl<'a> AsEmitFrom<'a> for DeleteAst {
    fn as_emit_from(&self) -> Option<&dyn EmitFrom<'a>> {
        Some(self as &dyn EmitFrom<'a>)
    }
}
impl<'a> AsEmitBody<'a> for DeleteAst {
    fn as_emit_body(&self) -> Option<&dyn EmitBody<'a>> {
        None
    }
}
