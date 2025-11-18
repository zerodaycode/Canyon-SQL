use std::borrow::Cow;
use std::fmt::Write;
use crate::connection::database_type::DatabaseType;

pub trait ToSqlTokens<'a> {
    fn to_tokens(&self) -> SqlToken<'a>;
}

pub enum SqlToken<'a> {
    Keyword(Cow<'a, str>),        // SELECT, WHERE, AND, OR, FROM, UPDATE, DELETE
    WhiteSpace,
    Ident(Cow<'a, str>),          // table, column
    Symbol(char),            // = , ( ) , .
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

    pub fn render(self, db: DatabaseType) -> String {
        let mut out = String::new();

        for t in self.tokens {
            match t {
                SqlToken::Keyword(k) => write!(out, " {}", k).unwrap(),
                SqlToken::Ident(i) => write!(out, " {}", i).unwrap(),
                SqlToken::Symbol(c) => write!(out, " {}", c).unwrap(),
                SqlToken::Placeholder(p) => match db { // TODO: invent something like placeholder kind, so we can avoid to match it on every pplaceholder?
                    DatabaseType::PostgreSql => write!(out, " ${}", p).unwrap(),
                    DatabaseType::SqlServer  => write!(out, " @P{}", p).unwrap(),
                    DatabaseType::MySQL      => write!(out, " ?").unwrap(),
                    DatabaseType::Deferred   => write!(out, " ?").unwrap(),
                },
            }
        }

        out.trim_start().to_string()
    }
}