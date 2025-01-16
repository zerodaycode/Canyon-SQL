#[cfg(feature = "mssql")]
use async_std::net::TcpStream;
#[cfg(feature = "mysql")]
use mysql_async::Pool;
#[cfg(feature = "mssql")]
use tiberius::{AuthMethod, Config};
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
            #[cfg(all(feature = "postgres", feature = "mssql", feature = "mysql"))]
            _ => panic!(),
        }
    }

    #[cfg(feature = "mssql")]
    pub fn sqlserver_connection(&mut self) -> &mut SqlServerConnection {
        match self {
            DatabaseConnection::SqlServer(conn) => conn,
            #[cfg(all(feature = "postgres", feature = "mssql", feature = "mysql"))]
            _ => panic!(),
        }
    }

    #[cfg(feature = "mysql")]
    pub fn mysql_connection(&self) -> &MysqlConnection {
        match self {
            DatabaseConnection::MySQL(conn) => conn,
            #[cfg(all(feature = "postgres", feature = "mssql", feature = "mysql"))]
            _ => panic!(),
        }
    }
}

pub mod connection_helpers {
    use super::*;

    #[cfg(feature = "postgres")]
    pub async fn create_postgres_connection(
        datasource: &DatasourceConfig,
    ) -> Result<DatabaseConnection, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
        use crate::datasources::{Auth, PostgresAuth};

        let (username, password) = match &datasource.auth {
            Auth::Postgres(postgres_auth)
                if matches!(
                    postgres_auth,
                    PostgresAuth::Basic { .. }
                ) =>
            {
                let PostgresAuth::Basic { username, password } = postgres_auth;
                (username.as_str(), password.as_str())
            }
            _ => return Err("Invalid auth configuration for a PostgreSQL datasource".into()),
        };

        let (new_client, new_connection) = tokio_postgres::connect(
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
        .await?;

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
        use crate::datasources::{Auth, SqlServerAuth};

        let mut config = Config::new();

        config.host(&datasource.properties.host);
        config.port(datasource.properties.port.unwrap_or_default());
        config.database(&datasource.properties.db_name);

        config.authentication(match &datasource.auth {
            Auth::SqlServer(sql_server_auth) => match sql_server_auth {
                SqlServerAuth::Basic { username, password } => {
                    AuthMethod::sql_server(username, password)
                }
                SqlServerAuth::Integrated => AuthMethod::Integrated,
            },
            _ => return Err("Invalid auth configuration for a SqlServer datasource".into()),
        });

        config.trust_cert(); // TODO: this should be specificaly set via user input

        let tcp = TcpStream::connect(config.get_addr())
            .await
            .expect("Error instantiating the SqlServer TCP Stream");

        tcp.set_nodelay(true)
            .expect("Error in the SqlServer `nodelay` config");

        let client = tiberius::Client::connect(config, tcp).await?;

        Ok(DatabaseConnection::SqlServer(SqlServerConnection {
            client: Box::leak(Box::new(client)),
        }))
    }

    #[cfg(feature = "mysql")]
    pub async fn create_mysql_connection(
        datasource: &DatasourceConfig,
    ) -> Result<DatabaseConnection, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
        use crate::datasources::{Auth, MySQLAuth};

        let (user, password) = match &datasource.auth {
            Auth::MySQL(MySQLAuth::Basic {
                username,
                password,
            }) => (username, password),
            _ => return Err("Invalid auth configuration for a MySQL datasource".into()),
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
