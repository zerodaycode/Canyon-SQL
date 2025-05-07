use crate::connection::contracts::DbConnection;
use crate::mapper::RowMapper;
use crate::query::querybuilder::r#impl::QueryBuilder;
use std::error::Error;

pub struct SelectQueryBuilder<'a, I: DbConnection + ?Sized, R: RowMapper> {
    pub(crate) _inner: QueryBuilder<'a, I, R>,
}

impl<'a, I: DbConnection + ?Sized, R: RowMapper> SelectQueryBuilder<'a, I, R> {
    /// Generates a new public instance of the [`SelectQueryBuilder`]
    pub fn new(
        table_schema_data: &str,
        input: &'a I,
    ) -> Result<Self, Box<(dyn Error + Send + Sync + 'a)>> {
        Ok(Self {
            _inner: QueryBuilder::new(format!("SELECT * FROM {table_schema_data}"), input)?,
        })
    }

    /// Launches the generated query to the database pointed by the selected datasource
    #[inline]
    pub async fn query(self) -> Result<Vec<R>, Box<(dyn Error + Send + Sync + 'a)>>
    where
        Vec<R>: FromIterator<<R as RowMapper>::Output>,
    {
        self._inner.query().await
    }
}
