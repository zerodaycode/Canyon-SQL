use super::datasources::Auth;
use crate::canyon::Canyon;
use serde::Deserialize;
use std::error::Error;
use std::fmt::Display;

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
///
/// // Or defer the database type resolution until runtime:
/// let builder = QueryBuilder::new_for(table, columns, DatabaseType::Deferred)?;
/// let query = builder.build()?; // will resolve to the default DB type
/// ```
#[derive(Deserialize, Debug, Eq, PartialEq, Clone, Copy, Default)]
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

    /// A **placeholder variant** used when the database dialect
    /// cannot be determined at compile time.
    ///
    /// The [`Deferred`](Self::Deferred) variant allows you to construct a query
    /// (for example, through a procedural macro like `CanyonCrud`) before the
    /// actual database type is known — typically at compile-time code generation.
    ///
    /// When using this variant, the [`crate::query::querybuilder::QueryBuilder::build()`] method will automatically
    /// attempt to resolve the concrete database type from the active [`Canyon`]
    /// instance at runtime.
    ///
    /// # Use case
    /// This is particularly useful for macros and compile-time query generation,
    /// where it’s desirable to emit code that’s agnostic of the target database.
    /// The macro can safely emit `DatabaseType::Deferred` in the generated code,
    /// and Canyon will resolve it dynamically when executing queries.
    ///
    /// # Example
    /// ```rust,ignore
    /// let query = SelectQueryBuilder::new(
    ///     &table_metadata,
    ///     DatabaseType::Deferred
    /// )?
    /// .where_("id", Operator::Eq, &42)
    /// .build()?; // resolved dynamically to the active database type
    /// ```
    #[default]
    Deferred, // TODO: review if this is yet viable
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
