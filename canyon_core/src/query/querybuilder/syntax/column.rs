#[derive(Debug, Clone)]
pub struct ColumnRef<'a> {
    pub column: &'a str,
    pub table: Option<&'a str>,
    // TODO: we need the table <table.column> no?
    // TODO: so we need a ColumnRefBuilder, to avoid have 70 new different new methods?
    pub alias: Option<&'a str>,
}

impl<'a> From<&'a str> for ColumnRef<'a> {
    fn from(value: &'a str) -> Self {
        Self::new(value)
    }
}

impl<'a> ColumnRef<'a> {
    pub fn new(column_name: &'a str) -> Self {
        Self { column: column_name, table: None, alias: None }
    }

    /// mutator that allows to modify a [`ColumnRef`] to have a <table>.<column> format
    ///
    /// Ex: SELECT * FROM <table>.<column> as <alias>
    pub fn table(mut self, table: &'a str) -> Self {
        self.table = Some(table);
        self
    }

    /// mutator that allows to modify a [`ColumnRef`] to have a AS clause for
    /// specify a SQL alias
    ///
    /// Ex: SELECT * FROM <table>.<column> as <alias>
    pub fn alias(mut self, alias: &'a str) -> Self {
        self.alias = Some(alias);
        self
    }
}
