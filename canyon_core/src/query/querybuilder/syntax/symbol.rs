use crate::query::querybuilder::syntax::dialect::IdentQuoting;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Symbol {
    Not,
    LParen,
    RParen,
    Apostrophe,
    Comma,
    Dot,
    Equals,
    Semicolon,
    Asterisk,
    LAngle,
    RAngle,
    PercentSign,
    Quote,
    DoubleQuote,
    Backtick,
    LBracket,
    RBracket,
    Backslash,

    Empty, //<-- Special symbol to represent an empty symbol, used for cases where we want to represent the absence of a symbol without using Option<Symbol>
}

impl From<IdentQuoting> for Symbol {
    fn from(quoting: IdentQuoting) -> Self {
        match quoting {
            IdentQuoting::DoubleQuote => Symbol::DoubleQuote,
            IdentQuoting::Backtick => Symbol::Backtick,
            IdentQuoting::OpeningBracket => Symbol::LBracket,
            IdentQuoting::ClosingBracket => Symbol::RBracket,
        }
    }
}
