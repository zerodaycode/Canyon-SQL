use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
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

    On,
    As,
    Like,
    Returning,
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
        };
        write!(f, "{}", i)
    }
}
