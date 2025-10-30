use crate::connection::contracts::DbConnection;
use crate::connection::database_type::DatabaseType;
use crate::connection::datasources::{Auth, DatasourceConfig, DatasourceProperties, PostgresAuth};
use crate::connection::{PgManager, PostgresConnectionPool};
use crate::mapper::RowMapper;
use crate::rows::FromSqlOwnedValue;
use crate::{query::parameters::QueryParameter, rows::CanyonRows};
use bb8::{Pool, PooledConnection};
use std::error::Error;
use std::sync::Arc;
use tokio_postgres::types::ToSql;
use tokio_postgres::{Config, NoTls};

/// A connector with a `PostgreSQL` database
#[cfg(feature = "postgres")]
pub struct PostgresConnector(PostgresConnectionPool);

#[cfg(feature = "postgres")]
impl PostgresConnector {
    pub async fn new(datasource: &DatasourceConfig) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(Self(create_postgres_connector(datasource).await?))
    }

    pub async fn get_pooled(
        &self,
    ) -> Result<PooledConnection<'_, PgManager>, Box<dyn Error + Send + Sync>> {
        Ok(self.0.get().await?)
    }
}

impl DbConnection for PostgresConnector {
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

// Façade helper to create a new postgres connector
async fn create_postgres_connector(
    datasource: &DatasourceConfig,
) -> Result<Arc<Pool<PgManager>>, Box<dyn Error + Send + Sync>> {
    let (user, password) = __impl::extract_postgres_auth(&datasource.auth)?;
    let config = __impl::set_tokio_postgres_configs(&datasource.properties, user, password);
    let conn_pool = __impl::create_postgres_connection_pool(config).await?;

    Ok(PostgresConnectionPool::from(conn_pool))
}

mod __impl {
    use super::*;

    pub(crate) fn set_tokio_postgres_configs(
        datasource_properties: &DatasourceProperties,
        user: &str,
        password: &str,
    ) -> Config {
        let mut config = tokio_postgres::Config::new();
        config.host(&datasource_properties.host);
        config.port(datasource_properties.port.unwrap_or_default());
        config.dbname(&datasource_properties.db_name);
        config.user(user);
        config.password(password);

        // Optimize connection settings for better performance
        config.connect_timeout(std::time::Duration::from_secs(5));
        config.keepalives_idle(std::time::Duration::from_secs(30));
        config.keepalives_interval(std::time::Duration::from_secs(10));
        config.keepalives_retries(3);

        config
    }

    pub(crate) fn extract_postgres_auth(
        auth: &Auth,
    ) -> Result<(&str, &str), Box<dyn std::error::Error + Send + Sync>> {
        match auth {
            Auth::Postgres(pg_auth) => match pg_auth {
                PostgresAuth::Basic { username, password } => Ok((username, password)),
            },
            #[cfg(any(feature = "mssql", feature = "mysql"))]
            _ => Err("Invalid auth configuration for a Postgres datasource.".into()),
        }
    }
    pub(crate) async fn create_postgres_connection_pool(
        config: Config,
    ) -> Result<Pool<PgManager>, Box<dyn Error + Send + Sync>> {
        let manager = PgManager::new(config, NoTls);
        let pool = bb8::Pool::builder().max_size(10u32).build(manager).await?;
        Ok(pool)
    }
}
