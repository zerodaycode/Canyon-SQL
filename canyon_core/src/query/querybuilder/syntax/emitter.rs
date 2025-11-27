use std::borrow::Cow;
use crate::query::querybuilder::syntax::ast::delete::DeleteAst;
use crate::query::querybuilder::syntax::ast::insert::InsertAst;
use crate::query::querybuilder::syntax::ast::select::SelectAst;
use crate::query::querybuilder::syntax::ast::update::UpdateAst;
use crate::query::querybuilder::syntax::clause::ConditionClause;
use crate::query::querybuilder::syntax::query_kind::QueryKind;
use crate::query::querybuilder::syntax::table_metadata::TableMetadata;
use crate::query::querybuilder::syntax::tokens::{SqlToken, Symbol, ToSqlTokens};
use crate::query::querybuilder::syntax::tokens::Symbol::{LParen, RParen};

// ---------- AST Processor marker trait ----------
pub trait AstProcessor: Default {
    // TODO: get base? as mut ref for convenience?
} // TODO: maybe this and the other one are visitor related?

// ---------- Emit traits ----------
pub trait EmitKind<'a> {
    fn emit_kind(&self, out: &mut Vec<SqlToken<'a>>);
}
pub trait EmitFrom<'a> {
    fn emit_from<'b>(&self, meta: &TableMetadata<'b>, out: &mut Vec<SqlToken<'b>>) ;
}
pub trait EmitBody<'a> {
    fn emit_body(&self, out: &mut Vec<SqlToken<'a>>);
}


// ---------- QueryEmitter enum ----------
pub enum QueryEmitter<'a> { // Isn't this almost queryKind?
    // Raw(BaseAst<'a>),
    Select(SelectAst<'a>),
    Insert(InsertAst<'a>),
    Update(UpdateAst<'a>),
    Delete(DeleteAst),
}
// ---------- QueryEmitter enum ----------
// pub struct QueryEmitter<'a, P: AstProcessor> { // Isn't this almost queryKind?
//     kind: QueryKind,
//     ast: P,
//     _p: &'a str,
// }

impl<'a> QueryEmitter<'a> {
    /// façade: executes all phases in logical order: KIND -> FROM -> BODY -> CONDITIONS
    pub fn emit_all_phases(
        &self,
        meta: &TableMetadata<'a>,
        conditions: &[ConditionClause<'a>],
        out: &mut Vec<SqlToken<'a>>
    ) {
        // 1. kind
        self.emit_kind(out);
        // 2. from (if any)
        self.emit_from(meta, out);
        // 3. body (set/values/insert columns...)
        self.emit_body(out); // TODO: swap body and conditions
        // 4. conditions (WHERE / AND / OR)
        for cond in conditions {
            // cond.to_tokens(out);
        }
    }
}

impl<'a> EmitKind<'a> for QueryEmitter<'a> {
    fn emit_kind(&self, out: &mut Vec<SqlToken<'a>>) {
        match self {
            Self::Select(ast) => ast.emit_kind(out),
            Self::Insert(ast) => ast.emit_kind(out),
            Self::Update(ast) => ast.emit_kind(out),
            Self::Delete(ast) => ast.emit_kind(out),
        }
    }
}

impl<'a> EmitFrom<'a> for QueryEmitter<'a> {
    fn emit_from<'b>(&self, meta: &TableMetadata<'b>, out: &mut Vec<SqlToken<'b>>) {
        match self {
            Self::Select(ast) => ast.emit_from(meta, out),
            Self::Insert(ast) => ast.emit_from(meta, out),
            Self::Update(_) => { },
            Self::Delete(ast) => ast.emit_from(meta, out)
        }
    }
}
impl<'a> EmitBody<'a> for QueryEmitter<'a> {
    fn emit_body(&self, out: &mut Vec<SqlToken<'a>>) {
        match self {
            Self::Select(ast) => ast.emit_body(out),
            Self::Insert(ast) => ast.emit_body(out),
            Self::Update(ast) => ast.emit_body(out),
            Self::Delete(_) => { /* delete has no body */ }
        }
    }
}
