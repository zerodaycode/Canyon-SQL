use super::datasources::Auth;
use crate::canyon::Canyon;
use serde::Deserialize;
use std::{error::Error, fmt::Display};

/// Represents the supported database backends in **Canyon-SQL**.
///
/// This enum abstracts over the specific database dialects supported by Canyon,
/// allowing queries and builders to adapt automatically to the correct SQL syntax
/// and placeholder conventions (`$1`, `?`, `@P1`, etc.) according to the active
/// [`DatabaseType`].
///
/// The variant used at runtime is determined either:
/// - Explicitly, when passed to a [`crate::query::querybuilder::QueryBuilder`] constructor, or
/// - Implicitly, from the first configured data source via
///   [`Canyon::get_default_db_type()`].
///
/// # Example
/// ```rust,ignore
/// use canyon_core::connection::database_type::DatabaseType;
///
/// // Create a query builder explicitly targeting PostgreSQL:
/// let builder = QueryBuilder::new_for(table, columns, DatabaseType::PostgreSql)?;
/// ```
#[derive(Deserialize, Debug, Eq, PartialEq, Clone, Copy)]
pub enum DatabaseType {
    /// The Postgres database backend.
    #[cfg(feature = "postgres")]
    #[serde(alias = "postgres", alias = "postgresql")]
    PostgreSql,

    /// The Microsoft SQL Server backend.
    #[cfg(feature = "mssql")]
    #[serde(alias = "sqlserver", alias = "mssql")]
    SqlServer,

    /// The MySQL or MariaDB backend.
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
