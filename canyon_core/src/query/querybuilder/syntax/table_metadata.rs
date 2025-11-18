use std::fmt::{Display, Formatter};
use crate::query::querybuilder::syntax::tokens::{SqlToken, ToSqlTokens};

#[derive(Clone, Default, Debug)]
pub struct TableMetadata {
    pub schema: Option<String>,
    pub name: String,
} // TODO: we can have those fields as Cow<'_> for max performance

impl<'a> ToSqlTokens<'a> for TableMetadata {
    fn to_tokens(&self) -> SqlToken<'a> {
        match &self.schema {
            Some(s) => {
                out.push(SqlToken::Ident(s));
                out.push(SqlToken::Symbol('.'));
                out.push(SqlToken::Ident(&self.name));
            }
            None => {
                out.push(SqlToken::Ident(&self.name));
            }
        }
    }
}
impl From<&str> for TableMetadata {
    /// Creates a new [`TableMetadata`] from a string slice.
    ///
    /// If the slice contains a dot, we assume that is a schema.table_name format, otherwise,
    /// we assume that the client is just creating a [`Self`] from the passed in string
    fn from(value: &str) -> Self {
        if let Some((schema, table)) = value.split_once('.') {
            TableMetadata {
                schema: Some(schema.to_string()),
                name: table.to_string(),
            }
        } else {
            TableMetadata {
                schema: None,
                name: value.to_string(),
            }
        }
    }
}


impl<'a> TableMetadata {
    pub fn new(schema: &'a str, name: &'a str) -> Self {
        Self { schema: Some(schema.to_string()), name: name.to_string() }
    }
    pub fn schema(&mut self, schema: String) { self.schema = Some(schema); }
    pub fn table_name(&mut self, table_name: String) { self.name = table_name }

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

impl Display for TableMetadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.schema {
            Some(schema_name) => {write!(f, "{}.{}", schema_name, self.name)}
            None => {write!(f, "{}", self.name)}
        }
    }
}

impl AsRef<str> for TableMetadata {
    fn as_ref(&self) -> &str {
        self.schema.as_ref().unwrap()
    }
}
