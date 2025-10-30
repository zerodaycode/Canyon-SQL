use crate::connection::contracts::DbConnection;
use crate::connection::database_type::DatabaseType;
use crate::connection::{PgManager, PostgresConnectionPool};
use crate::mapper::RowMapper;
use crate::rows::FromSqlOwnedValue;
use crate::{query::parameters::QueryParameter, rows::CanyonRows};
use bb8::PooledConnection;
use std::error::Error;
use tokio_postgres::types::ToSql;

/// A connector with a `PostgreSQL` database
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

impl DbConnection for PostgresConnection {
    async fn query_rows(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<CanyonRows, Box<dyn Error + Send + Sync>> {
        let r = self
            .get_pooled()
            .await?
            .query(stmt, &get_psql_params(params))
            .await?;
        Ok(CanyonRows::Postgres(r))
    }

    async fn query<S, R>(
        &self,
        stmt: S,
        params: &[&dyn QueryParameter],
    ) -> Result<Vec<R>, Box<dyn Error + Send + Sync>>
    where
        S: AsRef<str> + Send,
        R: RowMapper,
        Vec<R>: FromIterator<<R as RowMapper>::Output>,
    {
        Ok(self
            .get_pooled()
            .await?
            .query(stmt.as_ref(), &get_psql_params(params))
            .await?
            .iter()
            .flat_map(|row| R::deserialize_postgresql(row))
            .collect())
    }

    async fn query_one<R>(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<Option<R::Output>, Box<dyn Error + Send + Sync>>
    where
        R: RowMapper,
    {
        let result = self
            .get_pooled()
            .await?
            .query_one(stmt, &get_psql_params(params))
            .await;

        match result {
            Ok(row) => Ok(Some(R::deserialize_postgresql(&row)?)),
            Err(e) => match e.to_string().contains("unexpected number of rows") {
                true => Ok(None),
                _ => Err(e)?,
            },
        }
    }

    async fn query_one_for<T: FromSqlOwnedValue<T>>(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<T, Box<dyn Error + Send + Sync>> {
        let r = self
            .get_pooled()
            .await?
            .query_one(stmt, &get_psql_params(params))
            .await?;
        r.try_get::<usize, T>(0).map_err(From::from)
    }

    async fn execute(
        &self,
        stmt: &str,
        params: &[&dyn QueryParameter],
    ) -> Result<u64, Box<dyn Error + Send + Sync>> {
        self.get_pooled()
            .await?
            .execute(stmt, &get_psql_params(params))
            .await
            .map_err(From::from)
    }

    fn get_database_type(&self) -> Result<DatabaseType, Box<dyn Error + Send + Sync>> {
        Ok(DatabaseType::PostgreSql)
    }
}

fn get_psql_params<'a>(params: &'a [&'a dyn QueryParameter]) -> Vec<&'a (dyn ToSql + Sync)> {
    params
        .iter()
        .map(|param| param.as_postgres_param())
        .collect::<Vec<_>>()
}
