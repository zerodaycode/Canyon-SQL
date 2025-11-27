use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use crate::query::querybuilder::syntax::tokens::{SqlToken, Symbol, ToSqlTokens};

#[derive(Clone, Default, Debug)]
pub struct TableMetadata<'a> {
    pub schema: Option<Cow<'a, str>>,
    pub name: Cow<'a, str>,
} // TODO: we can have those fields as Cow<'_> for max performance

impl<'a> ToSqlTokens<'a> for TableMetadata<'a> {
    fn to_tokens(&self, out: &mut Vec<SqlToken<'a>>) {
        match &self.schema {
            Some(s) => {
                out.push(SqlToken::Ident(s.clone()));
                out.push(SqlToken::Symbol(Symbol::Dot));
            }
            None => {}
        };
        out.push(SqlToken::Ident(self.name.clone()));
    }
}
impl<'a> From<&'a str> for TableMetadata<'a> {
    /// Creates a new [`TableMetadata<'a>`] from a string slice.
    ///
    /// If the slice contains a dot, we assume that is a schema.table_name format, otherwise,
    /// we assume that the client is just creating a [`Self`] from the passed in string
    fn from(value: &'a str) -> Self {
        if let Some((schema, table)) = value.split_once('.') {
            TableMetadata {
                schema: Some(Cow::from(schema)),
                name: Cow::from(table),
            }
        } else {
            TableMetadata {
                schema: None,
                name: Cow::from(value),
            }
        }
    }
}


impl<'a> TableMetadata<'a> {
    pub fn new(schema: &'a str, name: &'a str) -> Self {
        Self { schema: Some(Cow::from(schema)), name: Cow::from(name) }
    }
    pub fn schema(&mut self, schema: String) { self.schema = Some(Cow::from(schema)); }
    pub fn table_name(&mut self, table_name: String) { self.name = Cow::from(table_name) }

    /// Returns an already formatted version of the schema and table of a target database table
    /// ready to be used in a SQL statement.
    ///
    /// This method allocates a new string, so it returns an owned one to the callee.
    /// Just take it in consideration if someday someone uses it outside the macro generation
    /// and there's some heavy callee procedure
    pub fn sql(&self) -> String {
        match &self.schema {
            Some(schema_name) => {format!("{}.{}", schema_name, self.name)}
            None => self.name.to_string()
        }
    }
}

impl<'a> Display for TableMetadata<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.schema {
            Some(schema_name) => {write!(f, "{}.{}", schema_name, self.name)}
            None => {write!(f, "{}", self.name)}
        }
    }
}

impl<'a> AsRef<str> for TableMetadata<'a> {
    fn as_ref(&self) -> &str {
        self.schema.as_ref().unwrap()
    }
}
