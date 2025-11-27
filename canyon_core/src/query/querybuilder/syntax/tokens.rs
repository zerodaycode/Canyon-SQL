use std::borrow::Cow;
use std::fmt::{Display, Write};
use crate::connection::database_type::DatabaseType;
use crate::query::operators::Comp;
use crate::query::querybuilder::syntax::tokens::SqlToken::{Ident, Keyword};

pub trait ToSqlTokens<'a> {
    fn to_tokens(&self, out: &mut Vec<SqlToken<'a>>);
}

pub trait ToSql<'a>: Display + ToSqlTokens<'a> {
    /// Writes in the past in buffer the string representation of any 
    fn write_as_sql(&self, out: &mut String); // Strong typing over the string
}

#[derive(Debug, Clone, PartialEq)]
pub enum Symbol {
    LParen,
    RParen,
    Comma,
    Dot,
    Equals,
    Semicolon,
    Asterisk,
}

#[derive(Debug)]
pub enum SqlToken<'a> {
    Keyword(Cow<'a, str>),        // SELECT, WHERE, AND, OR, FROM, UPDATE, DELETE // TODO: model them as ctc
    WhiteSpace,
    Ident(Cow<'a, str>),          // table, column
    Symbol(Symbol),            // =, ( ) , .
    Operator(Comp),            // Comp::Eq, Comp::GtEq...
    Placeholder(usize),      // $1, ? , @P1
}

impl<'a> SqlToken<'a> {
    pub(crate) fn new_keyword(kw: &'a str) -> Self {
        Keyword(Cow::from(kw))
    }

    pub(crate) fn new_ident(kw: &'a str) -> Self {
        Ident(Cow::from(kw))
    }
}


pub struct TokenWriter {
}

impl TokenWriter {
    pub fn new() -> Self {
        Self {  }
    }

    pub fn render<'a>(self, tokens: &[SqlToken<'a>], db: &'a DatabaseType) -> Result<String, std::fmt::Error> {
        let mut out = String::new();

        for tok in tokens {
            match tok {
                SqlToken::Keyword(s) => write!(out, " {}", s)?,
                SqlToken::Ident(s)   => write!(out, " {}", s)?,

                SqlToken::Symbol(sym) => match sym {
                    Symbol::Comma     => write!(out, ",")?,
                    Symbol::LParen    => write!(out, " (")?,
                    Symbol::RParen    => write!(out, ")")?,
                    Symbol::Dot       => write!(out, ".")?,
                    Symbol::Semicolon => write!(out, ";")?,
                    _ => {}
                },

                SqlToken::Operator(op) =>
                    write!(out, " {}", op)?,

                SqlToken::Placeholder(ph_idx) => {
                    match db {
                        DatabaseType::PostgreSql => write!(out, " ${}", ph_idx)?,
                        DatabaseType::SqlServer  => write!(out, " @P{}", ph_idx)?,
                        DatabaseType::MySQL      => write!(out, " ?")?,
                        _ => write!(out, " ?")?,
                    }
                }
                //
                // SqlToken::Number(n) =>
                //     write!(out, " {}", n)?,
                //
                // SqlToken::Raw(r) =>
                //     write!(out, " {}", r)?,
                _ => {}
            }
        }

        Ok(out.trim_start().to_string())
    }
}