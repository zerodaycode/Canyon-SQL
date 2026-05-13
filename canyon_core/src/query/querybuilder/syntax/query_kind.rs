#[derive(Default)]
pub enum QueryKind {
    #[default]
    Select,
    Insert,
    Update,
    Delete,
}

impl AsRef<str> for QueryKind {
    fn as_ref(&self) -> &str {
        match self {
            QueryKind::Select => "SELECT",
            QueryKind::Insert => "INSERT",
            QueryKind::Update => "UPDATE ",
            QueryKind::Delete => "DELETE ",
        }
    }
}
