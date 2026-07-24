use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Eq)]
pub enum Keyword {
    Select,
    Insert,
    Update,
    Delete,

    From,
    Into,

    Join,
    Left,
    Right,
    Inner,
    Outer,
    Full,
    FullOuter,

    On,
    As,
    Like,
    Returning,

    Where,
    And,
    Or,
    In,
    GroupBy,
    Having,
    Limit,
    OrderBy,
    Desc,
    Offset,
    Values,
    Set,
    Not,
    Cast,
    Concat,
    Distinct,
    Count,
    Output,
    Inserted,
}

impl Display for Keyword {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let i = match self {
            Keyword::Select => "SELECT",
            Keyword::Insert => "INSERT",
            Keyword::Update => "UPDATE",
            Keyword::Delete => "DELETE",
            Keyword::From => "FROM",
            Keyword::Into => "INTO",
            Keyword::On => "ON",
            Keyword::As => "AS",
            Keyword::Like => "LIKE",
            Keyword::Returning => "RETURNING",
            Keyword::Join => "JOIN",
            Keyword::Left => "LEFT",
            Keyword::Right => "RIGHT",
            Keyword::Inner => "INNER",
            Keyword::Outer => "OUTER",
            Keyword::Full => "FULL",
            Keyword::FullOuter => "FULL OUTER",
            Keyword::Where => "WHERE",
            Keyword::And => "AND",
            Keyword::Or => "OR",
            Keyword::In => "IN",
            Keyword::Desc => "DESC",
            Keyword::GroupBy => "GROUP BY",
            Keyword::Having => "HAVING",
            Keyword::Limit => "LIMIT",
            Keyword::OrderBy => "ORDER BY",
            Keyword::Offset => "OFFSET",
            Keyword::Values => "VALUES",
            Keyword::Set => "SET",
            Keyword::Not => "NOT",
            Keyword::Cast => "CAST",
            Keyword::Concat => "CONCAT",
            Keyword::Distinct => "DISTINCT",
            Keyword::Count => "COUNT",
            Keyword::Output => "OUTPUT",
            Keyword::Inserted => "INSERTED",
        };
        write!(f, "{}", i)
    }
}
