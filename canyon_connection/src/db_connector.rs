use std::fmt::Display;

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
use canyon_core::query::{DbConnection, Transaction};
use canyon_core::query_parameters::QueryParameter;
use canyon_core::rows::CanyonRows;

use async_trait::async_trait;

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

#[async_trait]
impl Transaction<Self> for DatabaseConnection {
    async fn query<'a, C, S, Z>(
        stmt: S,
        params: Z,
        db_conn: &C,
    ) -> Result<CanyonRows, Box<(dyn std::error::Error + Sync + Send + 'static)>>
    where
        S: AsRef<str> + std::fmt::Display + Sync + Send + 'a,
        Z: AsRef<[&'a dyn QueryParameter<'a>]> + Sync + Send + 'a,
        C: DbConnection + Display + Sync + Send + 'a,
    {
        db_conn.launch(stmt.as_ref(), params.as_ref()).await
    }
}

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

    // #[cfg(any(feature = "postgres", feature = "mysql"))]
    fn connection_string(user: &str, pswd: &str, datasource: &DatasourceConfig) -> String {
        let server = match datasource.get_db_type() {
            #[cfg(feature = "postgres")]
            DatabaseType::PostgreSql => "postgres",
            #[cfg(feature = "mysql")]
            DatabaseType::MySQL => "mysql",
            #[cfg(feature = "mssql")]
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

#[cfg(feature = "postgres")]
mod postgres_query_launcher {
    use super::*;
    #[async_trait]
    impl DbConnection for PostgreSqlConnection {
        async fn launch(
            &self,
            stmt: &str,
            params: &[&dyn QueryParameter<'_>],
        ) -> Result<CanyonRows, Box<(dyn std::error::Error + Sync + Send + 'static)>> {
            let mut m_params = Vec::new();
            for param in params {
                m_params.push((*param).as_postgres_param());
            }

            let r = self.client.query(stmt, m_params.as_slice()).await?;

            Ok(CanyonRows::Postgres(r))
        }
    }
}

#[cfg(feature = "mssql")]
mod sqlserver_query_launcher {
    use super::SqlServerConnection;
    use async_trait::async_trait;
    use canyon_core::{query::DbConnection, query_parameters::QueryParameter, rows::CanyonRows};
    use tiberius::Query;

    #[async_trait]
    impl DbConnection for SqlServerConnection {
        async fn launch(
            &self,
            stmt: &str,
            params: &[&dyn QueryParameter<'_>],
        ) -> Result<CanyonRows, Box<(dyn std::error::Error + Sync + Send + 'static)>> {
            // Re-generate de insert statement to adequate it to the SQL SERVER syntax to retrieve the PK value(s) after insert
            // TODO: redo this branch into the generated queries, before the MACROS
            // if stmt.contains("RETURNING") {
            //     let c = stmt.clone();
            //     let temp = c.split_once("RETURNING").unwrap();
            //     let temp2 = temp.0.split_once("VALUES").unwrap();
            //
            //     *stmt = format!(
            //         "{} OUTPUT inserted.{} VALUES {}",
            //         temp2.0.trim(),
            //         temp.1.trim(),
            //         temp2.1.trim()
            //     );
            // }

            // TODO: We must address the query generation. Look at the returning example, or the
            // replace below. We may use our own type Query to address this concerns when the query
            // is generated
            let mut mssql_query = Query::new(stmt.to_owned().replace('$', "@P"));
            // params
            //     .into_iter()
            //     .for_each(|param| mssql_query.bind(*param));

            for param in params.clone() {
                let p = param.clone();
                mssql_query.bind(p)
            }
            #[allow(mutable_transmutes)]
            let sqlservconn = unsafe {
                std::mem::transmute::<&SqlServerConnection, &mut SqlServerConnection>(self)
            };
            let _results = mssql_query
                .query(sqlservconn.client)
                .await?
                .into_results()
                .await?;

            Ok(CanyonRows::Tiberius(
                _results.into_iter().flatten().collect(),
            ))
        }
    }
}

#[cfg(feature = "mysql")]
mod mysql_query_launcher {
    #[cfg(feature = "mysql")]
    pub const DETECT_PARAMS_IN_QUERY: &str = r"\$([\d])+";
    #[cfg(feature = "mysql")]
    pub const DETECT_QUOTE_IN_QUERY: &str = r#"\"|\\"#;

    use std::sync::Arc;

    use async_trait::async_trait;
    use canyon_core::query::DbConnection;
    use mysql_async::prelude::Query;
    use mysql_async::QueryWithParams;
    use mysql_async::Value;

    use super::MysqlConnection;

    use canyon_core::query_parameters::QueryParameter;
    use canyon_core::rows::CanyonRows;
    use mysql_async::Row;
    use mysql_common::constants::ColumnType;
    use mysql_common::row;
    use regex::Regex;

    #[async_trait]
    impl DbConnection for MysqlConnection {
        async fn launch(
            &self,
            stmt: &str,
            params: &[&dyn QueryParameter<'_>],
        ) -> Result<CanyonRows, Box<(dyn std::error::Error + Sync + Send + 'static)>> {
        let mysql_connection = self.client.get_conn().await?;

        let stmt_with_escape_characters = regex::escape(&stmt);
        let query_string =
            Regex::new(DETECT_PARAMS_IN_QUERY)?.replace_all(&stmt_with_escape_characters, "?");

        let mut query_string = Regex::new(DETECT_QUOTE_IN_QUERY)?
            .replace_all(&query_string, "")
            .to_string();

        let mut is_insert = false;
        if let Some(index_start_clausule_returning) = query_string.find(" RETURNING") {
            query_string.truncate(index_start_clausule_returning);
            is_insert = true;
        }

        let params_query: Vec<Value> =
            reorder_params(&stmt, params, |f| (*f).as_mysql_param().to_value());

        let query_with_params = QueryWithParams {
            query: query_string,
            params: params_query,
        };

        let mut query_result = query_with_params
            .run(mysql_connection)
            .await
            .expect("Error executing query in mysql");

        let result_rows = if is_insert {
            let last_insert = query_result
                .last_insert_id()
                .map(Value::UInt)
                .expect("Error getting pk id in insert");

            vec![row::new_row(
                vec![last_insert],
                Arc::new([mysql_async::Column::new(ColumnType::MYSQL_TYPE_UNKNOWN)]),
            )]
        } else {
            query_result
                .collect::<Row>()
                .await
                .expect("Error resolved trait FromRow in mysql")
        };
        let a = CanyonRows::MySQL(result_rows);
        Ok(a)
        }    }

    #[cfg(feature = "mysql")]
    fn reorder_params<T>(
        stmt: &str,
        params: &[&'_ dyn QueryParameter<'_>],
        fn_parser: impl Fn(&&dyn QueryParameter<'_>) -> T,
    ) -> Vec<T> {
        let mut ordered_params = vec![];
        let rg = regex::Regex::new(DETECT_PARAMS_IN_QUERY)
            .expect("Error create regex with detect params pattern expression");

        for positional_param in rg.find_iter(stmt) {
            let pp: &str = positional_param.as_str();
            let pp_index = pp[1..] // param $1 -> get 1
                .parse::<usize>()
                .expect("Error parse mapped parameter to usized.")
                - 1;

            let element = params
                .get(pp_index)
                .expect("Error obtaining the element of the mapping against parameters.");
            ordered_params.push(fn_parser(element));
        }

        ordered_params
    }
}
