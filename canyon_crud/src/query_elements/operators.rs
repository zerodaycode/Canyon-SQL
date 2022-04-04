pub enum Comp {
    Eq,
    Neq,
    Gt,
    Lt,
}

impl Comp {
    pub fn as_string(&self) -> String {
        match *self {
            Self::Eq => " = ".to_string(),
            Self::Neq => " <> ".to_string(),
            Self::Gt => " >= ".to_string(),
            Self::Lt => " <= ".to_string()
        }
    }
}