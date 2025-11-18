use std::borrow::Cow;
use crate::query::querybuilder::syntax::tokens::{SqlToken, ToSqlTokens};

pub enum QueryKind {
    Select,
    Update,
    Delete,
}

impl<'a> ToSqlTokens<'a> for QueryKind {
    fn to_tokens(&self) -> SqlToken<'a> {
        match self {
            QueryKind::Select => SqlToken::Keyword(Cow::from("SELECT")),
            QueryKind::Update => SqlToken::Keyword(Cow::from("UPDATE")),
            QueryKind::Delete => SqlToken::Keyword(Cow::from("DELETE")),
        }
    }
}
impl AsRef<str> for QueryKind {
    fn as_ref(&self) -> &str {
        match self {
            QueryKind::Select => { "SELECT" }
            QueryKind::Update => { "UPDATE " }
            QueryKind::Delete => { "DELETE " }
        }
    }
}
