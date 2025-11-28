use crate::query::querybuilder::syntax::column::ColumnRef;
use crate::query::querybuilder::syntax::emitter::{AsEmitBody, AsEmitFrom, AsEmitKind, AstProcessor, EmitBody, EmitFrom, EmitKind, ToSql};
use crate::query::querybuilder::syntax::tokens::SqlToken;

pub struct UpdateAst<'a> {
    pub columns: Vec<ColumnRef<'a>>,
}

impl<'a> AstProcessor for UpdateAst<'a> {}
impl<'a> ToSql<'a> for UpdateAst<'a> {}

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

impl<'a> AsEmitKind<'a> for UpdateAst<'a> {
    fn as_emit_kind(&self) -> Option<&dyn EmitKind<'a>> { Some(self as &dyn EmitKind<'a>) }
}
impl<'a> AsEmitFrom<'a> for UpdateAst<'a> {
    fn as_emit_from(&self) -> Option<&dyn EmitFrom<'a>> { None }
}
impl<'a> AsEmitBody<'a> for UpdateAst<'a> {
    fn as_emit_body(&self) -> Option<&dyn EmitBody<'a>> { Some(self as &dyn EmitBody<'a>) }
}