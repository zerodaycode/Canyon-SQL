use crate::connection::database_type::DatabaseType;
use crate::query::query::Query;
use crate::query::querybuilder::r#impl::QueryBuilder;
use std::error::Error;

pub struct SelectQueryBuilder<'a> {
    pub(crate) _inner: QueryBuilder<'a>,
}

impl<'a> SelectQueryBuilder<'a> {
    /// Generates a new public instance of the [`SelectQueryBuilder`]
    pub fn new(
        table_schema_data: &str,
        database_type: DatabaseType,
    ) -> Result<Self, Box<(dyn Error + Send + Sync + 'a)>> {
        Ok(Self {
            _inner: QueryBuilder::new(format!("SELECT * FROM {table_schema_data}"), database_type)?,
        })
    }

    pub fn build(self) -> Result<Query<'a>, Box<dyn Error + Send + Sync>> {
        self._inner.build()
    }
}
