use crate::{query_parameters::QueryParameter, rows::CanyonRows};
use std::error::Error;
use std::fmt::Display;
use std::future::Future;
use crate::connection::database_type::DatabaseType;
use crate::connection::db_connector::DbConnection;
#[cfg(feature = "postgres")]
use tokio_postgres::Client;
use crate::mapper::RowMapper;

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
    ) -> impl Future<Output=Result<CanyonRows, Box<(dyn Error + Sync + Send)>>> + Send
    {
        postgres_query_launcher::query_rows(stmt, params, self)
    }

    fn query<'a, S, R: RowMapper<R>>(
        &self,
        stmt: S,
        params: &[&'a (dyn QueryParameter<'_>)],
    ) -> impl Future<Output = Result<Vec<R>, Box<(dyn Error + Sync + Send + 'a)>>> + Send
    where
        S: AsRef<str> + Display + Send
    {
        postgres_query_launcher::query(stmt, params, self)
    }

    fn query_one<'a, R>(&self, stmt: &str, params: &[&'a (dyn QueryParameter<'a>)])
        -> impl Future<Output=Result<Option<R>, Box<(dyn Error + Send + Sync)>>> + Send
    where
        R: RowMapper<R>
    {
        postgres_query_launcher::query_one(stmt, params, self)
    }
    
    fn get_database_type(&self) -> Result<DatabaseType, Box<(dyn Error + Sync + Send)>> {
        Ok(DatabaseType::PostgreSql)
    }
}

#[cfg(feature = "postgres")]
pub(crate) mod postgres_query_launcher {
    
    
    use tokio_postgres::types::ToSql;
    use super::*;

    #[inline(always)]
    pub(crate) async fn query_rows<'a>(
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
        conn: &PostgreSqlConnection,
    ) -> Result<CanyonRows, Box<(dyn Error + Sync + Send)>> {
        let m_params: Vec<_> = params.iter().map(|param| param.as_postgres_param()).collect();
        let r = conn.client.query(stmt, m_params.as_slice()).await?;
        Ok(CanyonRows::Postgres(r))
    }

    #[inline(always)]
    pub(crate) async fn query_one<'a, T: RowMapper<T>>(
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
        conn: &PostgreSqlConnection,
    ) -> Result<Option<T>, Box<(dyn Error + Sync + Send)>> {
        let m_params: Vec<_> = params.iter().map(|param| param.as_postgres_param()).collect();
        let r = conn.client.query_one(stmt, m_params.as_slice()).await?;
        Ok(Some(T::deserialize_postgresql(&r)))
    }

    #[inline(always)]
    pub(crate) async fn query<'a, S, R: RowMapper<R>>(
        stmt: S,
        params: &[&'a (dyn QueryParameter<'_>)],
        conn: &PostgreSqlConnection,
    ) -> Result<Vec<R>, Box<(dyn Error + Sync + Send + 'a)>>
    where
        S: AsRef<str> + Display + Send
    {
        Ok(conn.client
            .query(stmt.as_ref(), &get_psql_params(params))
            .await?
            .iter()
            .map(|row| { R::deserialize_postgresql(row) })
            .collect()
        )
    }
    
    fn get_psql_params<'a>(params: &[&'a (dyn QueryParameter<'_>)],)
        -> Vec<&'a (dyn ToSql + Sync)>
    {
        params
            .as_ref()
            .iter()
            .map(|param| param.as_postgres_param())
            .collect::<Vec<_>>()
    }
}
