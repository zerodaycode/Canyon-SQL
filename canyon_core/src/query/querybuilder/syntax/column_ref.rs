#[derive(Debug, Clone)]
pub struct ColumnRef<'a> {
    pub name: &'a str,
    pub alias: Option<&'a str>,
}

impl<'a> ColumnRef<'a> {
    pub fn new(name: &'a str) -> Self {
        Self { name, alias: None }
    }
    pub fn aliased(name: &'a str, alias: &'a str) -> Self {
        Self { name, alias: Some(alias) }
    }
}
