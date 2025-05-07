use crate::connection::contracts::DbConnection;
use crate::mapper::RowMapper;
use crate::query::querybuilder::r#impl::QueryBuilder;
use std::error::Error;

/// Contains the specific database operations of the *UPDATE* SQL statements.
///
/// * `set` - To construct a new `SET` clause to determine the columns to
///   update with the provided values
pub struct UpdateQueryBuilder<'a, I: DbConnection + ?Sized, R: RowMapper> {
    pub(crate) _inner: QueryBuilder<'a, I, R>,
}

impl<'a, I: DbConnection + ?Sized, R: RowMapper> UpdateQueryBuilder<'a, I, R> {
    /// Generates a new public instance of the [`UpdateQueryBuilder`]
    pub fn new(
        table_schema_data: &str,
        input: &'a I,
    ) -> Result<Self, Box<(dyn Error + Send + Sync + 'a)>> {
        Ok(Self {
            _inner: QueryBuilder::new(format!("UPDATE {table_schema_data}"), input)?,
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
