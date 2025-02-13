use serde::Deserialize;
use crate::connection::DEFAULT_DATASOURCE;
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
        value.get_db_type()
    }
}

/// The default implementation for [`DatabaseType`] returns the database type for the first
/// datasource configured
impl Default for DatabaseType {
    fn default() -> Self {
        DEFAULT_DATASOURCE.get_db_type()
    }
}
