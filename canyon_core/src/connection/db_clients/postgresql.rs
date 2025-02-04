use crate::{query_parameters::QueryParameter, rows::CanyonRows};
use std::error::Error;
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
    fn query<'a>(
        &self,
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
    ) -> impl Future<Output=Result<CanyonRows, Box<(dyn Error + Sync + Send)>>> + Send
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
    use super::*;

    #[inline(always)]
    pub(crate) async fn query<'a>(
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
}
