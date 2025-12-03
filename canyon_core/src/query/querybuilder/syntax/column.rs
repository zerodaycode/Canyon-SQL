use crate::query::querybuilder::syntax::symbol::Symbol::Dot;
use crate::query::querybuilder::syntax::tokens::{SqlToken, ToSqlTokens};

#[derive(Debug, Clone, Default)]
pub struct ColumnRef<'a> {
    pub table: Option<&'a str>,
    pub column: &'a str,
    pub alias: Option<&'a str>,
}

impl<'a> From<&'a str> for ColumnRef<'a> {
    fn from(value: &'a str) -> Self {
        Self::new(value)
    }
}

impl<'a> ToSqlTokens<'a> for ColumnRef<'a> {
    fn to_tokens(&self, out: &mut Vec<SqlToken<'a>>) {
        if let Some(table_ref) = self.table {
            out.push(SqlToken::new_ident(table_ref));
            out.push(SqlToken::Symbol(Dot))
        }

        out.push(SqlToken::new_ident(self.column));

        if let Some(alias) = self.alias {
            out.push(SqlToken::new_keyword("AS"));
            out.push(SqlToken::new_ident(alias));
        }
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
