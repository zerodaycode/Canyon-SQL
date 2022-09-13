use async_std::net::TcpStream;

use tiberius::{Config, AuthMethod};
use tokio_postgres::{Client, Connection, NoTls, Socket, tls::NoTlsStream};

use crate::datasources::DatasourceProperties;


/// A connection with a `PostgreSQL` database
pub struct PostgreSqlConnection {
    pub client: Client,
    pub connection: Connection<Socket, NoTlsStream>
}

/// A connection with a `SqlServer` database
pub struct SqlServerConnection {
    pub client: tiberius::Client<TcpStream>
}

/// The Canyon database connection handler.
pub struct DatabaseConnection {
    pub postgres_connection: Option<PostgreSqlConnection>,
    pub sqlserver_connection: Option<SqlServerConnection>
}

unsafe impl Send for DatabaseConnection {}
unsafe impl Sync for DatabaseConnection {}

impl DatabaseConnection {
    pub async fn new(datasource: &DatasourceProperties<'_>) -> Result<DatabaseConnection, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
        match datasource.db_type {
            "postgresql" => {
                let (new_client, new_connection) =
                    tokio_postgres::connect(
                    &format!(
                        "postgres://{user}:{pswd}@{host}/{db}",
                            user = datasource.username,
                            pswd = datasource.password,
                            host = datasource.host,
                            db = datasource.db_name
                        )[..], 
                    NoTls
                    ).await?;

                Ok(Self {
                    postgres_connection: Some(PostgreSqlConnection {
                        client: new_client,
                        connection: new_connection
                    }),
                    sqlserver_connection: None
                })
            },
            "sqlserver" => {
                let mut config = Config::new();

                config.host("ecomt.database.windows.net");
                config.port(1433);
                config.database("OteaCenterDes");

                // Using SQL Server authentication.
                config.authentication(AuthMethod::sql_server("administrador", "ny0crzlp@"));

                // on production, it is not a good idea to do this
                config.trust_cert();

                // Taking the address from the configuration, using async-std's
                // TcpStream to connect to the server.
                let tcp = TcpStream::connect(config.get_addr()).await
                    .ok().expect("Error instanciating the SqlServer TCP Stream");

                // We'll disable the Nagle algorithm. Buffering is handled
                // internally with a `Sink`.
                tcp.set_nodelay(true).ok().expect("Error in the SqlServer `nodelay` config");

                // Handling TLS, login and other details related to the SQL Server.
                let client = tiberius::Client::connect(config, tcp).await;

                Ok(Self {
                    postgres_connection: None,
                    sqlserver_connection: Some(SqlServerConnection {
                        client: client.ok().expect("Fallo en la conexión a la BBDD")
                    })
                })
            },
            &_ => return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    format!(
                        "There's no `{}` database supported in Canyon-SQL", 
                        datasource.db_type
                    )
                ).into_inner().unwrap()
            )
        }
    }
}


