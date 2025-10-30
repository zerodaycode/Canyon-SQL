use crate::connection::clients::mssql::sqlserver_query_launcher::execute_query;
use crate::connection::contracts::DbConnection;
use crate::connection::database_type::DatabaseType;
use crate::connection::datasources::DatasourceConfig;
use crate::connection::{MsManager, SqlServerConnectionPool};
use crate::mapper::RowMapper;
use crate::query::parameters::QueryParameter;
use crate::rows::{CanyonRows, FromSqlOwnedValue};
use bb8::PooledConnection;
use std::error::Error;
use tiberius::Query;

/// A connection with a `SqlServer` database
pub struct SqlServerConnector(SqlServerConnectionPool);

impl SqlServerConnector {
    pub async fn new(config: &DatasourceConfig) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(Self(__impl::create_sqlserver_connector(config).await?))
    }
    pub async fn get_pooled(
        &self,
    ) -> Result<PooledConnection<'_, MsManager>, Box<dyn Error + Send + Sync>> {
        Ok(self.0.get().await?)
    }
}

impl DbConnection for SqlServerConnector {
    async fn query_rows(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<CanyonRows, Box<dyn Error + Send + Sync>> {
        let mut conn = self.get_pooled().await?;
        let result = execute_query(stmt, params, &mut conn)
            .await?
            .into_results()
            .await?
            .into_iter()
            .flatten()
            .collect();

        Ok(CanyonRows::Tiberius(result))
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
        let mut conn = self.get_pooled().await?;
        Ok(execute_query(stmt.as_ref(), params, &mut conn)
            .await?
            .into_results()
            .await?
            .into_iter()
            .flatten()
            .flat_map(|row| R::deserialize_sqlserver(&row))
            .collect::<Vec<R>>())
    }

    async fn query_one<R>(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<Option<R::Output>, Box<dyn Error + Send + Sync>>
    where
        R: RowMapper,
    {
        let mut conn = self.get_pooled().await?;

        let result = execute_query(stmt, params, &mut conn)
            .await?
            .into_row()
            .await?;

        match result {
            Some(r) => Ok(Some(R::deserialize_sqlserver(&r)?)),
            None => Ok(None),
        }
    }

    async fn query_one_for<T: FromSqlOwnedValue<T>>(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<T, Box<dyn Error + Send + Sync>> {
        let mut conn = self.get_pooled().await?;
        let row = crate::connection::clients::mssql::sqlserver_query_launcher::execute_query(
            stmt, params, &mut conn,
        )
        .await?
        .into_row()
        .await?
        .ok_or_else(|| {
            format!(
                "Failure executing 'query_one_for' while retrieving the first row with stmt: {:?}",
                stmt
            )
        })?;

        Ok(row
            .into_iter()
            .map(T::from_sql_owned)
            .collect::<Vec<_>>()
            .remove(0)?
            .ok_or_else(|| format!("Failure executing 'query_one_for' while retrieving the first column value on the first row with stmt: {:?}", stmt))?
        )
    }

    async fn execute(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<u64, Box<dyn Error + Send + Sync>> {
        let mssql_query = crate::connection::clients::mssql::sqlserver_query_launcher::generate_mssql_query_client(stmt, params).await;
        let mut conn = self.get_pooled().await?;

        mssql_query
            .execute(&mut conn)
            .await
            .map(|r| r.total())
            .map_err(From::from)
    }

    fn get_database_type(&self) -> Result<DatabaseType, Box<dyn Error + Send + Sync>> {
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
    ) -> Result<QueryStream<'a>, Box<dyn Error + Send + Sync>> {
        let mssql_query = generate_mssql_query_client(stmt, params).await;
        mssql_query.query(conn).await.map_err(From::from)
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
    use crate::connection::datasources::{Auth, SqlServerAuth};
    use bb8::Pool;
    use std::sync::Arc;
    use tiberius::Config;

    pub(crate) async fn create_sqlserver_connector(
        datasource: &DatasourceConfig,
    ) -> Result<Arc<Pool<MsManager>>, Box<dyn Error + Send + Sync>> {
        let sqlserver_config = sqlserver_config_from_datasource(datasource)?;
        // let tcp = TcpStream::connect(tiberius_config.get_addr()).await?;
        // tcp.set_nodelay(true)?;

        let manager = MsManager::new(sqlserver_config);
        let pool = bb8::Pool::builder().max_size(10u32).build(manager).await?;

        Ok(SqlServerConnectionPool::from(pool))
    }

    fn sqlserver_config_from_datasource(
        datasource: &DatasourceConfig,
    ) -> Result<Config, Box<dyn Error + Send + Sync>> {
        let mut tiberius_config = tiberius::Config::new();

        tiberius_config.host(&datasource.properties.host);
        tiberius_config.port(datasource.get_port_or_default_by_db());
        tiberius_config.database(&datasource.properties.db_name);

        let auth_config = extract_mssql_auth(&datasource.auth)?;
        tiberius_config.authentication(auth_config);
        tiberius_config.trust_cert(); // TODO: this should be specifically set via user input
        tiberius_config.encryption(tiberius::EncryptionLevel::NotSupported); // TODO: user input
        // TODO: in MacOS 15, this is the actual workaround. We need to investigate further
        // https://github.com/prisma/tiberius/issues/364

        Ok(tiberius_config)
    }

    pub fn extract_mssql_auth(
        auth: &Auth,
    ) -> Result<tiberius::AuthMethod, Box<dyn std::error::Error + Send + Sync>> {
        match auth {
            Auth::SqlServer(sql_server_auth) => match sql_server_auth {
                SqlServerAuth::Basic { username, password } => {
                    Ok(tiberius::AuthMethod::sql_server(username, password))
                }
                SqlServerAuth::Integrated => Ok(tiberius::AuthMethod::Integrated),
            },
            #[cfg(any(feature = "postgres", feature = "mysql"))]
            _ => Err("Invalid auth configuration for a SqlServer datasource.".into()),
        }
    }
}
