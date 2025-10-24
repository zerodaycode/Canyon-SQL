use crate::mapper::RowMapper;
use crate::{query::parameters::QueryParameter, rows::CanyonRows};
use std::error::Error;
#[cfg(feature = "postgres")]
use tokio_postgres::Client;

/// A connection with a `PostgreSQL` database
#[cfg(feature = "postgres")]
pub struct PostgreSqlConnection {
    pub client: Client,
    // pub connection: Connection<Socket, NoTlsStream>, // TODO Hold it, or not to hold it... that's the question!
}

#[cfg(feature = "postgres")]
pub(crate) mod postgres_query_launcher {

    use super::*;
    use crate::rows::FromSqlOwnedValue;
    use tokio_postgres::types::ToSql;

    #[inline(always)]
    pub(crate) async fn query<S, R>(
        stmt: S,
        params: &[&'_ dyn QueryParameter],
        conn: &PostgreSqlConnection,
    ) -> Result<Vec<R>, Box<dyn Error + Send + Sync>>
    where
        S: AsRef<str> + Send,
        R: RowMapper,
        Vec<R>: FromIterator<<R as RowMapper>::Output>,
    {
        Ok(conn
            .client
            .query(stmt.as_ref(), &get_psql_params(params))
            .await?
            .iter()
            .flat_map(|row| R::deserialize_postgresql(row))
            .collect())
    }

    #[inline(always)]
    pub(crate) async fn query_rows<'a>(
        stmt: &str,
        params: &[&'a dyn QueryParameter],
        conn: &PostgreSqlConnection,
    ) -> Result<CanyonRows, Box<dyn Error + Send + Sync>> {
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
        params: &[&'a dyn QueryParameter],
        conn: &PostgreSqlConnection,
    ) -> Result<Option<R::Output>, Box<dyn Error + Send + Sync>>
    where
        R: RowMapper,
    {
        let m_params: Vec<_> = params
            .iter()
            .map(|param| param.as_postgres_param())
            .collect();
        let result = conn.client.query_one(stmt, m_params.as_slice()).await;

        match result {
            Ok(row) => Ok(Some(R::deserialize_postgresql(&row)?)),
            Err(e) => match e.to_string().contains("unexpected number of rows") {
                true => Ok(None),
                _ => Err(e)?,
            },
        }
    }

    #[inline(always)]
    pub(crate) async fn query_one_for<'a, T: FromSqlOwnedValue<T>>(
        stmt: &str,
        params: &[&'a dyn QueryParameter],
        conn: &PostgreSqlConnection,
    ) -> Result<T, Box<dyn Error + Send + Sync>> {
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
        params: &[&'_ dyn QueryParameter],
        conn: &PostgreSqlConnection,
    ) -> Result<u64, Box<dyn Error + Send + Sync>>
    where
        S: AsRef<str> + Send,
    {
        conn.client
            .execute(stmt.as_ref(), &get_psql_params(params))
            .await
            .map_err(From::from)
    }

    fn get_psql_params<'a>(params: &[&'a dyn QueryParameter]) -> Vec<&'a (dyn ToSql + Sync)> {
        params
            .as_ref()
            .iter()
            .map(|param| param.as_postgres_param())
            .collect::<Vec<_>>()
    }
}
