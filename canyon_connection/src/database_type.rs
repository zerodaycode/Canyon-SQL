use serde::Deserialize;

use crate::datasources::Auth;

/// Holds the current supported databases by Canyon-SQL
#[derive(Deserialize, Debug, Eq, PartialEq, Clone, Copy)]
pub enum DatabaseType {
    #[cfg(feature = "postgres")]
    #[serde(alias = "postgres", alias = "postgresql")]
    PostgreSql,
    #[cfg(feature = "mssql")]
    #[serde(alias = "sqlserver", alias = "mssql")]
    SqlServer,
    #[cfg(feature = "mysql")]
    #[serde(alias = "mysql")]
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