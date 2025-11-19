use std::borrow::Cow;
use std::fmt::Write;
use crate::connection::database_type::DatabaseType;

pub trait ToSqlTokens<'a> {
    fn to_tokens(&self) -> SqlToken<'a>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum Symbol {
    LParen,
    RParen,
    Comma,
    Dot,
    Semicolon,
}

pub enum SqlToken<'a> {
    Keyword(Cow<'a, str>),        // SELECT, WHERE, AND, OR, FROM, UPDATE, DELETE
    WhiteSpace,
    Ident(Cow<'a, str>),          // table, column
    Symbol(Symbol),            // = , ( ) , .
    Placeholder(usize),      // $1, ? , @P1
}

pub struct TokenWriter<'a> {
    pub tokens: Vec<SqlToken<'a>>,
}

impl<'a> TokenWriter<'a> {
    pub fn new() -> Self {
        Self { tokens: Vec::new() }
    }

    pub fn push(&mut self, token: SqlToken<'a>) {
        self.tokens.push(token);
    }

    pub fn render(self, db: DatabaseType) -> Result<String, std::fmt::Error> {
        let mut out = String::new();

        for tok in self.tokens {
            match tok {
                SqlToken::Keyword(s) => write!(out, " {}", s)?,
                SqlToken::Ident(s)   => write!(out, " {}", s)?,

                SqlToken::Symbol(sym) => match sym {
                    Symbol::Comma     => write!(out, ",")?,
                    Symbol::LParen    => write!(out, " (")?,
                    Symbol::RParen    => write!(out, ")")?,
                    Symbol::Dot       => write!(out, ".")?,
                    Symbol::Semicolon => write!(out, ";")?,
                },

                SqlToken::Operator(op) =>
                    write!(out, " {}", op.as_str())?,

                SqlToken::Placeholder => {
                    ph_idx += 1;
                    match db {
                        DatabaseType::PostgreSql => write!(out, " ${}", ph_idx)?,
                        DatabaseType::SqlServer  => write!(out, " @P{}", ph_idx)?,
                        DatabaseType::MySQL      => write!(out, " ?")?,
                        _ => write!(out, " ?")?,
                    }
                }

                SqlToken::Number(n) =>
                    write!(out, " {}", n)?,

                SqlToken::Raw(r) =>
                    write!(out, " {}", r)?,
            }
        }

        out.trim_start().to_string()
    }
}