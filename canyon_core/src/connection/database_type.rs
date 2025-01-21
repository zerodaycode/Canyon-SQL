use serde::Deserialize;

use super::datasources::Auth;

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
            Auth::Postgres(_) => DatabaseType::PostgreSql,
            #[cfg(feature = "mssql")]
            Auth::SqlServer(_) => DatabaseType::SqlServer,
            #[cfg(feature = "mysql")]
            Auth::MySQL(_) => DatabaseType::MySQL,
        }
    }
}
