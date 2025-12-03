use std::borrow::Cow;
use crate::query::querybuilder::syntax::column::ColumnRef;
use crate::query::querybuilder::syntax::emitter::{AsEmitBody, AsEmitFrom, AsEmitKind, AstProcessor, EmitBody, EmitFrom, EmitKind, ToSql};
use crate::query::querybuilder::syntax::having::HavingClause;
use crate::query::querybuilder::syntax::join::JoinClause;
use crate::query::querybuilder::syntax::order::OrderByClause;
use crate::query::querybuilder::syntax::symbol::Symbol;
use crate::query::querybuilder::syntax::table_metadata::TableMetadata;
use crate::query::querybuilder::syntax::tokens::{SqlToken, ToSqlTokens};

#[derive(Default)]
pub struct SelectAst<'a> {
    pub columns: Vec<ColumnRef<'a>>,
    pub joins: Vec<JoinClause<'a>>,
    pub order_by: Option<OrderByClause<'a>>,
    pub having: Option<HavingClause<'a>>,
    pub group_by: Vec<ColumnRef<'a>>, // TODO: ColumnRef
    pub limit: Option<u64>, // TODO: strong typing
    pub offset: Option<u64>,
}

impl<'a> SelectAst<'a> {
    pub const fn new() -> Self {
        Self {
            columns: Vec::new(),
            joins: Vec::new(),
            order_by: None,
            group_by: Vec::new(),
            having: None,
            limit: None,
            offset: None,
        }
    }
}

impl<'a> ToSql<'a> for SelectAst<'a> {}

impl<'a> AstProcessor for SelectAst<'a> {}

impl<'a> EmitKind<'a> for SelectAst<'a> {
    fn emit_kind(&self, out: &mut Vec<SqlToken<'a>>) {
        out.push(SqlToken::Keyword(Cow::Borrowed("SELECT")));
    }
}

impl<'a> EmitFrom<'a> for SelectAst<'a> {
    fn emit_from(&self, meta: &TableMetadata<'a>, out: &mut Vec<SqlToken<'a>>)  {
        // columns
        if self.columns.is_empty() {
            out.push(SqlToken::Symbol(Symbol::Asterisk));
        } else {
            for (i, col) in self.columns.iter().enumerate() {
                if i > 0 {
                    out.push(SqlToken::Symbol(Symbol::Comma));
                }
                col.to_tokens(out);
                // out.push(SqlToken::new_ident(col));
            }
        }
        // FROM
        out.push(SqlToken::new_keyword("FROM"));
        meta.to_tokens(out);

    }
}
impl<'a> EmitBody<'a> for SelectAst<'a> {
    fn emit_body(&self, _out: &mut Vec<SqlToken<'a>>) {
        // optional: ORDER BY, LIMIT, etc. left for child builder

        // joins (simplified)
        // for j in &self.joins {
        //     _out.push(SqlToken::new_keyword("LEFT JOIN")); // TODO: actual placeholder until
        //     // we bring the JoinClauses
        //     // TODO: out.push(SqlToken::new_ident(*j));
        // }
    }
}


// tell the system that SelectAst supports these phases:
impl<'a> AsEmitKind<'a> for SelectAst<'a> {
    fn as_emit_kind(&self) -> Option<&dyn EmitKind<'a>> { Some(self as &dyn EmitKind<'a>) }
}
impl<'a> AsEmitFrom<'a> for SelectAst<'a> {
    fn as_emit_from(&self) -> Option<&dyn EmitFrom<'a>> { Some(self as &dyn EmitFrom<'a>) }
}
impl<'a> AsEmitBody<'a> for SelectAst<'a> {
    fn as_emit_body(&self) -> Option<&dyn EmitBody<'a>> { Some(self as &dyn EmitBody<'a>) }
}
// impl<'a> AsEmitConditions<'a> for SelectAst<'a> {
//     fn as_emit_conditions(&self) -> Option<&dyn EmitConditions<'a>> { Some(self as &dyn EmitConditions<'a>) }
// }
