#[cfg(feature = "mssql")]
use async_std::net::TcpStream;
#[cfg(feature = "mysql")]
use mysql_async::Pool;
#[cfg(feature = "mssql")]
use tiberius::Config;
#[cfg(feature = "postgres")]
use tokio_postgres::{Client, NoTls};

use crate::database_type::DatabaseType;
use crate::datasources::DatasourceConfig;

/// A connection with a `PostgreSQL` database
#[cfg(feature = "postgres")]
pub struct PostgreSqlConnection {
    pub client: Client,
    // pub connection: Connection<Socket, NoTlsStream>, // TODO Hold it, or not to hold it... that's the question!
}

/// A connection with a `SqlServer` database
#[cfg(feature = "mssql")]
pub struct SqlServerConnection {
    pub client: &'static mut tiberius::Client<TcpStream>,
}

/// A connection with a `Mysql` database
#[cfg(feature = "mysql")]
pub struct MysqlConnection {
    pub client: Pool,
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

unsafe impl Send for DatabaseConnection {}
unsafe impl Sync for DatabaseConnection {}

impl DatabaseConnection {
    pub async fn new(
        datasource: &DatasourceConfig,
    ) -> Result<DatabaseConnection, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
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
    ) -> Result<DatabaseConnection, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
        let (user, password) = auth::extract_postgres_auth(&datasource.auth)?;
        let url = connection_string(user, password, datasource);

        let (client, connection) = tokio_postgres::connect(&url, NoTls).await?;

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
    ) -> Result<DatabaseConnection, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
        let mut tiberius_config = Config::new();

        tiberius_config.host(&datasource.properties.host);
        tiberius_config.port(datasource.properties.port.unwrap_or_default());
        tiberius_config.database(&datasource.properties.db_name);

        let auth_config = auth::extract_mssql_auth(&datasource.auth)?;
        tiberius_config.authentication(auth_config);
        tiberius_config.trust_cert(); // TODO: this should be specificaly set via user input

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
    ) -> Result<DatabaseConnection, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
        let (user, password) = auth::extract_mysql_auth(&datasource.auth)?;
        let url = connection_string(user, password, datasource);
        let mysql_connection = Pool::from_url(url)?;

        Ok(DatabaseConnection::MySQL(MysqlConnection {
            client: mysql_connection,
        }))
    }

    #[cfg(any(feature = "postgres", feature = "mysql"))]
    fn connection_string(user: &str, pswd: &str, datasource: &DatasourceConfig) -> String {
        let server = match datasource.get_db_type() {
            #[cfg(feature = "postgres")]
            DatabaseType::PostgreSql => "postgres",

            #[cfg(feature = "mysql")]
            DatabaseType::MySQL => "mysql",
            DatabaseType::SqlServer => todo!("Connection string for MSSQL should never be reached"),
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
    use crate::datasources::Auth;

    #[cfg(feature = "mysql")]
    use crate::datasources::MySQLAuth;
    #[cfg(feature = "postgres")]
    use crate::datasources::PostgresAuth;
    #[cfg(feature = "mssql")]
    use crate::datasources::SqlServerAuth;

    #[cfg(feature = "postgres")]
    pub fn extract_postgres_auth<'a>(
        auth: &'a Auth,
    ) -> Result<(&'a str, &'a str), Box<(dyn std::error::Error + Send + Sync + 'static)>> {
        match auth {
            Auth::Postgres(pg_auth) => match pg_auth {
                PostgresAuth::Basic { username, password } => Ok((username, password)),
            },
            #[cfg(any(feature = "mssql", feature = "mysql"))]
            _ => Err("Invalid auth configuration for a Postgres datasource.".into()),
        }
    }

    #[cfg(feature = "mssql")]
    pub fn extract_mssql_auth<'a>(
        auth: &'a Auth,
    ) -> Result<tiberius::AuthMethod, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
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
    pub fn extract_mysql_auth<'a>(
        auth: &'a Auth,
    ) -> Result<(&'a str, &'a str), Box<(dyn std::error::Error + Send + Sync + 'static)>> {
        match auth {
            Auth::MySQL(mysql_auth) => match mysql_auth {
                MySQLAuth::Basic { username, password } => Ok((username, password)),
            },
            #[cfg(any(feature = "postgres", feature = "mssql"))]
            _ => Err("Invalid auth configuration for a MySQL datasource.".into()),
        }
    }
}

// TODO: && NOTE: tests defined below should be integration tests, unfortunately, since they require a new connection to be made
// Or just to split them further, and just unit test the url string generation from the actual connection instantion
// #[cfg(test)]
// mod connection_tests {
//     use tokio;
//     use super::connection_helpers::*;
//     use crate::{canyon_database_connector::DatabaseConnection, datasources::{Auth, DatasourceConfig, DatasourceProperties, PostgresAuth}};

//     #[tokio::test]
//     #[cfg(feature = "postgres")]
//     async fn test_create_postgres_connection() {
//         use crate::datasources::PostgresAuth;

//         let config = DatasourceConfig {
//             name: "PostgresDs".to_string(),
//             auth: Auth::Postgres(PostgresAuth::Basic {
//                 username: "test_user".to_string(),
//                 password: "test_password".to_string(),
//             }),
//             properties: DatasourceProperties {
//                 host: "localhost".to_string(),
//                 port: Some(5432),
//                 db_name: "test_db".to_string(),
//                 migrations: None
//             },
//         };

//         let result = create_postgres_connection(&config).await;
//         assert!(result.is_ok());
//     }

//     #[tokio::test]
//     #[cfg(feature = "mssql")]
//     async fn test_create_sqlserver_connection() {
//         use crate::datasources::SqlServerAuth;

//         let config = DatasourceConfig {
//             name: "SqlServerDs".to_string(),
//             auth: Auth::SqlServer(SqlServerAuth::Basic {
//                 username: "test_user".to_string(),
//                 password: "test_password".to_string(),
//             }),
//             properties: DatasourceProperties {
//                 host: "localhost".to_string(),
//                 port: Some(1433),
//                 db_name: "test_db".to_string(),
//                 migrations: None
//             },
//         };

//         let result = create_sqlserver_connection(&config).await;
//         assert!(result.is_ok());
//     }

//     #[tokio::test]
//     #[cfg(feature = "mysql")]
//     async fn test_create_mysql_connection() {
//         use crate::datasources::MySQLAuth;

//         let config = DatasourceConfig {
//             name: "MySQLDs".to_string(),
//             auth: Auth::MySQL(MySQLAuth::Basic {
//                 username: "test_user".to_string(),
//                 password: "test_password".to_string(),
//             }),
//             properties: DatasourceProperties {
//                 host: "localhost".to_string(),
//                 port: Some(3306),
//                 db_name: "test_db".to_string(),
//                 migrations: None,
//             },
//         };

//         let result = create_mysql_connection(&config).await;
//         assert!(result.is_ok());
//     }

//     #[tokio::test]
//     async fn test_database_connection_new() {
//         #[cfg(feature = "postgres")]
//         {
//             use crate::datasources::PostgresAuth;

//             let config = DatasourceConfig {
//                 name: "PostgresDs".to_string(),
//                 auth: Auth::Postgres(PostgresAuth::Basic {
//                     username: "test_user".to_string(),
//                     password: "test_password".to_string(),
//                 }),
//                 properties: DatasourceProperties {
//                     host: "localhost".to_string(),
//                     port: Some(5432),
//                     db_name: "test_db".to_string(),
//                     migrations: None
//                 },
//             };

//             let result = DatabaseConnection::new(&config).await;
//             assert!(result.is_ok());
//         }

//         // #[cfg(feature = "mssql")]
//         // {
//         //     let config = DatasourceConfig {
//         //         db_type: DatabaseType::SqlServer,
//         //         auth: Auth::SqlServer(SqlServerAuth::Basic {
//         //             username: "test_user".to_string(),
//         //             password: "test_password".to_string(),
//         //         }),
//         //         properties: crate::datasources::Properties {
//         //             host: "localhost".to_string(),
//         //             port: Some(1433),
//         //             db_name: "test_db".to_string(),
//         //         },
//         //     };

//         //     let result = DatabaseConnection::new(&config).await;
//         //     assert!(result.is_ok());
//         // }

//         // #[cfg(feature = "mysql")]
//         // {
//         //     let config = DatasourceConfig {
//         //         db_type: DatabaseType::MySQL,
//         //         auth: Auth::MySQL(MySQLAuth::Basic {
//         //             username: "test_user".to_string(),
//         //             password: "test_password".to_string(),
//         //         }),
//         //         properties: crate::datasources::Properties {
//         //             host: "localhost".to_string(),
//         //             port: Some(3306),
//         //             db_name: "test_db".to_string(),
//         //         },
//         //     };

//         //     let result = DatabaseConnection::new(&config).await;
//         //     assert!(result.is_ok());
//         // }
//     }
// }
