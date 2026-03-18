use crate::query::operators::{Comp, LikeKind};
use crate::query::querybuilder::syntax::keyword::Keyword;
pub(crate) use crate::query::querybuilder::syntax::symbol::Symbol;
use crate::query::querybuilder::syntax::tokens::SqlToken::{Ident, Number};
use std::borrow::Cow;

pub trait ToSqlTokens<'a> {
    fn to_tokens(&self) -> impl IntoIterator<Item = SqlToken<'a>> + 'a;
}

/// 'newtype' (strong type) for the SqlToken container
#[derive(Debug, Default)]
pub struct SqlTokens<'a>(Vec<SqlToken<'a>>);
impl<'a> SqlTokens<'a> {
    // our custom internal APIs over the underlying wrapped collection
    pub fn ident<I: Into<Cow<'a, str>>>(&mut self, ident: I) {
        self.0.push(Ident(ident.into()))
    }

    pub fn numeric<N: Into<NumberKind>>(&mut self, num: N) {
        self.0.push(Number(num.into()))
    }

    pub fn keyword(&mut self, kw: Keyword) {
        self.0.push(SqlToken::Keyword(kw))
    }

    pub fn operator(&mut self, op: Comp) {
        self.0.push(SqlToken::Operator(op))
    }

    pub fn symbol(&mut self, sym: Symbol) {
        self.0.push(SqlToken::Symbol(sym))
    }

    pub fn placeholder(&mut self, pl_kind: PlaceholderKind) {
        self.0.push(SqlToken::Placeholder(pl_kind))
    }

    pub fn inner(self) -> Vec<SqlToken<'a>> {
        self.0
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }
}

impl<'a> IntoIterator for SqlTokens<'a> {
    type Item = SqlToken<'a>;
    type IntoIter = std::vec::IntoIter<SqlToken<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a SqlTokens<'a> {
    type Item = &'a SqlToken<'a>;
    type IntoIter = std::slice::Iter<'a, SqlToken<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut SqlTokens<'a> {
    type Item = &'a mut SqlToken<'a>;
    type IntoIter = std::slice::IterMut<'a, SqlToken<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl<'a> Extend<SqlToken<'a>> for SqlTokens<'a> {
    fn extend<T: IntoIterator<Item = SqlToken<'a>>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

#[derive(Debug, PartialEq)]
pub enum SqlToken<'a> {
    Keyword(Keyword), // SELECT, WHERE, AND, OR, FROM, UPDATE, DELETE // TODO: model them as ctc
    WhiteSpace,
    Ident(Cow<'a, str>),          // a raw literal value
    Number(NumberKind),           // a raw literal numeric value
    Symbol(Symbol),               // =, ( ) , .
    Operator(Comp),               // Comp::Eq, Comp::GtEq...
    Placeholder(PlaceholderKind), // $1, ? , @P1
}

#[derive(Debug, PartialEq)]
pub enum NumberKind {
    Integer(usize),
    F32(f32),
    F64(f64),
}

#[derive(Debug, PartialEq)]
pub enum PlaceholderKind {
    Value(usize),
    Like(LikeKind, usize),
    Range(usize, usize),
}

impl<'a> SqlToken<'a> {
    pub(crate) fn new_ident(kw: &'a str) -> Self {
        Ident(Cow::from(kw))
    }
}

mod __impl {
    use super::*;
    use std::fmt::Display;

    impl From<usize> for NumberKind {
        fn from(value: usize) -> Self {
            NumberKind::Integer(value)
        }
    }

    impl From<u32> for NumberKind {
        fn from(value: u32) -> Self {
            NumberKind::Integer(value as usize)
        }
    }

    impl From<u64> for NumberKind {
        fn from(value: u64) -> Self {
            NumberKind::Integer(value as usize)
        }
    }

    impl From<f32> for NumberKind {
        fn from(value: f32) -> Self {
            NumberKind::F32(value)
        }
    }

    impl From<f64> for NumberKind {
        fn from(value: f64) -> Self {
            NumberKind::F64(value)
        }
    }

    impl Display for NumberKind {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                NumberKind::Integer(i) => write!(f, "{}", i),
                NumberKind::F32(f32_val) => write!(f, "{}", f32_val),
                NumberKind::F64(f64_val) => write!(f, "{}", f64_val),
            }
        }
    }
}
