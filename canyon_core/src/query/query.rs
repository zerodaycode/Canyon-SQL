use crate::canyon::Canyon;
use crate::connection::contracts::DbConnection;
use crate::mapper::RowMapper;
use crate::query::parameters::QueryParameter;
use crate::rows::FromSqlOwnedValue;
use crate::transaction::Transaction;
use std::error::Error;
use std::fmt::Debug;

// TODO: query should implement ToStatement (as the drivers underneath Canyon) or similar
// to be usable directly in the input of Transaction and DbConnenction
/// Holds a sql sentence details
#[derive(Debug)]
pub struct Query<'a> {
    sql: String,
    params: Vec<&'a dyn QueryParameter>,
}

impl AsRef<str> for Query<'_> {
    fn as_ref(&self) -> &str {
        self.sql.as_str()
    }
}

unsafe impl Send for Query<'_> {}
unsafe impl Sync for Query<'_> {}

impl<'a> Query<'a> {
    /// Constructs a new [`Self`] but receiving the number of expected query parameters, allowing
    /// to pre-allocate the underlying linear collection that holds the arguments to the exact capacity,
    /// potentially saving re-allocations when the query is created
    pub fn new(sql: String, params: Vec<&'a dyn QueryParameter>) -> Query<'a> {
        Self { sql, params }
    }

    /// Returns the SQL sentence of the query
    pub const fn sql(&self) -> &str {
        self.sql.as_str()
    }

    pub const fn params(&self) -> &[&'a dyn QueryParameter] {
        self.params.as_slice()
    }

    /// Launches the generated query against the database assuming the default
    /// [`DbConnection`]
    pub async fn launch_default<T: Transaction + RowMapper>(
        self,
    ) -> Result<Vec<T>, Box<dyn Error + Send + Sync + 'a>>
    where
        Vec<T>: FromIterator<<T as RowMapper>::Output>,
    {
        let default_conn = Canyon::instance()?.get_default_connection()?;
        <T as Transaction>::query(&self.sql, &self.params, default_conn).await
    }

    pub async fn launch_one_for_default<T: Transaction, F: FromSqlOwnedValue<F>>(
        self,
    ) -> Result<F, Box<dyn Error + Send + Sync>> {
        let default_conn = Canyon::instance()?.get_default_connection()?;
        <T as Transaction>::query_one_for(&self.sql, &self.params, default_conn).await
    }

    pub async fn launch_one_for_with<T: Transaction, F: FromSqlOwnedValue<F>, I: DbConnection>(
        self,
        input: I,
    ) -> Result<F, Box<dyn Error + Send + Sync>> {
        input.query_one_for(&self.sql, &self.params).await
    }

    /// Launches the generated query against the database with the selected [`DbConnection`]
    pub async fn launch_with<I: DbConnection + 'a, R: RowMapper>(
        self,
        input: I,
    ) -> Result<Vec<R>, Box<dyn Error + Send + Sync + 'a>>
    where
        Vec<R>: FromIterator<<R as RowMapper>::Output>,
    {
        input.query(&self.sql, &self.params).await
    }
}

impl<'a> Transaction for Query<'a> {}
