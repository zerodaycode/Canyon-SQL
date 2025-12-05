use crate::connection::database_type::DatabaseType;
use crate::query::operators::{Comp, LikeKind};
pub(crate) use crate::query::querybuilder::syntax::symbol::Symbol;
use crate::query::querybuilder::syntax::tokens::SqlToken::{Ident, Keyword};
use std::borrow::Cow;

pub trait ToSqlTokens<'a> {
    fn to_tokens(&self, out: &mut Vec<SqlToken<'a>>);
}

#[derive(Debug, PartialEq)]
pub enum SqlToken<'a> {
    Keyword(Cow<'a, str>), // SELECT, WHERE, AND, OR, FROM, UPDATE, DELETE // TODO: model them as ctc
    WhiteSpace,
    Ident(Cow<'a, str>), // table, column
    Symbol(Symbol),      // =, ( ) , .
    Operator(Comp),      // Comp::Eq, Comp::GtEq...
    Placeholder(PlaceholderKind),  // $1, ? , @P1
}

#[derive(Debug, PartialEq)]
pub enum PlaceholderKind {
    Value(usize),
    Like(LikeKind, usize),
    Range(usize, usize)
}

impl<'a> SqlToken<'a> {
    pub(crate) fn new_keyword(kw: &'a str) -> Self {
        Keyword(Cow::from(kw))
    }

    pub(crate) fn new_ident(kw: &'a str) -> Self {
        Ident(Cow::from(kw))
    }
}
