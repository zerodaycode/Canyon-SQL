#[cfg(feature = "mssql")]
use crate::connection::clients::mssql::SqlServerConnection;
#[cfg(feature = "mysql")]
use crate::connection::clients::mysql::MySQLConnector;
#[cfg(feature = "postgres")]
use crate::connection::clients::postgresql::PostgresConnection;

use crate::connection::database_type::DatabaseType;
use crate::connection::datasources::DatasourceConfig;
use std::error::Error;

/// The Canyon database connection handler. When the client's program
/// starts, Canyon gets the information about the desired datasources,
/// process them and generates a pool of 1 to 1 database connection for
/// every datasource defined.
pub enum DatabaseConnection {
    // NOTE: is this a Datasource instead of a connection?
    #[cfg(feature = "postgres")]
    Postgres(PostgresConnection),
    #[cfg(feature = "mssql")]
    SqlServer(SqlServerConnection),
    #[cfg(feature = "mysql")]
    MySQL(MySQLConnector),
}

unsafe impl Send for DatabaseConnection {}
unsafe impl Sync for DatabaseConnection {}

impl DatabaseConnection {
    pub async fn new(datasource: &DatasourceConfig) -> Result<Self, Box<dyn Error + Send + Sync>> {
        // Add connection pooling at the client level for better performance
        match datasource.get_db_type() {
            #[cfg(feature = "postgres")]
            DatabaseType::PostgreSql => Ok(Self::Postgres(
                connection_helpers::create_postgres_connection(datasource).await?,
            )),

            #[cfg(feature = "mssql")]
            DatabaseType::SqlServer => Ok(Self::SqlServer(
                connection_helpers::create_sqlserver_connection(datasource).await?,
            )),

            #[cfg(feature = "mysql")]
            DatabaseType::MySQL => Ok(Self::MySQL(
                connection_helpers::create_mysql_connection(datasource).await?,
            )),
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

    /*
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
    */
}

mod connection_helpers {
    use super::*;
    use crate::connection::{
        MsManager, PgManager, PostgresConnectionPool, SqlServerConnectionPool,
    };

    use tokio_postgres::NoTls;

    #[cfg(feature = "postgres")]
    pub(crate) async fn create_postgres_connection(
        datasource: &DatasourceConfig,
    ) -> Result<PostgresConnection, Box<dyn Error + Send + Sync>> {
        let (user, password) = auth::extract_postgres_auth(&datasource.auth)?;

        // Use optimized connection settings
        let mut config = tokio_postgres::Config::new();
        config.host(&datasource.properties.host);
        config.port(datasource.properties.port.unwrap_or_default());
        config.dbname(&datasource.properties.db_name);
        config.user(user);
        config.password(password);

        // Optimize connection settings for better performance
        config.connect_timeout(std::time::Duration::from_secs(5));
        config.keepalives_idle(std::time::Duration::from_secs(30));
        config.keepalives_interval(std::time::Duration::from_secs(10));
        config.keepalives_retries(3);

        let manager = PgManager::new(config, NoTls);
        let pool = bb8::Pool::builder().max_size(10u32).build(manager).await?;

        PostgresConnection::new(PostgresConnectionPool::from(pool))
    }

    #[cfg(feature = "mssql")]
    pub(crate) async fn create_sqlserver_connection(
        datasource: &DatasourceConfig,
    ) -> Result<SqlServerConnection, Box<dyn Error + Send + Sync>> {
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

        let manager = MsManager::new(tiberius_config);
        let pool = bb8::Pool::builder().max_size(10u32).build(manager).await?;

        SqlServerConnection::new(SqlServerConnectionPool::from(pool))
    }

    #[cfg(feature = "mysql")]
    pub(crate) async fn create_mysql_connection(
        datasource: &DatasourceConfig,
    ) -> Result<MySQLConnector, Box<dyn Error + Send + Sync>> {
        let (user, password) = auth::extract_mysql_auth(&datasource.auth)?;
        let url = connection_string(user, password, datasource);

        // TODO: the pool constrains must be adquired from the datasource config
        let pool_constraints =
            mysql_async::PoolConstraints::new(2, 10).ok_or("Failure launching the MySQL pool")?;

        let mysql_opts = mysql_async::Opts::from_url(&url)?;
        let mysql_opts_builder = mysql_async::OptsBuilder::from_opts(mysql_opts)
            .pool_opts(mysql_async::PoolOpts::default().with_constraints(pool_constraints));

        Ok(MySQLConnector::new(mysql_async::Pool::new(
            mysql_opts_builder,
        )))
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
    ) -> Result<(&str, &str), Box<dyn std::error::Error + Send + Sync>> {
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

    #[cfg(feature = "mysql")]
    pub fn extract_mysql_auth(
        auth: &Auth,
    ) -> Result<(&str, &str), Box<dyn std::error::Error + Send + Sync>> {
        match auth {
            Auth::MySQL(mysql_auth) => match mysql_auth {
                MySQLAuth::Basic { username, password } => Ok((username, password)),
            },
            #[cfg(any(feature = "postgres", feature = "mssql"))]
            _ => Err("Invalid auth configuration for a MySQL datasource.".into()),
        }
    }
}
