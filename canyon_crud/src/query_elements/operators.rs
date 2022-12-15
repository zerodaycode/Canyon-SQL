pub trait Operator {
    fn as_str<'a>(&'a self) -> &'static str;
}

/// Enumerated type for represent the comparison operations
/// in SQL sentences
pub enum Comp {
    Eq,
    Neq,
    Gt,
    GtEq,
    Lt,
    LtEq,
}
impl Operator for Comp {
    fn as_str<'a>(&'a self) -> &'static str {
        match *self {
            Self::Eq => " = ",
            Self::Neq => " <> ",
            Self::Gt => " > ",
            Self::GtEq => " >= ",
            Self::Lt => " < ",
            Self::LtEq => " <= ",
        }
    }
}

