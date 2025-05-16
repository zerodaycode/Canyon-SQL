use super::datasources::Auth;
use crate::canyon::Canyon;
use serde::Deserialize;
use std::error::Error;
use std::fmt::Display;

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

impl Display for DatabaseType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "{:?}", self)
    }
}

impl From<&Auth> for DatabaseType {
    fn from(value: &Auth) -> Self {
        value.get_db_type()
    }
}

/// The default implementation for [`DatabaseType`] returns the database type for the first
/// datasource configured
impl DatabaseType {
    pub fn default_type() -> Result<Self, Box<dyn Error + Send + Sync>> {
        Canyon::instance()?
            .get_default_db_type()
            .map_err(|err| Box::new(err) as Box<dyn Error + Send + Sync>)
    }
}
