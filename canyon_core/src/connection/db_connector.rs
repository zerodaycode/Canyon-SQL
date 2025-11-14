#[cfg(feature = "mssql")]
use crate::connection::clients::mssql::SqlServerConnector;
#[cfg(feature = "mysql")]
use crate::connection::clients::mysql::MySQLConnector;
#[cfg(feature = "postgres")]
use crate::connection::clients::postgresql::PostgresConnector;

use crate::connection::database_type::DatabaseType;
use crate::connection::datasources::DatasourceConfig;
use crate::mapper::RowMapper;
use crate::query::parameters::QueryParameter;
use crate::rows::{CanyonRows, FromSqlOwnedValue};
use std::error::Error;

/// The Canyon database connection handler. When the client's program
/// starts, Canyon gets the information about the desired datasources,
/// process them and generates a pool of connections for
/// every datasource defined.
pub enum DatabaseConnector {
    #[cfg(feature = "postgres")]
    Postgres(PostgresConnector),
    #[cfg(feature = "mssql")]
    SqlServer(SqlServerConnector),
    #[cfg(feature = "mysql")]
    MySQL(MySQLConnector),
}

unsafe impl Send for DatabaseConnector {}
unsafe impl Sync for DatabaseConnector {}

crate::impl_db_connection_for_db_connector!(DatabaseConnector);
crate::impl_db_connection_for_db_connector!(&DatabaseConnector);
crate::impl_db_connection_for_db_connector!(&mut DatabaseConnector);

impl DatabaseConnector {
    pub async fn new(datasource: &DatasourceConfig) -> Result<Self, Box<dyn Error + Send + Sync>> {
        // Add connection pooling at the client level for better performance
        match datasource.get_db_type() {
            #[cfg(feature = "postgres")]
            DatabaseType::PostgreSql => {
                Ok(Self::Postgres(PostgresConnector::new(datasource).await?))
            }

            #[cfg(feature = "mssql")]
            DatabaseType::SqlServer => {
                Ok(Self::SqlServer(SqlServerConnector::new(datasource).await?))
            }

            #[cfg(feature = "mysql")]
            DatabaseType::MySQL => Ok(Self::MySQL(MySQLConnector::new(datasource).await?)),

            DatabaseType::Deferred => panic!("Deferred connection"),
        }
    }

    pub fn get_db_type(&self) -> DatabaseType {
        match self {
            #[cfg(feature = "postgres")]
            DatabaseConnector::Postgres(_) => DatabaseType::PostgreSql,
            #[cfg(feature = "mssql")]
            DatabaseConnector::SqlServer(_) => DatabaseType::SqlServer,
            #[cfg(feature = "mysql")]
            DatabaseConnector::MySQL(_) => DatabaseType::MySQL,
        }
    }
}
