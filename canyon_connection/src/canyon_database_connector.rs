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

pub mod connection_helpers {
    use super::*;
    use auth::AuthConfig;

    #[cfg(feature = "postgres")]
    pub async fn create_postgres_connection(
        datasource: &DatasourceConfig,
    ) -> Result<DatabaseConnection, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
        let (new_client, new_connection) =
            match auth::extract_auth(&datasource.auth, DatabaseType::PostgreSql)? {
                AuthConfig::Postgres(username, password) => {
                    tokio_postgres::connect(
                        &format!(
                            "postgres://{user}:{pswd}@{host}:{port}/{db}",
                            user = username,
                            pswd = password,
                            host = datasource.properties.host,
                            port = datasource.properties.port.unwrap_or_default(),
                            db = datasource.properties.db_name
                        ),
                        NoTls,
                    )
                    .await?
                }
                _ => {
                    return Err(
                        format!("Failed to set the auth for datasource: {:?}", datasource).into(),
                    )
                }
            };

        tokio::spawn(async move {
            if let Err(e) = new_connection.await {
                eprintln!(
                    "An error occurred while trying to connect to the PostgreSQL database: {e}"
                );
            }
        });

        Ok(DatabaseConnection::Postgres(PostgreSqlConnection {
            client: new_client,
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

        match auth::extract_auth(&datasource.auth, DatabaseType::SqlServer)? {
            AuthConfig::SqlServer(auth_method) => tiberius_config.authentication(auth_method),
            _ => {
                return Err(
                    format!("Failed to set the auth for datasource: {:?}", datasource).into(),
                )
            }
        };

        tiberius_config.trust_cert(); // TODO: this should be specificaly set via user input

        let tcp = TcpStream::connect(tiberius_config.get_addr())
            .await
            .expect("Error instantiating the SqlServer TCP Stream");

        tcp.set_nodelay(true)
            .expect("Error in the SqlServer `nodelay` config");

        let client = tiberius::Client::connect(tiberius_config, tcp).await?;

        Ok(DatabaseConnection::SqlServer(SqlServerConnection {
            client: Box::leak(Box::new(client)),
        }))
    }

    #[cfg(feature = "mysql")]
    pub async fn create_mysql_connection(
        datasource: &DatasourceConfig,
    ) -> Result<DatabaseConnection, Box<(dyn std::error::Error + Send + Sync + 'static)>> {

        let (user, password) = match auth::extract_auth(&datasource.auth, DatabaseType::MySQL)? {
            AuthConfig::MySQL(username, password) => (username, password),
            _ => {
                return Err(
                    format!("Failed to set the auth for datasource: {:?}", datasource).into(),
                )
            }
        };

        let url = format!(
            "mysql://{}:{}@{}:{}/{}",
            user,
            password,
            datasource.properties.host,
            datasource.properties.port.unwrap_or_default(),
            datasource.properties.db_name
        );

        let mysql_connection = Pool::from_url(url)?;

        Ok(DatabaseConnection::MySQL(MysqlConnection {
            client: mysql_connection,
        }))
    }
}

pub mod auth {
    use std::marker::PhantomData;

    use crate::{database_type::DatabaseType, datasources::Auth};

    #[cfg(feature = "mysql")]
    use crate::datasources::MySQLAuth;
    #[cfg(feature = "postgres")]
    use crate::datasources::PostgresAuth;
    #[cfg(feature = "mssql")]
    use crate::datasources::SqlServerAuth;

    /// Custom type to act as a brigde between the parsed input auth data on the Canyon config file with serde
    /// to the internal type(s) of the database connector vendors
    pub enum AuthConfig<'a> {
        #[cfg(feature = "postgres")]
        Postgres(&'a str, &'a str),

        #[cfg(feature = "mssql")]
        SqlServer(tiberius::AuthMethod),

        #[cfg(feature = "mysql")]
        MySQL(&'a str, &'a str),

        Phanton(PhantomData<&'a ()>),
    }

    pub fn extract_auth<'a>(
        auth: &'a Auth,
        db_type: DatabaseType,
    ) -> Result<AuthConfig<'a>, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
        match db_type {
            #[cfg(feature = "postgres")]
            DatabaseType::PostgreSql => extract_postgres_auth(auth),
            #[cfg(feature = "mssql")]
            DatabaseType::SqlServer => extract_mssql_auth(auth),
            #[cfg(feature = "mysql")]
            DatabaseType::MySQL => extract_mysql_auth(auth),
        }
    }

    #[cfg(feature = "postgres")]
    fn extract_postgres_auth<'a>(
        auth: &'a Auth,
    ) -> Result<AuthConfig<'a>, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
        match auth {
            Auth::Postgres(pg_auth) => match pg_auth {
                PostgresAuth::Basic { username, password } => {
                    Ok(AuthConfig::Postgres(username, password))
                }
            },
            #[cfg(any(feature = "mssql", feature = "mysql"))]
            _ => Err("Invalid auth configuration for a SqlServer datasource.".into()),
        }
    }

    #[cfg(feature = "mssql")]
    fn extract_mssql_auth<'a>(
        auth: &'a Auth,
    ) -> Result<AuthConfig<'a>, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
        match auth {
            Auth::SqlServer(sql_server_auth) => match sql_server_auth {
                SqlServerAuth::Basic { username, password } => Ok(AuthConfig::SqlServer(
                    tiberius::AuthMethod::sql_server(username, password),
                )),
                SqlServerAuth::Integrated => {
                    Ok(AuthConfig::SqlServer(tiberius::AuthMethod::Integrated))
                }
            },
            #[cfg(any(feature = "postgres", feature = "mysql"))]
            _ => Err("Invalid auth configuration for a SqlServer datasource.".into()),
        }
    }

    #[cfg(feature = "mysql")]
    fn extract_mysql_auth<'a>(
        auth: &'a Auth,
    ) -> Result<AuthConfig<'a>, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
        match auth {
            Auth::MySQL(mysql_auth) => match mysql_auth {
                MySQLAuth::Basic { username, password } => {
                    Ok(AuthConfig::MySQL(username, password))
                }
            },
            #[cfg(any(feature = "mssql", feature = "mysql"))]
            _ => Err("Invalid auth configuration for a SqlServer datasource.".into()),
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
