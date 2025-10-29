use crate::connection::{PgManager, PostgresConnectionPool};
use crate::mapper::RowMapper;
use crate::{query::parameters::QueryParameter, rows::CanyonRows};
use bb8::PooledConnection;
use std::error::Error;

/// A connection with a `PostgreSQL` database
#[cfg(feature = "postgres")]
pub struct PostgresConnection(PostgresConnectionPool);

#[cfg(feature = "postgres")]
impl PostgresConnection {
    pub fn new(pool: PostgresConnectionPool) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(Self(pool))
    }
    pub async fn get_pooled(
        &self,
    ) -> Result<PooledConnection<'_, PgManager>, Box<dyn Error + Send + Sync>> {
        Ok(self.0.get().await?)
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
        params: &[&'_ dyn QueryParameter],
        conn: &PostgresConnection,
    ) -> Result<Vec<R>, Box<dyn Error + Send + Sync>>
    where
        S: AsRef<str> + Send,
        R: RowMapper,
        Vec<R>: FromIterator<<R as RowMapper>::Output>,
    {
        Ok(conn
            .get_pooled()
            .await?
            .query(stmt.as_ref(), &get_psql_params(params))
            .await?
            .iter()
            .flat_map(|row| R::deserialize_postgresql(row))
            .collect())
    }

    #[inline(always)]
    pub(crate) async fn query_rows(
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
        conn: &PostgresConnection,
    ) -> Result<CanyonRows, Box<dyn Error + Send + Sync>> {
        let m_params: Vec<_> = params
            .iter()
            .map(|param| param.as_postgres_param())
            .collect();
        let r = conn
            .get_pooled()
            .await?
            .query(stmt, m_params.as_slice())
            .await?;
        Ok(CanyonRows::Postgres(r))
    }

    /// *NOTE*: implementation details of `query_one` when handling errors are
    /// discussed [here](https://github.com/sfackler/rust-postgres/issues/790#issuecomment-2095729043)
    #[inline(always)]
    pub(crate) async fn query_one<R>(
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
        conn: &PostgresConnection,
    ) -> Result<Option<R::Output>, Box<dyn Error + Send + Sync>>
    where
        R: RowMapper,
    {
        let m_params: Vec<_> = params
            .iter()
            .map(|param| param.as_postgres_param())
            .collect();
        let result = conn
            .get_pooled()
            .await?
            .query_one(stmt, m_params.as_slice())
            .await;

        match result {
            Ok(row) => Ok(Some(R::deserialize_postgresql(&row)?)),
            Err(e) => match e.to_string().contains("unexpected number of rows") {
                true => Ok(None),
                _ => Err(e)?,
            },
        }
    }

    #[inline(always)]
    pub(crate) async fn query_one_for<T: FromSqlOwnedValue<T>>(
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
        conn: &PostgresConnection,
    ) -> Result<T, Box<dyn Error + Send + Sync>> {
        let m_params: Vec<_> = params
            .iter()
            .map(|param| param.as_postgres_param())
            .collect();
        let r = conn
            .get_pooled()
            .await?
            .query_one(stmt, m_params.as_slice())
            .await?;
        r.try_get::<usize, T>(0).map_err(From::from)
    }

    #[inline(always)]
    pub(crate) async fn execute<'a, S>(
        stmt: S,
        params: &'a [&'a (dyn QueryParameter + 'a)],
        conn: &PostgresConnection,
    ) -> Result<u64, Box<dyn Error + Send + Sync>>
    where
        S: AsRef<str> + Send,
    {
        conn.get_pooled()
            .await?
            .execute(stmt.as_ref(), &get_psql_params(params))
            .await
            .map_err(From::from)
    }

    fn get_psql_params<'a>(params: &'a [&'a dyn QueryParameter]) -> Vec<&'a (dyn ToSql + Sync)> {
        params
            .iter()
            .map(|param| param.as_postgres_param())
            .collect::<Vec<_>>()
    }
}
