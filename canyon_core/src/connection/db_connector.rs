use crate::connection::database_type::DatabaseType;
use crate::connection::datasources::DatasourceConfig;
#[cfg(feature = "mssql")]
use crate::connection::db_clients::mssql::SqlServerConnection;
#[cfg(feature = "mysql")]
use crate::connection::db_clients::mysql::MysqlConnection;
#[cfg(feature = "postgres")]
use crate::connection::db_clients::postgresql::PostgreSqlConnection;
use crate::connection::db_connector::connection_helpers::{
    db_conn_launch_impl, db_conn_query_one_impl,
};
use crate::connection::{find_datasource_by_name_or_try_default, get_database_connection_by_ds};
use crate::mapper::RowMapper;
use crate::query_parameters::QueryParameter;
use crate::rows::{CanyonRows, FromSqlOwnedValue};
use std::error::Error;
use std::fmt::Display;
use std::future::Future;

pub trait DbConnection {
    fn query_rows<'a>(
        &self,
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
    ) -> impl Future<Output = Result<CanyonRows, Box<(dyn Error + Send + Sync)>>> + Send;

    fn query<'a, S, R>(
        &self,
        stmt: S,
        params: &[&'a (dyn QueryParameter<'a>)],
    ) -> impl Future<Output = Result<Vec<R>, Box<(dyn Error + Send + Sync)>>> + Send
    where
        S: AsRef<str> + Display + Send,
        R: RowMapper,
        Vec<R>: FromIterator<<R as RowMapper>::Output>;

    fn query_one<'a, R>(
        &self,
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
    ) -> impl Future<Output = Result<Option<R::Output>, Box<(dyn Error + Send + Sync)>>> + Send
    where
        R: RowMapper;

    /// Flexible and general method that queries the target database for a concrete instance
    /// of some type T.
    ///
    /// This is useful on statements that won't be mapped to user types (impl RowMapper) but
    /// there's a need for more flexibility on the return type. Ex: SELECT COUNT(*) from <table_name>,
    /// where there will be a single result of some numerical type
    fn query_one_for<'a, T: FromSqlOwnedValue<T>>(
        &self,
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
    ) -> impl Future<Output = Result<T, Box<(dyn Error + Send + Sync)>>> + Send;

    /// Executes the given SQL statement against the target database, being any implementor of self,
    /// returning only a numerical positive integer number reflecting the number of affected rows
    fn execute<'a>(
        &self,
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
    ) -> impl Future<Output = Result<u64, Box<(dyn Error + Send + Sync)>>> + Send;

    fn get_database_type(&self) -> Result<DatabaseType, Box<(dyn Error + Send + Sync)>>;
}

/// This impl of [` DbConnection` ] for [`&str`] allows the client to use the exposed input types
/// on the public API that works with a generic parameter to refer to a database connection
/// directly with an [`&str`] that must match one of the datasources defined
/// within the user config file
impl DbConnection for &str {
    async fn query_rows<'a>(
        &self,
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
    ) -> Result<CanyonRows, Box<(dyn Error + Send + Sync)>> {
        let conn = get_database_connection_by_ds(Some(self)).await?;
        conn.query_rows(stmt, params).await
    }

    async fn query<'a, S, R>(
        &self,
        stmt: S,
        params: &[&'a (dyn QueryParameter<'a>)],
    ) -> Result<Vec<R>, Box<(dyn Error + Send + Sync)>>
    where
        S: AsRef<str> + Display + Send,
        R: RowMapper,
        Vec<R>: FromIterator<<R as RowMapper>::Output>,
    {
        let conn = get_database_connection_by_ds(Some(self)).await?;
        conn.query(stmt, params).await
    }

    async fn query_one<'a, R>(
        &self,
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
    ) -> Result<Option<R::Output>, Box<(dyn Error + Send + Sync)>>
    where
        R: RowMapper,
    {
        let sane_ds_name = if !self.is_empty() { Some(*self) } else { None };
        let conn = get_database_connection_by_ds(sane_ds_name).await?;
        conn.query_one::<R>(stmt, params).await
    }

    async fn query_one_for<'a, T: FromSqlOwnedValue<T>>(
        &self,
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
    ) -> Result<T, Box<(dyn Error + Send + Sync)>> {
        let sane_ds_name = if !self.is_empty() { Some(*self) } else { None };
        let conn = get_database_connection_by_ds(sane_ds_name).await?;
        conn.query_one_for(stmt, params).await
    }

    async fn execute<'a>(
        &self,
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
    ) -> Result<u64, Box<(dyn Error + Send + Sync)>> {
        let sane_ds_name = if !self.is_empty() { Some(*self) } else { None };
        let conn = get_database_connection_by_ds(sane_ds_name).await?;
        conn.execute(stmt, params).await
    }

    fn get_database_type(&self) -> Result<DatabaseType, Box<(dyn Error + Send + Sync)>> {
        Ok(find_datasource_by_name_or_try_default(Some(*self))?.get_db_type())
    }
}

/// The Canyon database connection handler. When the client's program
/// starts, Canyon gets the information about the desired datasources,
/// process them and generates a pool of 1 to 1 database connection for
/// every datasource defined.
pub enum DatabaseConnection {
    // NOTE: is this a Datasource instead of a connection?
    #[cfg(feature = "postgres")]
    Postgres(PostgreSqlConnection), // NOTE: *Connection means *Client?
    #[cfg(feature = "mssql")]
    SqlServer(SqlServerConnection),
    #[cfg(feature = "mysql")]
    MySQL(MysqlConnection),
}

impl DbConnection for DatabaseConnection {
    async fn query_rows<'a>(
        &self,
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
    ) -> Result<CanyonRows, Box<(dyn Error + Send + Sync)>> {
        db_conn_launch_impl(self, stmt, params).await
    }

    async fn query<'a, S, R>(
        &self,
        stmt: S,
        params: &[&'a (dyn QueryParameter<'a>)],
    ) -> Result<Vec<R>, Box<(dyn Error + Send + Sync)>>
    where
        S: AsRef<str> + Display + Send,
        R: RowMapper,
        Vec<R>: FromIterator<<R as RowMapper>::Output>,
    {
        match self {
            #[cfg(feature = "postgres")]
            DatabaseConnection::Postgres(client) => client.query(stmt, params).await,

            #[cfg(feature = "mssql")]
            DatabaseConnection::SqlServer(client) => client.query(stmt, params).await,

            #[cfg(feature = "mysql")]
            DatabaseConnection::MySQL(client) => client.query(stmt, params).await,
        }
    }

    async fn query_one<'a, R>(
        &self,
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
    ) -> Result<Option<R::Output>, Box<(dyn Error + Send + Sync)>>
    where
        R: RowMapper,
    {
        db_conn_query_one_impl::<R>(self, stmt, params).await
    }

    async fn query_one_for<'a, T: FromSqlOwnedValue<T>>(
        &self,
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
    ) -> Result<T, Box<(dyn Error + Send + Sync)>> {
        match self {
            #[cfg(feature = "postgres")]
            DatabaseConnection::Postgres(client) => client.query_one_for(stmt, params).await,

            #[cfg(feature = "mssql")]
            DatabaseConnection::SqlServer(client) => client.query_one_for(stmt, params).await,

            #[cfg(feature = "mysql")]
            DatabaseConnection::MySQL(client) => client.query_one_for(stmt, params).await,
        }
    }
    async fn execute<'a>(
        &self,
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
    ) -> Result<u64, Box<(dyn Error + Send + Sync)>> {
        match self {
            #[cfg(feature = "postgres")]
            DatabaseConnection::Postgres(client) => client.execute(stmt, params).await,

            #[cfg(feature = "mssql")]
            DatabaseConnection::SqlServer(client) => client.execute(stmt, params).await,

            #[cfg(feature = "mysql")]
            DatabaseConnection::MySQL(client) => client.execute(stmt, params).await,
        }
    }

    fn get_database_type(&self) -> Result<DatabaseType, Box<(dyn Error + Send + Sync)>> {
        Ok(self.get_db_type())
    }
}

impl DbConnection for &mut DatabaseConnection {
    async fn query_rows<'a>(
        &self,
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
    ) -> Result<CanyonRows, Box<(dyn Error + Send + Sync)>> {
        db_conn_launch_impl(self, stmt, params).await
    }
    async fn query<'a, S, R>(
        &self,
        stmt: S,
        params: &[&'a (dyn QueryParameter<'a>)],
    ) -> Result<Vec<R>, Box<(dyn Error + Send + Sync)>>
    where
        S: AsRef<str> + Display + Send,
        R: RowMapper,
        Vec<R>: FromIterator<<R as RowMapper>::Output>,
    {
        match self {
            #[cfg(feature = "postgres")]
            DatabaseConnection::Postgres(client) => client.query(stmt, params).await,

            #[cfg(feature = "mssql")]
            DatabaseConnection::SqlServer(client) => client.query(stmt, params).await,

            #[cfg(feature = "mysql")]
            DatabaseConnection::MySQL(client) => client.query(stmt, params).await,
        }
    }

    async fn query_one<'a, R>(
        &self,
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
    ) -> Result<Option<R::Output>, Box<(dyn Error + Send + Sync)>>
    where
        R: RowMapper,
    {
        db_conn_query_one_impl::<R>(self, stmt, params).await
    }

    async fn query_one_for<'a, T: FromSqlOwnedValue<T>>(
        &self,
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
    ) -> Result<T, Box<(dyn Error + Send + Sync)>> {
        match self {
            #[cfg(feature = "postgres")]
            DatabaseConnection::Postgres(client) => client.query_one_for(stmt, params).await,

            #[cfg(feature = "mssql")]
            DatabaseConnection::SqlServer(client) => client.query_one_for(stmt, params).await,

            #[cfg(feature = "mysql")]
            DatabaseConnection::MySQL(client) => client.query_one_for(stmt, params).await,
        }
    }

    async fn execute<'a>(
        &self,
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
    ) -> Result<u64, Box<(dyn Error + Send + Sync)>> {
        match self {
            #[cfg(feature = "postgres")]
            DatabaseConnection::Postgres(client) => client.execute(stmt, params).await,

            #[cfg(feature = "mssql")]
            DatabaseConnection::SqlServer(client) => client.execute(stmt, params).await,

            #[cfg(feature = "mysql")]
            DatabaseConnection::MySQL(client) => client.execute(stmt, params).await,
        }
    }

    fn get_database_type(&self) -> Result<DatabaseType, Box<(dyn Error + Send + Sync)>> {
        Ok(self.get_db_type())
    }
}

unsafe impl Send for DatabaseConnection {}
unsafe impl Sync for DatabaseConnection {}

impl DatabaseConnection {
    pub async fn new(
        datasource: &DatasourceConfig,
    ) -> Result<DatabaseConnection, Box<(dyn Error + Send + Sync)>> {
        match datasource.get_db_type() {
            #[cfg(feature = "postgres")]
            DatabaseType::PostgreSql => {
                connection_helpers::create_postgres_connection(datasource).await
            }

            #[cfg(feature = "mssql")]
            DatabaseType::SqlServer => {
                connection_helpers::create_sqlserver_connection(datasource).await
            }

            #[cfg(feature = "mysql")]
            DatabaseType::MySQL => connection_helpers::create_mysql_connection(datasource).await,
        }
    }

    pub fn get_db_type(&self) -> DatabaseType {
        match self {
            #[cfg(feature = "postgres")]
            DatabaseConnection::Postgres(_) => DatabaseType::PostgreSql,
            #[cfg(feature = "mssql")]
            DatabaseConnection::SqlServer(_) => DatabaseType::SqlServer,
            #[cfg(feature = "mysql")]
            DatabaseConnection::MySQL(_) => DatabaseType::MySQL,
        }
    }

    #[cfg(feature = "postgres")]
    pub fn postgres_connection(&self) -> &PostgreSqlConnection {
        match self {
            DatabaseConnection::Postgres(conn) => conn,
            #[cfg(any(feature = "mssql", feature = "mysql"))]
            _ => panic!(),
        }
    }

    #[cfg(feature = "mssql")]
    pub fn sqlserver_connection(&mut self) -> &mut SqlServerConnection {
        match self {
            DatabaseConnection::SqlServer(conn) => conn,
            #[cfg(any(feature = "postgres", feature = "mysql"))]
            _ => panic!(),
        }
    }

    #[cfg(feature = "mysql")]
    pub fn mysql_connection(&self) -> &MysqlConnection {
        match self {
            DatabaseConnection::MySQL(conn) => conn,
            #[cfg(any(feature = "postgres", feature = "mssql"))]
            _ => panic!(),
        }
    }
}

mod connection_helpers {
    use super::*;

    #[cfg(feature = "postgres")]
    pub async fn create_postgres_connection(
        datasource: &DatasourceConfig,
    ) -> Result<DatabaseConnection, Box<(dyn Error + Send + Sync)>> {
        let (user, password) = auth::extract_postgres_auth(&datasource.auth)?;
        let url = connection_string(user, password, datasource);

        let (client, connection) = tokio_postgres::connect(&url, tokio_postgres::NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!(
                    "An error occurred while trying to connect to the PostgreSQL database: {e}"
                );
            }
        });

        Ok(DatabaseConnection::Postgres(PostgreSqlConnection {
            client,
        }))
    }

    #[cfg(feature = "mssql")]
    pub async fn create_sqlserver_connection(
        datasource: &DatasourceConfig,
    ) -> Result<DatabaseConnection, Box<(dyn Error + Send + Sync)>> {
        use async_std::net::TcpStream;
        let mut tiberius_config = tiberius::Config::new();

        tiberius_config.host(&datasource.properties.host);
        tiberius_config.port(datasource.properties.port.unwrap_or_default());
        tiberius_config.database(&datasource.properties.db_name);

        let auth_config = auth::extract_mssql_auth(&datasource.auth)?;
        tiberius_config.authentication(auth_config);
        tiberius_config.trust_cert(); // TODO: this should be specifically set via user input
        tiberius_config.encryption(tiberius::EncryptionLevel::NotSupported); // TODO: user input
                                                                             // TODO: in MacOS 15, this is the actual workaround. We need to investigate further
                                                                             // https://github.com/prisma/tiberius/issues/364

        let tcp = TcpStream::connect(tiberius_config.get_addr()).await?;
        tcp.set_nodelay(true)?;

        let client = tiberius::Client::connect(tiberius_config, tcp).await?;

        Ok(DatabaseConnection::SqlServer(SqlServerConnection {
            client: Box::leak(Box::new(client)),
        }))
    }

    #[cfg(feature = "mysql")]
    pub async fn create_mysql_connection(
        datasource: &DatasourceConfig,
    ) -> Result<DatabaseConnection, Box<(dyn Error + Send + Sync)>> {
        use mysql_async::Pool;

        let (user, password) = auth::extract_mysql_auth(&datasource.auth)?;
        let url = connection_string(user, password, datasource);
        let mysql_connection = Pool::from_url(url)?;

        Ok(DatabaseConnection::MySQL(MysqlConnection {
            client: mysql_connection,
        }))
    }

    // #[cfg(any(feature = "postgres", feature = "mysql"))]
    fn connection_string(user: &str, pswd: &str, datasource: &DatasourceConfig) -> String {
        let server = match datasource.get_db_type() {
            #[cfg(feature = "postgres")]
            DatabaseType::PostgreSql => "postgres",
            #[cfg(feature = "mysql")]
            DatabaseType::MySQL => "mysql",
            #[cfg(feature = "mssql")]
            DatabaseType::SqlServer => "", // # todo!("Connection string for MSSQL should never be reached"),
        };
        format!(
            "{server}://{user}:{pswd}@{host}:{port}/{db}",
            host = datasource.properties.host,
            port = datasource.properties.port.unwrap_or_default(),
            db = datasource.properties.db_name
        )
    }

    pub(crate) async fn db_conn_launch_impl<'a>(
        c: &DatabaseConnection,
        stmt: &str,
        params: &[&'a (dyn QueryParameter<'a> + 'a)],
    ) -> Result<CanyonRows, Box<dyn Error + Send + Sync>> {
        match c {
            #[cfg(feature = "postgres")]
            DatabaseConnection::Postgres(client) => client.query_rows(stmt, params).await,

            #[cfg(feature = "mssql")]
            DatabaseConnection::SqlServer(client) => client.query_rows(stmt, params).await,

            #[cfg(feature = "mysql")]
            DatabaseConnection::MySQL(client) => client.query_rows(stmt, params).await,
        }
    }

    pub(crate) async fn db_conn_query_one_impl<'a, R>(
        c: &DatabaseConnection,
        stmt: &str,
        params: &[&'a (dyn QueryParameter<'a> + 'a)],
    ) -> Result<Option<R::Output>, Box<dyn Error + Send + Sync>>
    where
        R: RowMapper,
    {
        match c {
            #[cfg(feature = "postgres")]
            DatabaseConnection::Postgres(client) => client.query_one::<R>(stmt, params).await,

            #[cfg(feature = "mssql")]
            DatabaseConnection::SqlServer(client) => client.query_one::<R>(stmt, params).await,

            #[cfg(feature = "mysql")]
            DatabaseConnection::MySQL(client) => client.query_one::<R>(stmt, params).await,
        }
    }
}

mod auth {
    use crate::connection::datasources::Auth;

    #[cfg(feature = "mysql")]
    use crate::connection::datasources::MySQLAuth;
    #[cfg(feature = "postgres")]
    use crate::connection::datasources::PostgresAuth;
    #[cfg(feature = "mssql")]
    use crate::connection::datasources::SqlServerAuth;

    #[cfg(feature = "postgres")]
    pub fn extract_postgres_auth(
        auth: &Auth,
    ) -> Result<(&str, &str), Box<(dyn std::error::Error + Send + Sync)>> {
        match auth {
            Auth::Postgres(pg_auth) => match pg_auth {
                PostgresAuth::Basic { username, password } => Ok((username, password)),
            },
            #[cfg(any(feature = "mssql", feature = "mysql"))]
            _ => Err("Invalid auth configuration for a Postgres datasource.".into()),
        }
    }

    #[cfg(feature = "mssql")]
    pub fn extract_mssql_auth(
        auth: &Auth,
    ) -> Result<tiberius::AuthMethod, Box<(dyn std::error::Error + Send + Sync)>> {
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

    #[cfg(feature = "mysql")]
    pub fn extract_mysql_auth(
        auth: &Auth,
    ) -> Result<(&str, &str), Box<(dyn std::error::Error + Send + Sync)>> {
        match auth {
            Auth::MySQL(mysql_auth) => match mysql_auth {
                MySQLAuth::Basic { username, password } => Ok((username, password)),
            },
            #[cfg(any(feature = "postgres", feature = "mssql"))]
            _ => Err("Invalid auth configuration for a MySQL datasource.".into()),
        }
    }
}
