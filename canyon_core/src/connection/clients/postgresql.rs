use crate::connection::contracts::DbConnection;
use crate::connection::database_type::DatabaseType;
use crate::connection::datasources::{Auth, DatasourceConfig, PostgresAuth};
use crate::error::{CanyonResult, ConfigurationError, ConnectionError, QueryError};
use crate::mapper::RowMapper;
use crate::rows::FromSqlOwnedValue;
use crate::{query::parameters::QueryParameter, rows::CanyonRows};
use bb8::{Pool, PooledConnection};
use bb8_postgres::PostgresConnectionManager;
use std::sync::Arc;
use tokio_postgres::types::ToSql;
use tokio_postgres::{Config, NoTls};

type PgManager = PostgresConnectionManager<NoTls>;
type PostgresConnectionPool = Arc<bb8::Pool<PgManager>>;

/// A connector with a `PostgreSQL` database
pub struct PostgresConnector(PostgresConnectionPool);
impl PostgresConnector {
    pub async fn new(datasource: &DatasourceConfig) -> CanyonResult<Self> {
        Ok(Self(create_postgres_connector(datasource).await?))
    }

    pub async fn get_pooled(&self) -> CanyonResult<PooledConnection<'_, PgManager>> {
        self.0
            .get()
            .await
            .map_err(|source| ConnectionError::postgres_pool(source).into())
    }
}

impl DbConnection for PostgresConnector {
    async fn query_rows(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> CanyonResult<CanyonRows> {
        let r = self
            .get_pooled()
            .await?
            .query(stmt, &get_psql_params(params))
            .await
            .map_err(QueryError::postgres)?;
        Ok(CanyonRows::Postgres(r))
    }

    async fn query<S, R>(&self, stmt: S, params: &[&dyn QueryParameter]) -> CanyonResult<Vec<R>>
    where
        S: AsRef<str> + Send,
        R: RowMapper,
        Vec<R>: FromIterator<<R as RowMapper>::Output>,
    {
        self.get_pooled()
            .await?
            .query(stmt.as_ref(), &get_psql_params(params))
            .await
            .map_err(QueryError::postgres)?
            .iter()
            .map(R::deserialize_postgresql)
            .collect()
    }

    async fn query_one<R>(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> CanyonResult<Option<R::Output>>
    where
        R: RowMapper,
    {
        let result = self
            .get_pooled()
            .await?
            .query_opt(stmt, &get_psql_params(params))
            .await
            .map_err(QueryError::postgres)?;

        result.as_ref().map(R::deserialize_postgresql).transpose()
    }

    async fn query_one_for<T: FromSqlOwnedValue>(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> CanyonResult<T> {
        let r = self
            .get_pooled()
            .await?
            .query_opt(stmt, &get_psql_params(params))
            .await
            .map_err(QueryError::postgres)?
            .ok_or(QueryError::NoRows)?;
        r.try_get::<usize, T>(0)
            .map_err(|source| QueryError::postgres(source).into())
    }

    async fn execute(&self, stmt: &str, params: &[&dyn QueryParameter]) -> CanyonResult<u64> {
        self.get_pooled()
            .await?
            .execute(stmt, &get_psql_params(params))
            .await
            .map_err(|source| QueryError::postgres(source).into())
    }

    fn get_database_type(&self) -> CanyonResult<DatabaseType> {
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
) -> CanyonResult<Arc<Pool<PgManager>>> {
    let (user, password) = __impl::extract_postgres_auth(&datasource.auth)?;
    let config = __impl::set_tokio_postgres_configs(datasource, user, password);
    let conn_pool = __impl::create_postgres_connection_pool(config).await?;

    Ok(PostgresConnectionPool::from(conn_pool))
}

mod __impl {
    use super::*;

    pub(crate) fn set_tokio_postgres_configs(
        datasource_config: &DatasourceConfig,
        user: &str,
        password: &str,
    ) -> Config {
        let mut config = tokio_postgres::Config::new();
        config.host(&datasource_config.properties.host);
        config.port(datasource_config.get_port_or_default_by_db());
        config.dbname(&datasource_config.properties.db_name);
        config.user(user);
        config.password(password);

        // Optimize connection settings for better performance
        config.connect_timeout(std::time::Duration::from_secs(5));
        config.keepalives_idle(std::time::Duration::from_secs(30));
        config.keepalives_interval(std::time::Duration::from_secs(10));
        config.keepalives_retries(3);

        config
    }

    pub(crate) fn extract_postgres_auth(auth: &Auth) -> CanyonResult<(&str, &str)> {
        match auth {
            Auth::Postgres(pg_auth) => match pg_auth {
                PostgresAuth::Basic { username, password } => Ok((username, password)),
            },
            #[cfg(any(feature = "mssql", feature = "mysql"))]
            _ => Err(ConfigurationError::InvalidAuthentication {
                backend: DatabaseType::PostgreSql,
            }
            .into()),
        }
    }

    pub(crate) async fn create_postgres_connection_pool(
        config: Config,
    ) -> CanyonResult<Pool<PgManager>> {
        let manager = PgManager::new(config, NoTls);
        let pool = bb8::Pool::builder()
            .max_size(10u32)
            .build(manager)
            .await
            .map_err(ConnectionError::postgres)?;
        Ok(pool)
    }
}

#[cfg(test)]
mod tests {
    use super::__impl;
    use crate::connection::datasources::{
        Auth, DatasourceConfig, DatasourceProperties, PostgresAuth,
    };

    #[test]
    fn test_extract_postgres_auth_basic() {
        let auth = Auth::Postgres(PostgresAuth::Basic {
            username: "pguser".into(),
            password: "pgpass".into(),
        });

        let (user, pass) = __impl::extract_postgres_auth(&auth).unwrap();
        assert_eq!(user, "pguser");
        assert_eq!(pass, "pgpass");
    }

    #[test]
    fn test_set_tokio_postgres_configs_basic() {
        let datasource = DatasourceConfig {
            name: "pg_test".into(),
            properties: DatasourceProperties {
                host: "localhost".into(),
                db_name: "pg_db".into(),
                port: Some(5433),
                migrations: None,
                #[cfg(feature = "mssql")]
                mssql_tls: Default::default(),
            },
            auth: Auth::Postgres(PostgresAuth::Basic {
                username: "pguser".into(),
                password: "pgpass".into(),
            }),
        };

        let config = __impl::set_tokio_postgres_configs(&datasource, "pguser", "pgpass");

        assert_eq!(
            config.get_hosts(),
            vec![tokio_postgres::config::Host::Tcp("localhost".into())]
        );
        assert_eq!(config.get_dbname(), Some("pg_db"));
        assert_eq!(config.get_user(), Some("pguser"));
        assert_eq!(*config.get_ports().first().unwrap(), 5433);

        // sanity check for configured timeouts and keepalives
        assert_eq!(
            config.get_connect_timeout(),
            Some(std::time::Duration::from_secs(5)).as_ref()
        );
        assert_eq!(
            config.get_keepalives_idle(),
            std::time::Duration::from_secs(30)
        );
        assert_eq!(
            config.get_keepalives_interval(),
            Some(std::time::Duration::from_secs(10))
        );
        assert_eq!(config.get_keepalives_retries(), Some(3));
    }

    #[test]
    fn test_set_tokio_postgres_configs_default_port() {
        let datasource = DatasourceConfig {
            name: "pg_test_default".into(),
            properties: DatasourceProperties {
                host: "127.0.0.1".into(),
                db_name: "default_db".into(),
                port: None,
                migrations: None,
                #[cfg(feature = "mssql")]
                mssql_tls: Default::default(),
            },
            auth: Auth::Postgres(PostgresAuth::Basic {
                username: "user".into(),
                password: "pass".into(),
            }),
        };

        let config = __impl::set_tokio_postgres_configs(&datasource, "user", "pass");
        assert_eq!(*config.get_ports().first().unwrap(), 5432); // default Postgres port
        assert_eq!(config.get_dbname(), Some("default_db"));
        assert_eq!(config.get_user(), Some("user"));
    }
}
