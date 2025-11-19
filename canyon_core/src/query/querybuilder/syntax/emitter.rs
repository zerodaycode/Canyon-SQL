use crate::connection::database_type::DatabaseType;
use crate::query::parameters::QueryParameter;
use crate::query::querybuilder::syntax::tokens::{SqlToken};
use crate::query::querybuilder::ast::{BaseAst, SelectAst, UpdateAst, InsertAst, DeleteAst};
use crate::query::querybuilder::syntax::clause::ConditionClauseKind;

// Trait each AST piece can optionally implement to emit tokens (we'll implement per-AST)
pub trait EmitTokens<'a> {
    fn emit_tokens(&self, out: &mut Vec<SqlToken<'a>>);
}

use crate::query::querybuilder::syntax::table_metadata::TableMetadata;
use crate::query::querybuilder::syntax::query_kind::QueryKind;
use crate::query::querybuilder::syntax::column::ColumnRef;
use crate::query::querybuilder::syntax::join::JoinClause;
use crate::query::querybuilder::syntax::order::OrderByClause;
use crate::query::querybuilder::syntax::having::HavingClause;
use crate::query::querybuilder::syntax::clause::ConditionClause;

// Helper: emit table
fn emit_table<'a>(table: &TableMetadata, out: &mut Vec<SqlToken<'a>>) {
    if let Some(s) = &table.schema {
        out.push(SqlToken::Ident(s));
        out.push(SqlToken::Symbol("."));
    }
    out.push(SqlToken::Ident(&table.name));
}

impl<'a> EmitTokens<'a> for BaseAst<'a> {
    fn emit_tokens(&self, out: &mut Vec<SqlToken<'a>>) {
        // query kind handled by caller usually
        // emit WHERE/conditions if present (deferred placeholders handled by caller)
        if !self.conditions.is_empty() {
            for (i, cond) in self.conditions.iter().enumerate() {
                // prefix: first cond -> WHERE else -> AND / OR / IN kind
                let prefix = if i == 0 { cond.kind.as_str() } else { cond.kind.as_str() };
                out.push(SqlToken::Keyword(prefix));
                out.push(SqlToken::Ident(cond.column_name));
                // for IN, operator text already is "IN", and we will add placeholders externally
                if cond.kind == ConditionClauseKind::In {
                    out.push(SqlToken::Operator("IN"));
                    out.push(SqlToken::Symbol("("));
                    // placeholders will be appended by the caller (because IN has multiple params)
                    out.push(SqlToken::Symbol(")"));
                } else {
                    out.push(SqlToken::Operator(cond.operator.as_str()));
                    // placeholder token appended by caller emitter once param index known
                }
            }
        }
    }
}

impl<'a> EmitTokens<'a> for SelectAst<'a> {
    fn emit_tokens(&self, out: &mut Vec<SqlToken<'a>>) {
        // SELECT clause
        out.push(SqlToken::Keyword("SELECT"));
        if self.columns.is_empty() {
            out.push(SqlToken::Symbol("*"));
        } else {
            for (i, col) in self.columns.iter().enumerate() {
                if i > 0 { out.push(SqlToken::Symbol(",")); }
                out.push(SqlToken::Ident(col.name));
                if let Some(alias) = col.alias { out.push(SqlToken::Keyword("AS")); out.push(SqlToken::Ident(alias)); }
            }
        }

        // FROM
        out.push(SqlToken::Keyword("FROM"));
        emit_table(&self.base.table, out);

        // JOINs
        for j in &self.joins {
            out.push(SqlToken::Keyword(j.kind.as_str()));
            emit_table(&j.table, out);
            out.push(SqlToken::Keyword("ON"));
            out.push(SqlToken::Ident(j.left));
            out.push(SqlToken::Operator(j.operator.as_str()));
            out.push(SqlToken::Ident(j.right));
        }

        // WHERE / HAVING / GROUP BY / ORDER BY / LIMIT / OFFSET -> handled by base and specific
        self.base.emit_tokens(out);

        if !self.group_by.is_empty() {
            out.push(SqlToken::Keyword("GROUP BY"));
            for (i, col) in self.group_by.iter().enumerate() {
                if i > 0 { out.push(SqlToken::Symbol(",")); }
                out.push(SqlToken::Ident(col));
            }
        }

        if !self.having.is_empty() {
            out.push(SqlToken::Keyword("HAVING"));
            for (i, h) in self.having.iter().enumerate() {
                if i > 0 { out.push(SqlToken::Keyword("AND")); }
                out.push(SqlToken::Ident(h.column));
                out.push(SqlToken::Operator(h.operator.as_str()));
                // placeholder: appended by caller with param index
            }
        }

        if !self.order_by.is_empty() {
            out.push(SqlToken::Keyword("ORDER BY"));
            for (i, ob) in self.order_by.iter().enumerate() {
                if i > 0 { out.push(SqlToken::Symbol(",")); }
                out.push(SqlToken::Ident(ob.column));
                if ob.descending { out.push(SqlToken::Keyword("DESC")); }
            }
        }

        if let Some(limit) = self.limit {
            out.push(SqlToken::Keyword("LIMIT"));
            out.push(SqlToken::Number(limit));
        }
        if let Some(offset) = self.offset {
            out.push(SqlToken::Keyword("OFFSET"));
            out.push(SqlToken::Number(offset));
        }
    }
}

impl<'a> EmitTokens<'a> for UpdateAst<'a> {
    fn emit_tokens(&self, out: &mut Vec<SqlToken<'a>>) {
        out.push(SqlToken::Keyword("UPDATE"));
        emit_table(&self.base.table, out);
        out.push(SqlToken::Keyword("SET"));
        for (i, (col, _val)) in self.set_clauses.iter().enumerate() {
            if i > 0 { out.push(SqlToken::Symbol(",")); }
            out.push(SqlToken::Ident(col));
            out.push(SqlToken::Operator("="));
            // placeholder appended by caller
        }
        self.base.emit_tokens(out);
    }
}

impl<'a> EmitTokens<'a> for DeleteAst<'a> {
    fn emit_tokens(&self, out: &mut Vec<SqlToken<'a>>) {
        out.push(SqlToken::Keyword("DELETE"));
        out.push(SqlToken::Keyword("FROM"));
        emit_table(&self.base.table, out);
        self.base.emit_tokens(out);
    }
}

impl<'a> EmitTokens<'a> for InsertAst<'a> {
    fn emit_tokens(&self, out: &mut Vec<SqlToken<'a>>) {
        out.push(SqlToken::Keyword("INSERT INTO"));
        emit_table(&self.base.table, out);
        if !self.columns.is_empty() {
            out.push(SqlToken::Symbol("("));
            for (i, c) in self.columns.iter().enumerate() {
                if i > 0 { out.push(SqlToken::Symbol(",")); }
                out.push(SqlToken::Ident(c));
            }
            out.push(SqlToken::Symbol(")"));
        }
        out.push(SqlToken::Keyword("VALUES"));
        out.push(SqlToken::Symbol("("));
        // placeholders appended by caller
        out.push(SqlToken::Symbol(")"));
    }
}