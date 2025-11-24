use std::borrow::Cow;
use crate::query::querybuilder::syntax::ast::select::SelectAst;
use crate::query::querybuilder::syntax::clause::ConditionClause;
use crate::query::querybuilder::syntax::query_kind::QueryKind;
use crate::query::querybuilder::syntax::table_metadata::TableMetadata;
use crate::query::querybuilder::syntax::tokens::{SqlToken, Symbol, ToSqlTokens};
use crate::query::querybuilder::syntax::tokens::Symbol::{Comma, LParen, RParen};

// ---------- AST Processor marker trait ----------
pub trait AstProcessor: Default {} // TODO: maybe this and the other one are visitor related?

// ---------- Emit traits ----------
pub trait EmitKind<'a> {
    fn emit_kind(&self, out: &mut Vec<SqlToken<'a>>);
}
pub trait EmitFrom<'a> {
    fn emit_from<'b: 'a>(&'b self, meta: &'b TableMetadata<'b>, out: &mut Vec<SqlToken<'b>>) ;
}
pub trait EmitBody<'a> {
    fn emit_body(&self, out: &mut Vec<SqlToken<'a>>);
}

#[derive(Debug)]
pub struct DeleteEmitter;
impl<'a> EmitKind<'a> for DeleteEmitter {
    fn emit_kind(&self, out: &mut Vec<SqlToken<'a>>) {
        out.push(SqlToken::new_keyword("DELETE"));
    }
}
impl<'a> EmitFrom<'a> for DeleteEmitter {
    fn emit_from<'b: 'a>(&'b self, meta: &'b TableMetadata<'b>, out: &mut Vec<SqlToken<'b>>) {
        out.push(SqlToken::new_keyword("FROM"));
        meta.to_tokens(out);
    }
}
// no body for delete

#[derive(Debug)]
pub struct UpdateEmitter<'a> {
    pub set_clauses: Vec<(&'a str, &'a str)>, // TODO: better placeholders? params values are already on the base container
    // pub set_clauses: Vec<(&'a str, &'a dyn QueryParameter)>, // TODO: better placeholders? params values are already on the base container
    // TODO: this should be a tuple of ColumnRef and a Placeholder
}
impl<'a> EmitKind<'a> for UpdateEmitter<'a> {
    fn emit_kind(&self, out: &mut Vec<SqlToken<'a>>) {
        out.push(SqlToken::Keyword(Cow::Borrowed("UPDATE")));
    }
}
impl<'a> EmitBody<'a> for UpdateEmitter<'a> {
    fn emit_body(&self, out: &mut Vec<SqlToken<'a>>) {
        out.push(SqlToken::new_keyword("SET"));
        for (i, (col, _val)) in self.set_clauses.iter().enumerate() {
            if i > 0 { out.push(SqlToken::Symbol(Symbol::Comma)); }
            out.push(SqlToken::new_ident(col));
            out.push(SqlToken::Symbol(Symbol::Equals));
            // out.push(SqlToken::PlaceholderNext); // TODO: emit placeholder
            // TODO: create a counter of the
        }
    }
}

#[derive(Debug)]
pub struct InsertEmitter<'a> {
    pub columns: Vec<&'a str>,
    pub values_count: usize,
}
impl<'a> EmitKind<'a> for InsertEmitter<'a> {
    fn emit_kind(&self, out: &mut Vec<SqlToken<'a>>) {
        out.push(SqlToken::new_keyword("INSERT"));
    }
}
impl<'a> EmitFrom<'a> for InsertEmitter<'a> {
    fn emit_from<'b: 'a>(&'b self, meta: &'b TableMetadata<'b>, out: &mut Vec<SqlToken<'b>>) {
        meta.to_tokens(out);
        if !self.columns.is_empty() {
            out.push(SqlToken::new_ident("INTO"));
            out.push(SqlToken::Symbol(LParen));
            for (i, c) in self.columns.iter().enumerate() {
                if i > 0 { out.push(SqlToken::Symbol(Symbol::Comma)); }
                out.push(SqlToken::new_ident(c));
            }
            out.push(SqlToken::Symbol(Symbol::RParen));
        }
    }
}
impl<'a> EmitBody<'a> for InsertEmitter<'a> {
    fn emit_body(&self, out: &mut Vec<SqlToken<'a>>) {
        out.push(SqlToken::Keyword(Cow::Borrowed("VALUES")));
        out.push(SqlToken::Symbol(LParen));
        for i in 0..self.values_count {
            if i > 0 { out.push(SqlToken::Symbol(Comma)); }
            // out.push(SqlToken::PlaceholderNext); self as the real type and call a custom impl
        }
        out.push(SqlToken::Symbol(RParen));
    }
}

// // ---------- QueryEmitter enum ----------
// pub enum QueryEmitter<'a> { // Isn't this almost queryKind?
//     // Raw(BaseAst<'a>),
//     Select(SelectAst<'a>),
//     Insert(InsertEmitter<'a>),
//     Update(UpdateEmitter<'a>),
//     Delete(DeleteEmitter),
// }
// ---------- QueryEmitter enum ----------
pub struct QueryEmitter<P: AstProcessor> { // Isn't this almost queryKind?
    kind: QueryKind,
    ast: P,
}

impl<'a, P: AstProcessor> QueryEmitter<P> {
    pub fn new(kind: QueryKind) -> Self {
        Self {
            kind,
            ast: P::default()
        }
    }
    
    pub fn emit_kind(&self, out: &mut Vec<SqlToken<'a>>) {
        self.emit_kind(out) // inner impl
    }

    // only variants that support FROM are matched here
    pub fn emit_from<'b: 'a>(&'b self, meta: &'b TableMetadata<'b>, out: &mut Vec<SqlToken<'b>>)  {
        match self.kind {
            QueryKind::Select | QueryKind::Insert| QueryKind::Delete=> self.emit_from(meta, out),
            QueryKind::Update => { /* Update has no FROM here */ }
        }
    }

    pub fn emit_body(&self, out: &mut Vec<SqlToken<'a>>) {
        match self.kind {
            QueryKind::Select | QueryKind::Insert| QueryKind::Update => self.emit_body(out),
            QueryKind::Delete => { /* delete has no body */ }
        }
    }

    /// façade: executes all phases in logical order: KIND -> FROM -> BODY -> CONDITIONS
    pub fn emit_all_phases(
        &'a self,
        meta: &'a TableMetadata<'a>,
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
