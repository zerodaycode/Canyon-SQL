
#[derive(Default)]
pub enum QueryKind {
    #[default] Select,
    Insert,
    Update,
    Delete,
}

// impl<'a> ToSqlTokens<'a> for QueryKind {
//     fn to_tokens(&self) -> SqlToken<'a> {
//         match self {
//             QueryKind::Select => SqlToken::Keyword(Cow::from("SELECT")),
//             QueryKind::Insert => SqlToken::Keyword(Cow::from("INSERT")),
//             QueryKind::Update => SqlToken::Keyword(Cow::from("UPDATE")),
//             QueryKind::Delete => SqlToken::Keyword(Cow::from("DELETE")),
//         }
//     }
// }
impl AsRef<str> for QueryKind {
    fn as_ref(&self) -> &str {
        match self {
            QueryKind::Select => { "SELECT" }
            QueryKind::Insert => { "INSERT" }
            QueryKind::Update => { "UPDATE " }
            QueryKind::Delete => { "DELETE " }
        }
    }
}
