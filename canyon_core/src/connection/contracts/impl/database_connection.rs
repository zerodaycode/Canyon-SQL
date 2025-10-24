use crate::{
    connection::{
        contracts::DbConnection, database_type::DatabaseType, db_connector::DatabaseConnection,
    },
    mapper::RowMapper,
    query::parameters::QueryParameter,
    rows::{CanyonRows, FromSqlOwnedValue},
};
use std::error::Error;

impl DbConnection for DatabaseConnection {
    async fn query_rows(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<CanyonRows, Box<dyn Error + Send + Sync>> {
        db_conn_query_rows_impl(self, stmt, params).await
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
        db_conn_query_impl(self, stmt, params).await
    }

    async fn query_one<R>(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<Option<R::Output>, Box<dyn Error + Send + Sync>>
    where
        R: RowMapper,
    {
        db_conn_query_one_impl::<R>(self, stmt, params).await
    }

    async fn query_one_for<T: FromSqlOwnedValue<T>>(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<T, Box<dyn Error + Send + Sync>> {
        db_conn_query_one_for_impl::<T>(self, stmt, params).await
    }

    async fn execute(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<u64, Box<dyn Error + Send + Sync>> {
        db_conn_execute_impl(self, stmt, params).await
    }

    fn get_database_type(&self) -> Result<DatabaseType, Box<dyn Error + Send + Sync>> {
        Ok(self.get_db_type())
    }
}

impl DbConnection for &DatabaseConnection {
    async fn query_rows(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<CanyonRows, Box<dyn Error + Send + Sync>> {
        db_conn_query_rows_impl(self, stmt, params).await
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
        db_conn_query_impl(self, stmt, params).await
    }

    async fn query_one<R>(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<Option<R::Output>, Box<dyn Error + Send + Sync>>
    where
        R: RowMapper,
    {
        db_conn_query_one_impl::<R>(self, stmt, params).await
    }

    async fn query_one_for<T: FromSqlOwnedValue<T>>(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<T, Box<dyn Error + Send + Sync>> {
        db_conn_query_one_for_impl::<T>(self, stmt, params).await
    }

    async fn execute(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<u64, Box<dyn Error + Send + Sync>> {
        db_conn_execute_impl(self, stmt, params).await
    }

    fn get_database_type(&self) -> Result<DatabaseType, Box<dyn Error + Send + Sync>> {
        Ok(self.get_db_type())
    }
}

impl DbConnection for &mut DatabaseConnection {
    async fn query_rows(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<CanyonRows, Box<dyn Error + Send + Sync>> {
        db_conn_query_rows_impl(self, stmt, params).await
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
        db_conn_query_impl(self, stmt, params).await
    }

    async fn query_one<R>(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<Option<R::Output>, Box<dyn Error + Send + Sync>>
    where
        R: RowMapper,
    {
        db_conn_query_one_impl::<R>(self, stmt, params).await
    }

    async fn query_one_for<T: FromSqlOwnedValue<T>>(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<T, Box<dyn Error + Send + Sync>> {
        db_conn_query_one_for_impl::<T>(self, stmt, params).await
    }

    async fn execute(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<u64, Box<dyn Error + Send + Sync>> {
        db_conn_execute_impl(self, stmt, params).await
    }

    fn get_database_type(&self) -> Result<DatabaseType, Box<dyn Error + Send + Sync>> {
        Ok(self.get_db_type())
    }
}

pub(crate) async fn db_conn_query_rows_impl<'a>(
    c: &DatabaseConnection,
    stmt: &str,
    params: &[&'a (dyn QueryParameter + 'a)],
) -> Result<CanyonRows, Box<dyn Error + Send + Sync>> {
    match c {
        #[cfg(feature = "postgres")]
        DatabaseConnection::Postgres(client) => client.query_rows(stmt, params).await,

        #[cfg(feature = "mssql")]
        DatabaseConnection::SqlServer(client) => client.query_rows(stmt, params).await,

        #[cfg(feature = "mysql")]
        DatabaseConnection::MySQL(client) => client.query_rows(stmt, params).await,
    }
}

pub(crate) async fn db_conn_query_one_impl<R>(
    c: &DatabaseConnection,
    stmt: &str,
    params: &[&'_ (dyn QueryParameter + '_)],
) -> Result<Option<R::Output>, Box<dyn Error + Send + Sync>>
where
    R: RowMapper,
{
    match c {
        #[cfg(feature = "postgres")]
        DatabaseConnection::Postgres(client) => client.query_one::<R>(stmt, params).await,

        #[cfg(feature = "mssql")]
        DatabaseConnection::SqlServer(client) => client.query_one::<R>(stmt, params).await,

        #[cfg(feature = "mysql")]
        DatabaseConnection::MySQL(client) => client.query_one::<R>(stmt, params).await,
    }
}

pub(crate) async fn db_conn_query_impl<S, R>(
    c: &DatabaseConnection,
    stmt: S,
    params: &[&'_ dyn QueryParameter],
) -> Result<Vec<R>, Box<dyn Error + Send + Sync>>
where
    S: AsRef<str> + Send,
    R: RowMapper,
    Vec<R>: FromIterator<<R as RowMapper>::Output>,
{
    match c {
        #[cfg(feature = "postgres")]
        DatabaseConnection::Postgres(client) => client.query(stmt, params).await,

        #[cfg(feature = "mssql")]
        DatabaseConnection::SqlServer(client) => client.query(stmt, params).await,

        #[cfg(feature = "mysql")]
        DatabaseConnection::MySQL(client) => client.query(stmt, params).await,
    }
}

pub(crate) async fn db_conn_query_one_for_impl<T>(
    c: &DatabaseConnection,
    stmt: &str,
    params: &[&'_ dyn QueryParameter],
) -> Result<T, Box<dyn Error + Send + Sync>>
where
    T: FromSqlOwnedValue<T>,
{
    match c {
        #[cfg(feature = "postgres")]
        DatabaseConnection::Postgres(client) => client.query_one_for(stmt, params).await,

        #[cfg(feature = "mssql")]
        DatabaseConnection::SqlServer(client) => client.query_one_for(stmt, params).await,

        #[cfg(feature = "mysql")]
        DatabaseConnection::MySQL(client) => client.query_one_for(stmt, params).await,
    }
}

pub(crate) async fn db_conn_execute_impl(
    c: &DatabaseConnection,
    stmt: &str,
    params: &[&'_ dyn QueryParameter],
) -> Result<u64, Box<dyn Error + Send + Sync>> {
    match c {
        #[cfg(feature = "postgres")]
        DatabaseConnection::Postgres(client) => client.execute(stmt, params).await,

        #[cfg(feature = "mssql")]
        DatabaseConnection::SqlServer(client) => client.execute(stmt, params).await,

        #[cfg(feature = "mysql")]
        DatabaseConnection::MySQL(client) => client.execute(stmt, params).await,
    }
}
