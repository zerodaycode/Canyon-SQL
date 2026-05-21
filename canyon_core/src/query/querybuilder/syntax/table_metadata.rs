use crate::query::bounds;
use crate::query::querybuilder::syntax::dialect::SqlDialect;
use crate::query::querybuilder::syntax::symbol::Symbol;
use crate::query::querybuilder::syntax::tokens::{SqlToken, SqlTokens, ToSqlTokens};
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use crate::query::querybuilder::syntax::emitter::types::helpers::push_quoted_ident;

#[derive(Clone, Default, Debug)]
pub struct TableMetadata<'a> {
    pub schema: Option<&'a str>,
    pub name: &'a str,
}

impl<'a, T> From<T> for TableMetadata<'a>
where
    T: bounds::TableMetadata<'a>,
{
    fn from(value: T) -> Self {
        Self::from(value.as_str()) // this covers the need of producing <table>.<column>
    }
}

impl<'a, D: SqlDialect> ToSqlTokens<'a, D> for TableMetadata<'a> {
    fn to_tokens(&self) -> impl IntoIterator<Item = SqlToken<'a>> + 'a {
        let mut out = SqlTokens::with_capacity(3);
        if let Some(schema) = &self.schema {
            push_quoted_ident::<D>(schema, &mut out);
            out.symbol(Symbol::Dot);
        };
        push_quoted_ident::<D>(self.name, &mut out);
        out
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
                schema: Some(schema),
                name: table,
            }
        } else {
            TableMetadata {
                schema: None,
                name: value,
            }
        }
    }
}

impl<'a> TableMetadata<'a> {
    pub fn new(table_name: &'a str) -> Self {
        Self::from(table_name)
    }
    pub fn schema(&mut self, schema: &'a str) {
        self.schema = Some(schema);
    }
    pub fn table_name(&mut self, table_name: &'a str) {
        self.name = table_name;
    }

    /// Returns an already formatted version of the schema and table of a target database table
    /// ready to be used in a SQL statement.
    ///
    /// This method allocates a new string, so it returns an owned one to the callee.
    /// Just take it in consideration if someday someone uses it outside the macro generation
    /// and there's some heavy callee procedure
    pub fn sql(&self) -> String {
        match &self.schema {
            Some(schema_name) => {
                format!("{}.{}", schema_name, self.name)
            }
            None => self.name.to_string(),
        }
    }
}

impl<'a> Display for TableMetadata<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.schema {
            Some(schema_name) => {
                write!(f, "{}.{}", schema_name, self.name)
            }
            None => {
                write!(f, "{}", self.name)
            }
        }
    }
}
