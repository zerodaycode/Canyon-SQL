use serde::Deserialize;

#[cfg(feature = "mssql")]
use async_std::net::TcpStream;
#[cfg(feature = "mysql")]
use mysql_async::Pool;
#[cfg(feature = "mssql")]
use tiberius::{AuthMethod, Config};
#[cfg(feature = "postgres")]
use tokio_postgres::{Client, NoTls};

use crate::datasources::{Auth, DatasourceConfig};

/// Represents the current supported databases by Canyon
#[derive(Deserialize, Debug, Eq, PartialEq, Clone, Copy)]
pub enum DatabaseType {
    #[serde(alias = "postgres", alias = "postgresql")]
    #[cfg(feature = "postgres")]
    PostgreSql,
    #[serde(alias = "sqlserver", alias = "mssql")]
    #[cfg(feature = "mssql")]
    SqlServer,
    #[serde(alias = "mysql")]
    #[cfg(feature = "mysql")]
    MySQL,
}

impl From<&Auth> for DatabaseType {
    fn from(value: &Auth) -> Self {
        match value {
            #[cfg(feature = "postgres")]
            crate::datasources::Auth::Postgres(_) => DatabaseType::PostgreSql,
            #[cfg(feature = "mssql")]
            crate::datasources::Auth::SqlServer(_) => DatabaseType::SqlServer,
            #[cfg(feature = "mysql")]
            crate::datasources::Auth::MySQL(_) => DatabaseType::MySQL,
        }
    }
}

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

/*
 * NOTE: initial ideas for making a better codebase as well as we improve our public API in
 * order to make the Canyon-SQL DAO code much more unit-testable for our end users
 *
 * 1- Type above are not *Connection, they are *Client. They also may be converted to unit structs.
 * 2- DatabaseConnection is not again a Connection. Is an aggregate of Clients.
 * 3- impl DatabaseClient (when renamed) should be refactored, since it's really bloated
 * 4- Find a better way to handle the _()=> wildcard patterns on the matching ops
 *
 * 5- make public the function that retrieves the datasource from the global container, so
 *    users and macros can now fetch datasources, and macros fn params can be no ds => same
 *    behaviour, but calling the public api from the macro, avoiding to make public the global
 *    constant, only the function (that's really nice! :) ) and the *_datasource and *_ds functions
 *    can be now _conn or similar. Even we can decide to remove the default one and make always
 *    explicit the pass in of the connection
 *
 *  5.1- Make a new trait (this really can be a DatabaseConnection) that makes the execution of
 *  the queries a mockable entity by a third party library (ex: mockall). That would be an option
 *  over having *Connection
 *
 *  6- remove trait bounds that doesn't make sense. RowMapper shouldn't be :Transaction<T>,
 *  QueryBuilder shouldn't be CrudOperations. There's for sure more
 *
 *  7- Consider if we need a new ` canyon_mapper` module to decouple the mapping data ideas from
 *  the CRUD idea
 *
 *  8- Review how the querybuilder fetches the connection, as we may acomplish the same goal as
 *  the one defined in 5, using datasources (or clients)
 *
 *  9- Abort the compilation process earlier if there's no datasource defined. Also, do the same
 *  for when there's no cfg feature selected (Canyon can't work without a database) and we may not
 *  write code that assumes a default NEVER
 *
 *  10- An idea for refactoring the global context of the datasources would be... to not to have
 *  it! That will force the user to manage the connection everytime, but produces less painful code
 *  without global contexts, but maybe we're able to find a better intermediate solution
 *
 *  11- Can we make the query executors being part of another different thing instead of CRUD? Can
 *  we made them into a better idea and execution of the same?
 */

/// The Canyon database connection handler. When the client's program
/// starts, Canyon gets the information about the desired datasources,
/// process them and generates a pool of 1 to 1 database connection for
/// every datasource defined.
pub enum DatabaseConnection {
    #[cfg(feature = "postgres")]
    Postgres(PostgreSqlConnection),
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
                let (username, password) = match &datasource.auth {
                    crate::datasources::Auth::Postgres(postgres_auth) => match postgres_auth {
                        crate::datasources::PostgresAuth::Basic { username, password } => {
                            (username.as_str(), password.as_str())
                        }
                    },
                    #[cfg(feature = "mssql")]
                    crate::datasources::Auth::SqlServer(_) => {
                        panic!("Found SqlServer auth configuration for a PostgreSQL datasource")
                    }
                    #[cfg(feature = "mysql")]
                    crate::datasources::Auth::MySQL(_) => {
                        panic!("Found MySql auth configuration for a PostgreSQL datasource")
                    }
                };
                let (new_client, new_connection) = tokio_postgres::connect(
                    &format!(
                        "postgres://{user}:{pswd}@{host}:{port}/{db}",
                        user = username,
                        pswd = password,
                        host = datasource.properties.host,
                        port = datasource.properties.port.unwrap_or_default(),
                        db = datasource.properties.db_name
                    )[..],
                    NoTls,
                )
                .await?;

                tokio::spawn(async move {
                    if let Err(e) = new_connection.await {
                        eprintln!("An error occurred while trying to connect to the PostgreSQL database: {e}");
                    }
                });

                Ok(DatabaseConnection::Postgres(PostgreSqlConnection {
                    client: new_client,
                    // connection: new_connection,
                }))
            }
            #[cfg(feature = "mssql")]
            DatabaseType::SqlServer => {
                let mut config = Config::new();

                config.host(&datasource.properties.host);
                config.port(datasource.properties.port.unwrap_or_default());
                config.database(&datasource.properties.db_name);

                // Using SQL Server authentication.
                config.authentication(match &datasource.auth {
                    #[cfg(feature = "postgres")]
                    crate::datasources::Auth::Postgres(_) => {
                        panic!("Found PostgreSQL auth configuration for a SqlServer database")
                    }
                    crate::datasources::Auth::SqlServer(sql_server_auth) => match sql_server_auth {
                        crate::datasources::SqlServerAuth::Basic { username, password } => {
                            AuthMethod::sql_server(username, password)
                        }
                        crate::datasources::SqlServerAuth::Integrated => AuthMethod::Integrated,
                    },
                    #[cfg(feature = "mysql")]
                    crate::datasources::Auth::MySQL(_) => {
                        panic!("Found PostgreSQL auth configuration for a SqlServer database")
                    }
                });

                // on production, it is not a good idea to do this. We should upgrade
                // Canyon in future versions to allow the user take care about this
                // configuration
                config.trust_cert();

                // Taking the address from the configuration, using async-std's
                // TcpStream to connect to the server.
                let tcp = TcpStream::connect(config.get_addr())
                    .await
                    .expect("Error instantiating the SqlServer TCP Stream");

                // We'll disable the Nagle algorithm. Buffering is handled
                // internally with a `Sink`.
                tcp.set_nodelay(true)
                    .expect("Error in the SqlServer `nodelay` config");

                // Handling TLS, login and other details related to the SQL Server.
                let client = tiberius::Client::connect(config, tcp).await;

                Ok(DatabaseConnection::SqlServer(SqlServerConnection {
                    client: Box::leak(Box::new(
                        client.expect("A failure happened connecting to the database"),
                        // TODO: with details of the failure and the datasource that triggered them
                    )),
                }))
            }
            #[cfg(feature = "mysql")]
            DatabaseType::MySQL => {
                let (user, password) = match &datasource.auth {
                    #[cfg(feature = "mssql")]
                    crate::datasources::Auth::SqlServer(_) => {
                        panic!("Found SqlServer auth configuration for a PostgreSQL datasource")
                    }
                    #[cfg(feature = "postgres")]
                    crate::datasources::Auth::Postgres(_) => {
                        panic!("Found MySql auth configuration for a PostgreSQL datasource")
                    }
                    #[cfg(feature = "mysql")]
                    crate::datasources::Auth::MySQL(mysql_auth) => match mysql_auth {
                        crate::datasources::MySQLAuth::Basic { username, password } => {
                            (username, password)
                        }
                    },
                };

                //TODO add options to optionals params in url

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
                    client: { mysql_connection },
                }))
            }
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

#[cfg(test)]
mod database_connection_handler {
    use super::*;
    use crate::CanyonSqlConfig;

    /// Tests the behaviour of the `DatabaseType::from_datasource(...)`
    #[test]
    fn check_from_datasource() {
        #[cfg(all(feature = "postgres", feature = "mssql", feature = "mysql"))]
        {
            const CONFIG_FILE_MOCK_ALT_ALL: &str = r#"
                [canyon_sql]
                datasources = [
                    {name = 'PostgresDS', auth = { postgresql = { basic = { username = "postgres", password = "postgres" } } }, properties.host = 'localhost', properties.db_name = 'triforce', properties.migrations='enabled' },
                    {name = 'SqlServerDS', auth = { sqlserver = { basic = { username = "sa", password = "SqlServer-10" } } }, properties.host = '192.168.0.250.1', properties.port = 3340, properties.db_name = 'triforce2', properties.migrations='disabled' },
                    {name = 'MysqlDS', auth = { mysql = { basic = { username = "root", password = "root" } } }, properties.host = '192.168.0.250.1', properties.port = 3340, properties.db_name = 'triforce2', properties.migrations='disabled' }
                ]
            "#;
            let config: CanyonSqlConfig = toml::from_str(CONFIG_FILE_MOCK_ALT_ALL)
                .expect("A failure happened retrieving the [canyon_sql] section");
            assert_eq!(
                config.canyon_sql.datasources[0].get_db_type(),
                DatabaseType::PostgreSql
            );
            assert_eq!(
                config.canyon_sql.datasources[1].get_db_type(),
                DatabaseType::SqlServer
            );
            assert_eq!(
                config.canyon_sql.datasources[2].get_db_type(),
                DatabaseType::MySQL
            );
        }

        #[cfg(feature = "postgres")]
        {
            const CONFIG_FILE_MOCK_ALT_PG: &str = r#"
                [canyon_sql]
                datasources = [
                    {name = 'PostgresDS', auth = { postgresql = { basic = { username = "postgres", password = "postgres" } } }, properties.host = 'localhost', properties.db_name = 'triforce', properties.migrations='enabled' },
                ]
            "#;
            let config: CanyonSqlConfig = toml::from_str(CONFIG_FILE_MOCK_ALT_PG)
                .expect("A failure happened retrieving the [canyon_sql] section");
            assert_eq!(
                config.canyon_sql.datasources[0].get_db_type(),
                DatabaseType::PostgreSql
            );
        }

        #[cfg(feature = "mssql")]
        {
            const CONFIG_FILE_MOCK_ALT_MSSQL: &str = r#"
                [canyon_sql]
                datasources = [
                    {name = 'SqlServerDS', auth = { sqlserver = { basic = { username = "sa", password = "SqlServer-10" } } }, properties.host = '192.168.0.250.1', properties.port = 3340, properties.db_name = 'triforce2', properties.migrations='disabled' }
                ]
            "#;
            let config: CanyonSqlConfig = toml::from_str(CONFIG_FILE_MOCK_ALT_MSSQL)
                .expect("A failure happened retrieving the [canyon_sql] section");
            assert_eq!(
                config.canyon_sql.datasources[0].get_db_type(),
                DatabaseType::SqlServer
            );
        }

        #[cfg(feature = "mysql")]
        {
            const CONFIG_FILE_MOCK_ALT_MYSQL: &str = r#"
                [canyon_sql]
                datasources = [
                    {name = 'MysqlDS', auth = { mysql = { basic = { username = "root", password = "root" } } }, properties.host = '192.168.0.250.1', properties.port = 3340, properties.db_name = 'triforce2', properties.migrations='disabled' }
                ]
            "#;

            let config: CanyonSqlConfig = toml::from_str(CONFIG_FILE_MOCK_ALT_MYSQL)
                .expect("A failure happened retrieving the [canyon_sql] section");
            assert_eq!(
                config.canyon_sql.datasources[0].get_db_type(),
                DatabaseType::MySQL
            );
        }
    }
}
