use crate::connection::contracts::DbConnection;
use crate::connection::database_type::DatabaseType;
use crate::mapper::RowMapper;
use crate::query::parameters::QueryParameter;
use crate::rows::{CanyonRows, FromSqlOwnedValue};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(feature = "mssql")]
pub mod mssql;
#[cfg(feature = "mysql")]
pub mod mysql;
#[cfg(feature = "postgres")]
pub mod postgresql;

#[macro_use]
pub mod str;
pub mod database_connection;

// Apply the macro to implement DbConnection for &str and str
impl_db_connection!(str);
impl_db_connection!(&str);

impl<T> DbConnection for Arc<Mutex<T>>
where
    T: DbConnection + Send,
    Self: Clone,
{
    async fn query_rows(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<CanyonRows, Box<dyn Error + Send + Sync>> {
        self.lock().await.query_rows(stmt, params).await
    }

    async fn query<S, R>(
        &self,
        stmt: S,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<Vec<R>, Box<dyn Error + Send + Sync>>
    where
        S: AsRef<str> + Send,
        R: RowMapper,
        Vec<R>: FromIterator<<R as RowMapper>::Output>,
    {
        self.lock().await.query(stmt, params).await
    }

    async fn query_one<R>(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<Option<R::Output>, Box<dyn Error + Send + Sync>>
    where
        R: RowMapper,
    {
        self.lock().await.query_one::<R>(stmt, params).await
    }

    async fn query_one_for<F: FromSqlOwnedValue<F>>(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<F, Box<dyn Error + Send + Sync>> {
        self.lock().await.query_one_for::<F>(stmt, params).await
    }

    async fn execute(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<u64, Box<dyn Error + Send + Sync>> {
        self.lock().await.execute(stmt, params).await
    }

    fn get_database_type(&self) -> Result<DatabaseType, Box<dyn Error + Send + Sync>> {
        todo!()
    }
}
