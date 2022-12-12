pub enum Comp {
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
}

impl Comp {
    pub fn as_string(&self) -> String {
        match *self {
            Self::Eq => " = ".to_string(),
            Self::Neq => " <> ".to_string(),
            Self::Gt => " > ".to_string(),
            Self::Gte => " >= ".to_string(),
            Self::Lt => " < ".to_string(),
            Self::Lte => " <= ".to_string(),
        }
    }
}
