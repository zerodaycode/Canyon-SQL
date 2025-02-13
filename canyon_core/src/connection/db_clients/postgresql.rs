use crate::connection::database_type::DatabaseType;
use crate::connection::db_connector::DbConnection;
use crate::mapper::RowMapper;
use crate::rows::FromSqlOwnedValue;
use crate::{query_parameters::QueryParameter, rows::CanyonRows};
use std::error::Error;
use std::fmt::Display;
use std::future::Future;
#[cfg(feature = "postgres")]
use tokio_postgres::Client;

/// A connection with a `PostgreSQL` database
#[cfg(feature = "postgres")]
pub struct PostgreSqlConnection {
    pub client: Client,
    // pub connection: Connection<Socket, NoTlsStream>, // TODO Hold it, or not to hold it... that's the question!
}

impl DbConnection for PostgreSqlConnection {
    fn query_rows<'a>(
        &self,
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
    ) -> impl Future<Output = Result<CanyonRows, Box<(dyn Error + Sync + Send)>>> + Send {
        postgres_query_launcher::query_rows(stmt, params, self)
    }

    fn query<'a, S, R>(
        &self,
        stmt: S,
        params: &[&'a (dyn QueryParameter<'a>)],
    ) -> impl Future<Output = Result<Vec<R>, Box<(dyn Error + Sync + Send)>>> + Send
    where
        S: AsRef<str> + Display + Send,
        R: RowMapper<Output = R>
    {
        postgres_query_launcher::query(stmt, params, self)
    }

    fn query_one<'a, R>(
        &self,
        stmt: &str,
        params: &[&'a (dyn QueryParameter<'a>)],
    ) -> impl Future<Output = Result<Option<R>, Box<(dyn Error + Send + Sync)>>> + Send
        where R: RowMapper<Output = R>
    {
        postgres_query_launcher::query_one(stmt, params, self)
    }

    fn query_one_for<'a, T: FromSqlOwnedValue<T>>(
        &self,
        stmt: &str,
        params: &[&'a (dyn QueryParameter<'a>)],
    ) -> impl Future<Output = Result<T, Box<(dyn Error + Send + Sync)>>> + Send {
        postgres_query_launcher::query_one_for(stmt, params, self)
    }

    fn execute<'a>(
        &self,
        stmt: &str,
        params: &[&'a (dyn QueryParameter<'a>)],
    ) -> impl Future<Output = Result<u64, Box<(dyn Error + Send + Sync)>>> + Send {
        postgres_query_launcher::execute(stmt, params, self)
    }

    fn get_database_type(&self) -> Result<DatabaseType, Box<(dyn Error + Sync + Send)>> {
        Ok(DatabaseType::PostgreSql)
    }
}

#[cfg(feature = "postgres")]
pub(crate) mod postgres_query_launcher {

    use super::*;
    use crate::rows::FromSqlOwnedValue;
    use tokio_postgres::types::ToSql;

    #[inline(always)]
    pub(crate) async fn query<S, R>(
        stmt: S,
        params: &[&'_ (dyn QueryParameter<'_>)],
        conn: &PostgreSqlConnection,
    ) -> Result<Vec<R>, Box<(dyn Error + Sync + Send)>>
    where
        S: AsRef<str> + Display + Send,
        R: RowMapper<Output = R>
    {
        Ok(conn
            .client
            .query(stmt.as_ref(), &get_psql_params(params))
            .await?
            .iter()
            .map(|row| R::deserialize_postgresql(row))
            .collect())
    }

    #[inline(always)]
    pub(crate) async fn query_rows<'a>(
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
        conn: &PostgreSqlConnection,
    ) -> Result<CanyonRows, Box<(dyn Error + Sync + Send)>> {
        let m_params: Vec<_> = params
            .iter()
            .map(|param| param.as_postgres_param())
            .collect();
        let r = conn.client.query(stmt, m_params.as_slice()).await?;
        Ok(CanyonRows::Postgres(r))
    }

    /// *NOTE*: implementation details of `query_one` when handling errors are
    /// discussed [here](https://github.com/sfackler/rust-postgres/issues/790#issuecomment-2095729043)
    #[inline(always)]
    pub(crate) async fn query_one<'a, R>(
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
        conn: &PostgreSqlConnection,
    ) -> Result<Option<R>, Box<(dyn Error + Sync + Send)>> 
        where R: RowMapper<Output = R>
    {
        let m_params: Vec<_> = params
            .iter()
            .map(|param| param.as_postgres_param())
            .collect();
        let result = conn.client.query_one(stmt, m_params.as_slice()).await;
        
        match result {
            Ok(row) => { Ok(Some(R::deserialize_postgresql(&row))) },
            Err(e) => match e.to_string().contains("unexpected number of rows") {
                true => { Ok(None) },
                _ => Err(e)?,
            }
        }
    }

    #[inline(always)]
    pub(crate) async fn query_one_for<'a, T: FromSqlOwnedValue<T>>(
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
        conn: &PostgreSqlConnection,
    ) -> Result<T, Box<(dyn Error + Sync + Send)>> {
        let m_params: Vec<_> = params
            .iter()
            .map(|param| param.as_postgres_param())
            .collect();
        let r = conn.client.query_one(stmt, m_params.as_slice()).await?;
        r.try_get::<usize, T>(0).map_err(From::from)
    }

    #[inline(always)]
    pub(crate) async fn execute<S>(
        stmt: S,
        params: &[&'_ (dyn QueryParameter<'_>)],
        conn: &PostgreSqlConnection,
    ) -> Result<u64, Box<(dyn Error + Sync + Send)>>
    where
        S: AsRef<str> + Display + Send,
    {
        conn.client
            .execute(stmt.as_ref(), &get_psql_params(params))
            .await
            .map_err(From::from)
    }

    fn get_psql_params<'a>(params: &[&'a (dyn QueryParameter<'_>)]) -> Vec<&'a (dyn ToSql + Sync)> {
        params
            .as_ref()
            .iter()
            .map(|param| param.as_postgres_param())
            .collect::<Vec<_>>()
    }
}
