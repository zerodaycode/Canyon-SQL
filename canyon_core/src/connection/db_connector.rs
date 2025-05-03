#[cfg(feature = "mssql")]
use crate::connection::clients::mssql::SqlServerConnection;
#[cfg(feature = "mysql")]
use crate::connection::clients::mysql::MysqlConnection;
#[cfg(feature = "postgres")]
use crate::connection::clients::postgresql::PostgreSqlConnection;
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
    Postgres(PostgreSqlConnection), // NOTE: *Connection means *Client?
    #[cfg(feature = "mssql")]
    SqlServer(SqlServerConnection),
    #[cfg(feature = "mysql")]
    MySQL(MysqlConnection),
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
