use std::borrow::Cow;
use crate::query::parameters::QueryParameter;
use crate::query::querybuilder::syntax::emitter::{EmitBody, EmitFrom, EmitKind};
use crate::query::querybuilder::syntax::table_metadata::TableMetadata;
use crate::query::querybuilder::syntax::tokens::{SqlToken, Symbol, ToSqlTokens};
use crate::query::querybuilder::syntax::tokens::Symbol::{Comma, LParen, RParen};

pub struct InsertAst<'a> {
    pub columns: Vec<&'a str>,
    pub values: Vec<&'a dyn QueryParameter>,
}

impl<'a> EmitKind<'a> for InsertAst<'a> {
    fn emit_kind(&self, out: &mut Vec<SqlToken<'a>>) {
        out.push(SqlToken::new_keyword("INSERT"));
    }
}
impl<'a> EmitFrom<'a> for InsertAst<'a> {
    fn emit_from<'b>(&self, meta: &TableMetadata<'b>, out: &mut Vec<SqlToken<'b>>) {
        if !self.columns.is_empty() { // this can't be emitted here
            // TODO: can we create like a pre-validator trait a-la-emit-kind but for checking invariants?
            out.push(SqlToken::new_ident("INTO"));
            meta.to_tokens(out);
            out.push(SqlToken::Symbol(LParen));
            for (i, c) in self.columns.iter().enumerate() {
                if i > 0 { out.push(SqlToken::Symbol(Symbol::Comma)); }
                //out.push(SqlToken::new_ident(c));
            }
            out.push(SqlToken::Symbol(Symbol::RParen));
        }
    }
}
impl<'a> EmitBody<'a> for InsertAst<'a> {
    fn emit_body(&self, out: &mut Vec<SqlToken<'a>>) {
        out.push(SqlToken::Keyword(Cow::Borrowed("VALUES")));
        out.push(SqlToken::Symbol(LParen));
        // for i in 0..self.values_count {
        //     if i > 0 { out.push(SqlToken::Symbol(Comma)); }
        //     // out.push(SqlToken::PlaceholderNext); self as the real type and call a custom impl
        // }
        out.push(SqlToken::Symbol(RParen));
    }
}

impl<'a> InsertAst<'a> {
    pub fn new() -> Self {
        Self { columns: Vec::new(), values: Vec::new() }
    }
}