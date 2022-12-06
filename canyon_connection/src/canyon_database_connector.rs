use async_std::net::TcpStream;

use tiberius::{AuthMethod, Config};
use tokio_postgres::{Client, NoTls};

use crate::datasources::DatasourceProperties;

/// Represents the current supported databases by Canyon
#[derive(Debug, Eq, PartialEq, Clone, Copy, Default)]
pub enum DatabaseType {
    #[default] PostgreSql,
    SqlServer,
}

impl DatabaseType {
    /// Returns a variant from Self given a *DatasourceProperties* representing
    /// some of the available databases in `Canyon-SQL`
    pub fn from_datasource(datasource: &DatasourceProperties<'_>) -> Self {
        match datasource.db_type {
            "postgresql" => Self::PostgreSql,
            "sqlserver" => Self::SqlServer,
            _ => todo!(), // TODO Change for boxed dyn error type
        }
    }
}

/// A connection with a `PostgreSQL` database
pub struct PostgreSqlConnection {
    pub client: Client,
    // pub connection: Connection<Socket, NoTlsStream>,
}

/// A connection with a `SqlServer` database
pub struct SqlServerConnection {
    pub client: &'static mut tiberius::Client<TcpStream>,
}

/// The Canyon database connection handler. When a new query is launched,
/// the `new` associated function returns `Self`, containing in one of its
/// members an active connection against the matched database type on the
/// datasource triggering this process
///
/// !! Future of this impl. Two aspect to discuss:
/// - Should we store the active connections? And not triggering
///   this process on every query? Or it's better to open and close
///   the connection with the database on every query?
///
/// - Now that `Mutex` allow const initializations, we should
///   refactor the initialization in a real static handler?
pub struct DatabaseConnection {
    pub postgres_connection: Option<PostgreSqlConnection>,
    pub sqlserver_connection: Option<SqlServerConnection>,
    pub database_type: DatabaseType,
}

unsafe impl Send for DatabaseConnection {}
unsafe impl Sync for DatabaseConnection {}


impl DatabaseConnection {
    pub async fn new(
        datasource: &DatasourceProperties<'_>,
    ) -> Result<DatabaseConnection, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
        match datasource.db_type {
            "postgresql" => {
                let (new_client, new_connection) = tokio_postgres::connect(
                    &format!(
                        "postgres://{user}:{pswd}@{host}:{port}/{db}",
                        user = datasource.username,
                        pswd = datasource.password,
                        host = datasource.host,
                        port = datasource.port.unwrap_or_default(),
                        db = datasource.db_name
                    )[..],
                    NoTls,
                )
                .await?;

                tokio::spawn(async move {
                    if let Err(e) = new_connection.await {
                        eprintln!("An error occured while trying to connect to the PostgreSQL database: {e}");
                    }
                });

                Ok(Self {
                    postgres_connection: Some(PostgreSqlConnection {
                        client: new_client,
                        // connection: new_connection,
                    }),
                    sqlserver_connection: None,
                    database_type: DatabaseType::from_datasource(datasource),
                })
            }
            "sqlserver" => {
                let mut config = Config::new();

                config.host(datasource.host);
                config.port(datasource.port.unwrap_or_default());
                config.database(datasource.db_name);

                // Using SQL Server authentication.
                config.authentication(AuthMethod::sql_server(
                    datasource.username,
                    datasource.password,
                ));

                // on production, it is not a good idea to do this
                config.trust_cert();

                // Taking the address from the configuration, using async-std's
                // TcpStream to connect to the server.
                let tcp = TcpStream::connect(config.get_addr())
                    .await
                    .expect("Error instanciating the SqlServer TCP Stream");

                // We'll disable the Nagle algorithm. Buffering is handled
                // internally with a `Sink`.
                tcp.set_nodelay(true)
                    .expect("Error in the SqlServer `nodelay` config");

                // Handling TLS, login and other details related to the SQL Server.
                let client = tiberius::Client::connect(config, tcp).await;

                Ok(Self {
                    postgres_connection: None,
                    sqlserver_connection: Some(SqlServerConnection {
                        client: Box::leak(
                            Box::new(
                                client.expect("A failure happened connecting to the database")
                            )
                        ),
                    }),
                    database_type: DatabaseType::from_datasource(datasource),
                })
            }
            &_ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    format!(
                        "There's no `{}` database supported in Canyon-SQL",
                        datasource.db_type
                    ),
                )
                .into_inner()
                .unwrap())
            }
        }
    }
}

#[cfg(test)]
mod database_connection_handler {
    use super::*;
    use crate::CanyonSqlConfig;

    const CONFIG_FILE_MOCK_ALT: &str = r#"
        [canyon_sql]
        datasources = [
            {name = 'PostgresDS', properties.db_type = 'postgresql', properties.username = 'username', properties.password = 'random_pass', properties.host = 'localhost', properties.db_name = 'triforce'},
            {name = 'SqlServerDS', properties.db_type = 'sqlserver', properties.username = 'username2', properties.password = 'random_pass2', properties.host = '192.168.0.250.1', properties.port = 3340, properties.db_name = 'triforce2'}
        ]
    "#;

    /// Tests the behaviour of the `DatabaseType::from_datasource(...)`
    #[test]
    fn check_from_datasource() {
        let config: CanyonSqlConfig = toml::from_str(CONFIG_FILE_MOCK_ALT)
            .expect("A failure happened retrieving the [canyon_sql] section");

        let psql_ds = &config.canyon_sql.datasources[0].properties;
        let sqls_ds = &config.canyon_sql.datasources[1].properties;

        assert_eq!(
            DatabaseType::from_datasource(psql_ds),
            DatabaseType::PostgreSql
        );
        assert_eq!(
            DatabaseType::from_datasource(sqls_ds),
            DatabaseType::SqlServer
        );
    }
}
