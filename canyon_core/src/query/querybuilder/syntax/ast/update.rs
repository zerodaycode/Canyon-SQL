use crate::query::querybuilder::syntax::column::ColumnRef;
use crate::query::querybuilder::syntax::emitter::{AstProcessor, EmitBody, EmitKind};
use crate::query::querybuilder::syntax::tokens::SqlToken;

pub struct UpdateAst<'a> {
    pub columns: Vec<ColumnRef<'a>>,
}

impl<'a> AstProcessor for UpdateAst<'a> {}

impl<'a> EmitKind<'a> for UpdateAst<'a> {
    fn emit_kind(&self, out: &mut Vec<SqlToken<'a>>) {
        out.push(SqlToken::new_keyword("UPDATE"));
    }
}

impl<'a> EmitBody<'a> for UpdateAst<'a> {
    fn emit_body(&self, out: &mut Vec<SqlToken<'a>>) {
        out.push(SqlToken::new_keyword("SET"));

        // for (i, (col, _val)) in self.columns.iter().enumerate() {
        //     if i > 0 { out.push(SqlToken::Symbol(",")); }
        //     out.push(SqlToken::Ident(col));
        //     out.push(SqlToken::Symbol("="));
        //     out.push(SqlToken::PlaceholderNext);
        // }
    }
}

impl<'a> Default for UpdateAst<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> UpdateAst<'a> {
    pub fn new() -> Self {
        Self { columns: Vec::new() }
    }
}
