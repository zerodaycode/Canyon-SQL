use crate::query::querybuilder::syntax::dialect::SqlDialect;
pub(crate) use crate::query::{
    operators::{LikeKind, Operator},
    querybuilder::syntax::{keyword::Keyword, symbol::Symbol, tokens::SqlToken::Number},
};
use std::borrow::Cow;

pub trait ToSqlTokens<'a, D: SqlDialect> {
    fn to_tokens(&self) -> impl IntoIterator<Item = SqlToken<'a>> + 'a;
}

/// 'newtype' (strong type) for the SqlToken container
#[derive(Debug, Default)]
pub struct SqlTokens<'a>(Vec<SqlToken<'a>>);
impl<'a> SqlTokens<'a> {
    // our custom internal APIs over the underlying wrapped collection
    pub fn ident<S>(&mut self, ident: S) -> &mut Self
    where
        S: Into<Cow<'a, str>>,
    {
        self.0.push(SqlToken::Ident(ident.into()));
        self
    }

    pub fn numeric<N: Into<NumberKind>>(&mut self, num: N) {
        self.0.push(Number(num.into()))
    }

    pub fn keyword(&mut self, kw: Keyword) {
        self.0.push(SqlToken::Keyword(kw))
    }

    pub fn operator(&mut self, op: Operator) {
        self.0.push(SqlToken::Operator(op))
    }

    pub fn symbol(&mut self, sym: Symbol) {
        self.0.push(SqlToken::Symbol(sym))
    }

    pub fn placeholder(&mut self, pl_kind: PlaceholderKind) {
        self.0.push(SqlToken::Placeholder(pl_kind))
    }

    /// Adds a [`SqlToken::WhiteSpace`] to the output buffer
    pub fn whitespace(&mut self) {
        self.0.push(SqlToken::WhiteSpace)
    }

    pub fn inner(self) -> Vec<SqlToken<'a>> {
        self.0
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn first(&self) -> Option<&SqlToken<'a>> {
        self.0.first()
    }

    pub fn last(&self) -> Option<&SqlToken<'a>> {
        self.0.last()
    }

    pub fn remove_last_if<F>(&mut self, predicate: F) -> Option<SqlToken<'a>>
    where
        F: FnOnce(&SqlToken<'a>) -> bool,
    {
        if let Some(last) = self.0.last()
            && predicate(last)
        {
            return self.0.pop();
        }
        None
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

impl<'a> Extend<SqlToken<'a>> for &'a mut SqlTokens<'a> {
    fn extend<T: IntoIterator<Item = SqlToken<'a>>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SqlToken<'a> {
    Keyword(Keyword), // SELECT, WHERE, AND, OR, FROM, UPDATE, DELETE // TODO: model them as ctc
    WhiteSpace,
    Ident(Cow<'a, str>),          // a raw literal value
    Number(NumberKind),           // a raw literal numeric value
    Symbol(Symbol),               // =, ( ) , .
    Operator(Operator),           // Comp::Eq, Comp::GtEq...
    Placeholder(PlaceholderKind), // $1, ? , @P1
}

#[derive(Debug, PartialEq, Eq)]
pub enum NumberKind {
    Integer(usize),
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum PlaceholderKind {
    Value(usize),          // $1, ? , @P1
    Like(LikeKind, usize), // for LIKE placeholders, we need to know the kind of LIKE (e.g., starts with, ends with, contains) and the index of the placeholder
    Range(usize, usize), // for range placeholders, we need to know the start and end index of the range (e.g., for BETWEEN ? AND ?, we need to know the indices of both placeholders)
}

mod __impl_sql_token {
    use super::*;
    use crate::query::querybuilder::syntax::dialect::IdentQuoting;

    impl<'a> From<IdentQuoting> for SqlToken<'a> {
        fn from(quoting: IdentQuoting) -> Self {
            match quoting {
                IdentQuoting::Backtick => SqlToken::Symbol(Symbol::Backtick),
                IdentQuoting::DoubleQuote => SqlToken::Symbol(Symbol::DoubleQuote),
                IdentQuoting::OpeningBracket => SqlToken::Symbol(Symbol::LBracket), // Note: we use LBracket for both [ and ] since they are used in pairs
                IdentQuoting::ClosingBracket => SqlToken::Symbol(Symbol::RBracket), // Note: we use LBracket for both [ and ] since they are used in pairs
            }
        }
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

    impl Display for NumberKind {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                NumberKind::Integer(i) => write!(f, "{}", i),
            }
        }
    }
}
