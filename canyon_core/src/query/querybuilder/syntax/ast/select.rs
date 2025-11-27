use std::borrow::Cow;
use crate::query::querybuilder::syntax::column::ColumnRef;
use crate::query::querybuilder::syntax::emitter::{AstProcessor, EmitBody, EmitFrom, EmitKind};
use crate::query::querybuilder::syntax::having::HavingClause;
use crate::query::querybuilder::syntax::join::JoinClause;
use crate::query::querybuilder::syntax::order::OrderByClause;
use crate::query::querybuilder::syntax::table_metadata::TableMetadata;
use crate::query::querybuilder::syntax::tokens::{SqlToken, Symbol, ToSqlTokens};

#[derive(Default)]
pub struct SelectAst<'a> {
    pub columns: Vec<ColumnRef<'a>>,
    pub joins: Vec<JoinClause<'a>>,
    pub group_by: Vec<&'a str>,
    pub having: Vec<HavingClause<'a>>,
    pub order_by: Vec<OrderByClause<'a>>,
    pub limit: Option<u64>, // TODO: strong typing
    pub offset: Option<u64>,
}

impl<'a> SelectAst<'a> {
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            joins: Vec::new(),
            group_by: Vec::new(),
            having: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
        }
    }
}

impl<'a> AstProcessor for SelectAst<'a> {}

impl<'a> EmitKind<'a> for SelectAst<'a> {
    fn emit_kind(&self, out: &mut Vec<SqlToken<'a>>) {
        out.push(SqlToken::Keyword(Cow::Borrowed("SELECT")));
    }
}

impl<'a> EmitFrom<'a> for SelectAst<'a> {
    fn emit_from<'b>(&self, meta: &TableMetadata<'b>, out: &mut Vec<SqlToken<'b>>)  {
        // columns
        if self.columns.is_empty() {
            out.push(SqlToken::Symbol(Symbol::Asterisk));
        } else {
            for (i, c) in self.columns.iter().enumerate() {
                if i > 0 { out.push(SqlToken::Symbol(Symbol::Comma)); }
                // TODO: out.push(SqlToken::new_ident(c));
            }
        }
        // FROM
        out.push(SqlToken::new_keyword("FROM"));
        meta.to_tokens(out);

        // joins (simplified)
        for j in &self.joins {
            out.push(SqlToken::new_keyword("LEFT JOIN")); // TODO: actual placeholder until
            // we bring the JoinClauses
            // TODO: out.push(SqlToken::new_ident(*j));
        }
    }
}
impl<'a> EmitBody<'a> for SelectAst<'a> {
    fn emit_body(&self, _out: &mut Vec<SqlToken<'a>>) {
        // optional: ORDER BY, LIMIT, etc. left for child builder
    }
}
