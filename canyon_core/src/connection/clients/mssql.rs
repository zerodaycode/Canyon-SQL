use crate::connection::clients::mssql::sqlserver_query_launcher::execute_query;
use crate::connection::contracts::DbConnection;
use crate::connection::database_type::DatabaseType;
use crate::connection::datasources::DatasourceConfig;
#[cfg(any(feature = "postgres", feature = "mysql"))]
use crate::error::ConfigurationError;
use crate::error::{CanyonResult, ConnectionError, QueryError};
use crate::mapper::RowMapper;
use crate::query::parameters::QueryParameter;
use crate::rows::{CanyonRows, FromSqlOwnedValue};
use bb8::PooledConnection;
use bb8_tiberius::ConnectionManager as TiberiusConnectionManager;
use std::sync::Arc;
use tiberius::Query;

type SqlServerConnectionPool = Arc<bb8::Pool<TiberiusConnectionManager>>;

/// A connector for a `SqlServer` database
pub struct SqlServerConnector(SqlServerConnectionPool);

impl SqlServerConnector {
    pub async fn new(config: &DatasourceConfig) -> CanyonResult<Self> {
        Ok(Self(__impl::create_sqlserver_connector(config).await?))
    }
    pub async fn get_pooled(
        &self,
    ) -> CanyonResult<PooledConnection<'_, TiberiusConnectionManager>> {
        self.0
            .get()
            .await
            .map_err(|source| ConnectionError::sql_server_pool(source).into())
    }
}

impl DbConnection for SqlServerConnector {
    async fn query_rows(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> CanyonResult<CanyonRows> {
        let mut conn = self.get_pooled().await?;
        let result = execute_query(stmt, params, &mut conn)
            .await?
            .into_results()
            .await
            .map_err(QueryError::sql_server)?
            .into_iter()
            .flatten()
            .collect();

        Ok(CanyonRows::Tiberius(result))
    }

    async fn query<S, R>(&self, stmt: S, params: &[&'_ dyn QueryParameter]) -> CanyonResult<Vec<R>>
    where
        S: AsRef<str> + Send,
        R: RowMapper,
        Vec<R>: FromIterator<<R as RowMapper>::Output>,
    {
        let mut conn = self.get_pooled().await?;
        execute_query(stmt.as_ref(), params, &mut conn)
            .await?
            .into_results()
            .await
            .map_err(QueryError::sql_server)?
            .into_iter()
            .flatten()
            .map(|row| R::deserialize_sqlserver(&row))
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
        let mut conn = self.get_pooled().await?;

        let result = execute_query(stmt, params, &mut conn)
            .await?
            .into_row()
            .await
            .map_err(QueryError::sql_server)?;

        match result {
            Some(r) => Ok(Some(R::deserialize_sqlserver(&r)?)),
            None => Ok(None),
        }
    }

    async fn query_one_for<T: FromSqlOwnedValue>(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> CanyonResult<T> {
        let mut conn = self.get_pooled().await?;
        let row = crate::connection::clients::mssql::sqlserver_query_launcher::execute_query(
            stmt, params, &mut conn,
        )
        .await?
        .into_row()
        .await
        .map_err(QueryError::sql_server)?
        .ok_or(QueryError::NoRows)?;

        row.into_iter()
            .next()
            .ok_or(QueryError::NoColumns)
            .and_then(|value| {
                T::from_sql_owned(value)
                    .map_err(QueryError::sql_server)
                    .and_then(|value| value.ok_or(QueryError::UnexpectedNull))
            })
            .map_err(Into::into)
    }

    async fn execute(&self, stmt: &str, params: &[&'_ dyn QueryParameter]) -> CanyonResult<u64> {
        let mssql_query = crate::connection::clients::mssql::sqlserver_query_launcher::generate_mssql_query_client(stmt, params).await;
        let mut conn = self.get_pooled().await?;

        mssql_query
            .execute(&mut conn)
            .await
            .map(|r| r.total())
            .map_err(|source| QueryError::sql_server(source).into())
    }

    fn get_database_type(&self) -> CanyonResult<DatabaseType> {
        Ok(DatabaseType::SqlServer)
    }
}

pub(crate) mod sqlserver_query_launcher {
    use super::*;
    use tiberius::QueryStream;

    pub(crate) async fn execute_query<'a>(
        stmt: &str,
        params: &[&dyn QueryParameter],
        conn: &'a mut bb8::PooledConnection<'_, bb8_tiberius::ConnectionManager>,
    ) -> CanyonResult<QueryStream<'a>> {
        let mssql_query = generate_mssql_query_client(stmt, params).await;
        mssql_query
            .query(conn)
            .await
            .map_err(|source| QueryError::sql_server(source).into())
    }

    pub(crate) async fn generate_mssql_query_client<'a>(
        stmt: &str,
        params: &[&'a dyn QueryParameter],
    ) -> Query<'a> {
        let mut stmt = String::from(stmt);

        if stmt.contains("RETURNING") {
            // TODO: when the InsertQuerybuilder with a api on the builder for the returning clause
            let c = stmt.clone();
            let temp = c.split_once("RETURNING").unwrap();
            let temp2 = temp.0.split_once("VALUES").unwrap();

            stmt = format!(
                "{} OUTPUT inserted.{} VALUES {}",
                temp2.0.trim(),
                temp.1.trim(),
                temp2.1.trim()
            );
        }

        let stmt = stmt.replace('$', "@P"); // TODO: this should be solved by the querybuilder
        generate_query_and_bind_params(stmt, params)
    }

    // Query and parameters are generated in this procedure together to avoid lifetime errors
    fn generate_query_and_bind_params<'a>(
        stmt: String,
        params: &[&'a (dyn QueryParameter + 'a)],
    ) -> Query<'a> {
        let mut mssql_query = Query::new(stmt);
        params.iter().for_each(|param| {
            mssql_query.bind(*param);
        });
        mssql_query
    }
}

pub(crate) mod __impl {
    use super::*;
    use crate::connection::datasources::{Auth, SqlServerAuth, SqlServerTlsMode};
    use bb8::Pool;
    use std::sync::Arc;
    use tiberius::{Config, EncryptionLevel};

    pub(crate) async fn create_sqlserver_connector(
        datasource: &DatasourceConfig,
    ) -> CanyonResult<Arc<Pool<TiberiusConnectionManager>>> {
        let sqlserver_config = sqlserver_config_from_datasource(datasource)?;

        let manager = TiberiusConnectionManager::new(sqlserver_config);
        let pool = bb8::Pool::builder()
            .max_size(10u32)
            .build(manager)
            .await
            .map_err(ConnectionError::sql_server_manager)?;

        Ok(SqlServerConnectionPool::from(pool))
    }

    pub(crate) fn sqlserver_config_from_datasource(
        datasource: &DatasourceConfig,
    ) -> CanyonResult<Config> {
        let mut tiberius_config = tiberius::Config::new();

        tiberius_config.host(&datasource.properties.host);
        tiberius_config.port(datasource.get_port_or_default_by_db());
        tiberius_config.database(&datasource.properties.db_name);

        let auth_config = extract_mssql_auth(&datasource.auth)?;
        tiberius_config.authentication(auth_config);

        let (encryption, trust_server_certificate) =
            sqlserver_tls_options(datasource.properties.mssql_tls);
        tiberius_config.encryption(encryption);
        if trust_server_certificate {
            tiberius_config.trust_cert();
        }

        Ok(tiberius_config)
    }

    pub(crate) const fn sqlserver_tls_options(mode: SqlServerTlsMode) -> (EncryptionLevel, bool) {
        match mode {
            SqlServerTlsMode::Required => (EncryptionLevel::Required, false),
            SqlServerTlsMode::TrustServerCertificate => (EncryptionLevel::Required, true),
            SqlServerTlsMode::Disabled => (EncryptionLevel::NotSupported, false),
        }
    }

    pub(crate) fn extract_mssql_auth(auth: &Auth) -> CanyonResult<tiberius::AuthMethod> {
        match auth {
            Auth::SqlServer(sql_server_auth) => match sql_server_auth {
                SqlServerAuth::Basic { username, password } => {
                    Ok(tiberius::AuthMethod::sql_server(username, password))
                }
            },
            #[cfg(any(feature = "postgres", feature = "mysql"))]
            _ => Err(ConfigurationError::InvalidAuthentication {
                backend: DatabaseType::SqlServer,
            }
            .into()),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::__impl;
    use crate::connection::datasources::{
        Auth, DatasourceConfig, DatasourceProperties, SqlServerAuth, SqlServerTlsMode,
    };
    use tiberius::{AuthMethod, EncryptionLevel};

    #[test]
    fn test_extract_mssql_auth_basic() {
        let auth = Auth::SqlServer(SqlServerAuth::Basic {
            username: "sa".to_string(),
            password: "password123".to_string(),
        });

        let result = __impl::extract_mssql_auth(&auth).unwrap();

        match result {
            // We can only check the variant, not its internals (private fields)
            AuthMethod::SqlServer(_) => {} // success
            _ => panic!("Expected AuthMethod::SqlServer variant"),
        }
    }

    #[test]
    fn test_sqlserver_config_from_datasource_basic() {
        let datasource = DatasourceConfig {
            name: "test_source".into(),
            properties: DatasourceProperties {
                host: "localhost".into(),
                db_name: "test_db".into(),
                port: None, // default
                migrations: None,
                mssql_tls: SqlServerTlsMode::Required,
            },
            auth: Auth::SqlServer(SqlServerAuth::Basic {
                username: "sa".into(),
                password: "pass123".into(),
            }),
        };

        let config = __impl::sqlserver_config_from_datasource(&datasource).unwrap();
        assert_eq!(config.get_addr(), "localhost:1433");
    }

    #[test]
    fn sqlserver_tls_modes_map_to_secure_explicit_options() {
        assert_eq!(
            __impl::sqlserver_tls_options(SqlServerTlsMode::Required),
            (EncryptionLevel::Required, false)
        );
        assert_eq!(
            __impl::sqlserver_tls_options(SqlServerTlsMode::TrustServerCertificate),
            (EncryptionLevel::Required, true)
        );
        assert_eq!(
            __impl::sqlserver_tls_options(SqlServerTlsMode::Disabled),
            (EncryptionLevel::NotSupported, false)
        );
    }
}
