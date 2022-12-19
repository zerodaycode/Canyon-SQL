pub trait Operator {
    fn as_str<'a>(&'a self) -> &'static str;
}

/// Enumerated type for represent the comparison operations
/// in SQL sentences
pub enum Comp {
    /// Operator "=" equals
    Eq,
    /// Operator "!=" not equals
    Neq,
    /// Operator ">" greather than <value>
    Gt,
    /// Operator ">=" greather or equals than <value>
    GtEq,
    /// Operator "<" less than <value>
    Lt,
    /// Operator "=<" less or equals than <value>
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

