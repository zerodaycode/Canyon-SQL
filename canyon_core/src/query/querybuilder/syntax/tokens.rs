use crate::connection::database_type::DatabaseType;
use crate::query::operators::Comp;
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
    Placeholder(usize),  // $1, ? , @P1
}

impl<'a> SqlToken<'a> {
    pub(crate) fn new_keyword(kw: &'a str) -> Self {
        Keyword(Cow::from(kw))
    }

    pub(crate) fn new_ident(kw: &'a str) -> Self {
        Ident(Cow::from(kw))
    }
}

pub struct TokenWriter {}

impl TokenWriter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render<'a>(
        self,
        tokens: &[SqlToken<'a>],
        db: &'a DatabaseType,
    ) -> Result<String, std::fmt::Error> {
        let mut out = String::new();

        for tok in tokens {
            __impl::output_token_to_string_buffer(tok, db, &mut out)?; // TODO: split, for db and others (maybe)
        }

        Ok(out.trim_start().to_string())
    }
}

mod __impl {
    use crate::connection::database_type::DatabaseType;
    use crate::query::querybuilder::syntax::tokens::{SqlToken, Symbol};
    use std::fmt::Write;

    pub(crate) fn output_token_to_string_buffer(
        token: &SqlToken,
        db: &DatabaseType,
        f: &mut String,
    ) -> Result<(), std::fmt::Error> {
        let _: () = match token {
            SqlToken::Keyword(s) => write!(f, " {}", s)?,
            SqlToken::Ident(s) => write!(f, " {}", s)?,

            SqlToken::Symbol(sym) => match sym {
                Symbol::Comma => write!(f, ",")?,
                Symbol::LParen => write!(f, " (")?,
                Symbol::RParen => write!(f, ")")?,
                Symbol::Dot => write!(f, ".")?,
                Symbol::Semicolon => write!(f, ";")?,
                Symbol::Equals => write!(f, " =")?,
                Symbol::Asterisk => write!(f, " *")?,
            },

            SqlToken::Operator(op) => write!(f, " {}", op)?,

            SqlToken::Placeholder(ph_idx) => match db {
                DatabaseType::PostgreSql => write!(f, " ${}", ph_idx)?,
                DatabaseType::SqlServer => write!(f, " @P{}", ph_idx)?,
                DatabaseType::MySQL => write!(f, " ?")?,
                _ => write!(f, " ?")?,
            },
            _ => {}
        };
        Ok(())
    }
}
