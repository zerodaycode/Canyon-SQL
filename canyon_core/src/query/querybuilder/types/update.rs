use crate::connection::database_type::DatabaseType;
use crate::query::query::Query;
use crate::query::querybuilder::r#impl::QueryBuilder;
use std::error::Error;

/// Contains the specific database operations of the *UPDATE* SQL statements.
pub struct UpdateQueryBuilder<'a> {
    pub(crate) _inner: QueryBuilder<'a>,
}

impl<'a> UpdateQueryBuilder<'a> {
    /// Generates a new public instance of the [`UpdateQueryBuilder`]
    pub fn new(
        table_schema_data: &str,
        database_type: DatabaseType,
    ) -> Result<Self, Box<(dyn Error + Send + Sync + 'a)>> {
        Ok(Self {
            _inner: QueryBuilder::new(format!("UPDATE {table_schema_data}"), database_type)?,
        })
    }

    pub fn build(self) -> Result<Query<'a>, Box<dyn Error + Send + Sync>> {
        self._inner.build()
    }
}
