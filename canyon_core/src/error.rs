//! Typed errors exposed by Canyon-SQL.

use crate::connection::database_type::DatabaseType;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;

/// The result type returned by Canyon-SQL public APIs.
pub type CanyonResult<T> = Result<T, CanyonError>;

/// Top-level error for Canyon-SQL operations.
#[derive(Debug)]
#[non_exhaustive]
pub enum CanyonError {
    Configuration(ConfigurationError),
    Connection(ConnectionError),
    Query(QueryError),
    QueryBuilder(QueryBuilderError),
    Mapping(MappingError),
}

impl Display for CanyonError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Configuration(error) => Display::fmt(error, formatter),
            Self::Connection(error) => Display::fmt(error, formatter),
            Self::Query(error) => Display::fmt(error, formatter),
            Self::QueryBuilder(error) => Display::fmt(error, formatter),
            Self::Mapping(error) => Display::fmt(error, formatter),
        }
    }
}

impl Error for CanyonError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Configuration(error) => Some(error),
            Self::Connection(error) => Some(error),
            Self::Query(error) => Some(error),
            Self::QueryBuilder(error) => Some(error),
            Self::Mapping(error) => Some(error),
        }
    }
}

macro_rules! impl_canyon_error_from {
    ($variant:ident, $error:ty) => {
        impl From<$error> for CanyonError {
            fn from(error: $error) -> Self {
                Self::$variant(error)
            }
        }
    };
}

impl_canyon_error_from!(Configuration, ConfigurationError);
impl_canyon_error_from!(Connection, ConnectionError);
impl_canyon_error_from!(Query, QueryError);
impl_canyon_error_from!(QueryBuilder, QueryBuilderError);
impl_canyon_error_from!(Mapping, MappingError);

/// Errors produced while locating, reading or deserializing Canyon configuration.
#[derive(Debug)]
#[non_exhaustive]
pub enum ConfigurationError {
    NotFound,
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    Deserialize(toml::de::Error),
    InvalidAuthentication {
        backend: DatabaseType,
    },
}

impl Display for ConfigurationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => formatter.write_str("Canyon configuration file not found"),
            Self::Read { path, .. } => {
                write!(formatter, "failed to read Canyon configuration at {path:?}")
            }
            Self::Deserialize(_) => {
                formatter.write_str("failed to deserialize Canyon configuration")
            }
            Self::InvalidAuthentication { backend } => {
                write!(
                    formatter,
                    "invalid authentication configuration for {backend}"
                )
            }
        }
    }
}

impl Error for ConfigurationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Read { source, .. } => Some(source),
            Self::Deserialize(source) => Some(source),
            Self::NotFound | Self::InvalidAuthentication { .. } => None,
        }
    }
}

/// Errors produced while creating, selecting or locking database connections.
#[derive(Debug)]
#[non_exhaustive]
pub enum ConnectionError {
    DatasourceNotFound {
        name: Option<String>,
    },
    CanyonNotInitialized,
    ConnectionBusy,
    InvalidPoolConfiguration {
        backend: DatabaseType,
    },
    #[cfg(feature = "postgres")]
    Postgres(Box<tokio_postgres::Error>),
    #[cfg(feature = "postgres")]
    PostgresPool(Box<bb8::RunError<tokio_postgres::Error>>),
    #[cfg(feature = "mysql")]
    MySql(Box<mysql_async::Error>),
    #[cfg(feature = "mssql")]
    SqlServer(Box<tiberius::error::Error>),
    #[cfg(feature = "mssql")]
    SqlServerManager(Box<bb8_tiberius::Error>),
    #[cfg(feature = "mssql")]
    SqlServerPool(Box<bb8::RunError<bb8_tiberius::Error>>),
}

impl ConnectionError {
    #[cfg(feature = "postgres")]
    pub fn postgres(source: tokio_postgres::Error) -> Self {
        Self::Postgres(Box::new(source))
    }

    #[cfg(feature = "postgres")]
    pub fn postgres_pool(source: bb8::RunError<tokio_postgres::Error>) -> Self {
        Self::PostgresPool(Box::new(source))
    }

    #[cfg(feature = "mysql")]
    pub fn mysql(source: mysql_async::Error) -> Self {
        Self::MySql(Box::new(source))
    }

    #[cfg(feature = "mssql")]
    pub fn sql_server(source: tiberius::error::Error) -> Self {
        Self::SqlServer(Box::new(source))
    }

    #[cfg(feature = "mssql")]
    pub fn sql_server_manager(source: bb8_tiberius::Error) -> Self {
        Self::SqlServerManager(Box::new(source))
    }

    #[cfg(feature = "mssql")]
    pub fn sql_server_pool(source: bb8::RunError<bb8_tiberius::Error>) -> Self {
        Self::SqlServerPool(Box::new(source))
    }
}

impl Display for ConnectionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::DatasourceNotFound { name: Some(name) } => {
                write!(formatter, "datasource `{name}` was not found")
            }
            Self::DatasourceNotFound { name: None } => {
                formatter.write_str("no default datasource is configured")
            }
            Self::CanyonNotInitialized => {
                formatter.write_str("Canyon is not initialized; call `Canyon::init()` first")
            }
            Self::ConnectionBusy => formatter.write_str("database connection is busy"),
            Self::InvalidPoolConfiguration { backend } => {
                write!(
                    formatter,
                    "invalid connection pool configuration for {backend}"
                )
            }
            #[cfg(feature = "postgres")]
            Self::Postgres(_) => formatter.write_str("PostgreSQL connection failed"),
            #[cfg(feature = "postgres")]
            Self::PostgresPool(_) => formatter.write_str("PostgreSQL connection pool failed"),
            #[cfg(feature = "mysql")]
            Self::MySql(_) => formatter.write_str("MySQL connection failed"),
            #[cfg(feature = "mssql")]
            Self::SqlServer(_) => formatter.write_str("SQL Server connection failed"),
            #[cfg(feature = "mssql")]
            Self::SqlServerManager(_) => {
                formatter.write_str("SQL Server connection manager failed")
            }
            #[cfg(feature = "mssql")]
            Self::SqlServerPool(_) => formatter.write_str("SQL Server connection pool failed"),
        }
    }
}

impl Error for ConnectionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            #[cfg(feature = "postgres")]
            Self::Postgres(source) => Some(source.as_ref()),
            #[cfg(feature = "postgres")]
            Self::PostgresPool(source) => Some(source.as_ref()),
            #[cfg(feature = "mysql")]
            Self::MySql(source) => Some(source.as_ref()),
            #[cfg(feature = "mssql")]
            Self::SqlServer(source) => Some(source.as_ref()),
            #[cfg(feature = "mssql")]
            Self::SqlServerManager(source) => Some(source.as_ref()),
            #[cfg(feature = "mssql")]
            Self::SqlServerPool(source) => Some(source.as_ref()),
            Self::DatasourceNotFound { .. }
            | Self::CanyonNotInitialized
            | Self::ConnectionBusy
            | Self::InvalidPoolConfiguration { .. } => None,
        }
    }
}

/// Errors produced while executing a query through a database driver.
#[derive(Debug)]
#[non_exhaustive]
pub enum QueryError {
    #[cfg(feature = "postgres")]
    Postgres(Box<tokio_postgres::Error>),
    #[cfg(feature = "mysql")]
    MySql(Box<mysql_async::Error>),
    #[cfg(feature = "mysql")]
    MySqlValue(Box<mysql_common::value::convert::FromValueError>),
    #[cfg(feature = "mssql")]
    SqlServer(Box<tiberius::error::Error>),
    NoRows,
    NoColumns,
    UnexpectedNull,
}

impl QueryError {
    #[cfg(feature = "postgres")]
    pub fn postgres(source: tokio_postgres::Error) -> Self {
        Self::Postgres(Box::new(source))
    }

    #[cfg(feature = "mysql")]
    pub fn mysql(source: mysql_async::Error) -> Self {
        Self::MySql(Box::new(source))
    }

    #[cfg(feature = "mysql")]
    pub fn mysql_value(source: mysql_common::value::convert::FromValueError) -> Self {
        Self::MySqlValue(Box::new(source))
    }

    #[cfg(feature = "mssql")]
    pub fn sql_server(source: tiberius::error::Error) -> Self {
        Self::SqlServer(Box::new(source))
    }
}

impl Display for QueryError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "postgres")]
            Self::Postgres(_) => formatter.write_str("PostgreSQL query failed"),
            #[cfg(feature = "mysql")]
            Self::MySql(_) => formatter.write_str("MySQL query failed"),
            #[cfg(feature = "mysql")]
            Self::MySqlValue(_) => formatter.write_str("failed to convert a MySQL query value"),
            #[cfg(feature = "mssql")]
            Self::SqlServer(_) => formatter.write_str("SQL Server query failed"),
            Self::NoRows => formatter.write_str("the query returned no rows"),
            Self::NoColumns => formatter.write_str("the query returned a row without columns"),
            Self::UnexpectedNull => formatter.write_str("the query returned an unexpected NULL"),
        }
    }
}

impl Error for QueryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            #[cfg(feature = "postgres")]
            Self::Postgres(source) => Some(source.as_ref()),
            #[cfg(feature = "mysql")]
            Self::MySql(source) => Some(source.as_ref()),
            #[cfg(feature = "mysql")]
            Self::MySqlValue(source) => Some(source.as_ref()),
            #[cfg(feature = "mssql")]
            Self::SqlServer(source) => Some(source.as_ref()),
            Self::NoRows | Self::NoColumns | Self::UnexpectedNull => None,
        }
    }
}

/// Errors produced while validating or building a query.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum QueryBuilderError {
    EmptyInClause { table: String, column: String },
    EmptySetClause,
    SetClauseAlreadyPresent,
    InvalidClauseOrder { clause: String },
    MissingPrimaryKey,
    MissingPrimaryKeyValue,
    MissingForeignKeyValue { column: String },
    UnsupportedOperation { operation: String },
    Rendering(std::fmt::Error),
}

impl Display for QueryBuilderError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInClause { table, column } => write!(
                formatter,
                "an IN clause for `{table}`.`{column}` cannot contain an empty value list"
            ),
            Self::EmptySetClause => formatter.write_str("an UPDATE query requires a SET clause"),
            Self::SetClauseAlreadyPresent => {
                formatter.write_str("the UPDATE query already contains a SET clause")
            }
            Self::InvalidClauseOrder { clause } => {
                write!(formatter, "the `{clause}` clause is in an invalid position")
            }
            Self::MissingPrimaryKey => {
                formatter.write_str("the entity does not define a primary key")
            }
            Self::MissingPrimaryKeyValue => {
                formatter.write_str("the entity does not contain a primary-key value")
            }
            Self::MissingForeignKeyValue { column } => {
                write!(
                    formatter,
                    "the entity does not contain foreign-key column `{column}`"
                )
            }
            Self::UnsupportedOperation { operation } => {
                write!(
                    formatter,
                    "the `{operation}` operation is not supported for this entity"
                )
            }
            Self::Rendering(_) => formatter.write_str("failed to render the SQL query"),
        }
    }
}

impl Error for QueryBuilderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Rendering(source) => Some(source),
            Self::EmptyInClause { .. }
            | Self::EmptySetClause
            | Self::SetClauseAlreadyPresent
            | Self::InvalidClauseOrder { .. }
            | Self::MissingPrimaryKey
            | Self::MissingPrimaryKeyValue
            | Self::MissingForeignKeyValue { .. }
            | Self::UnsupportedOperation { .. } => None,
        }
    }
}

impl From<std::fmt::Error> for QueryBuilderError {
    fn from(error: std::fmt::Error) -> Self {
        Self::Rendering(error)
    }
}

/// Errors produced while converting a driver row into an application type.
#[derive(Debug)]
#[non_exhaustive]
pub enum MappingError {
    ColumnNotFound {
        entity: String,
        column: String,
        backend: DatabaseType,
    },
    UnexpectedNull {
        entity: String,
        column: String,
        backend: DatabaseType,
    },
    BackendMismatch {
        expected: DatabaseType,
    },
    UnsupportedRowType,
    #[cfg(feature = "postgres")]
    Postgres {
        entity: String,
        column: String,
        source: Box<tokio_postgres::Error>,
    },
    #[cfg(feature = "mysql")]
    MySql {
        entity: String,
        column: String,
        source: Box<mysql_common::value::convert::FromValueError>,
    },
    #[cfg(feature = "mssql")]
    SqlServer {
        entity: String,
        column: String,
        source: Box<tiberius::error::Error>,
    },
    Custom {
        message: String,
    },
}

impl MappingError {
    pub fn column_not_found(
        entity: impl Into<String>,
        column: impl Into<String>,
        backend: DatabaseType,
    ) -> Self {
        Self::ColumnNotFound {
            entity: entity.into(),
            column: column.into(),
            backend,
        }
    }

    pub fn unexpected_null(
        entity: impl Into<String>,
        column: impl Into<String>,
        backend: DatabaseType,
    ) -> Self {
        Self::UnexpectedNull {
            entity: entity.into(),
            column: column.into(),
            backend,
        }
    }

    pub fn custom(message: impl Into<String>) -> Self {
        Self::Custom {
            message: message.into(),
        }
    }

    #[cfg(feature = "postgres")]
    pub fn postgres(
        entity: impl Into<String>,
        column: impl Into<String>,
        source: tokio_postgres::Error,
    ) -> Self {
        Self::Postgres {
            entity: entity.into(),
            column: column.into(),
            source: Box::new(source),
        }
    }

    #[cfg(feature = "mysql")]
    pub fn mysql(
        entity: impl Into<String>,
        column: impl Into<String>,
        source: mysql_common::value::convert::FromValueError,
    ) -> Self {
        Self::MySql {
            entity: entity.into(),
            column: column.into(),
            source: Box::new(source),
        }
    }

    #[cfg(feature = "mssql")]
    pub fn sql_server(
        entity: impl Into<String>,
        column: impl Into<String>,
        source: tiberius::error::Error,
    ) -> Self {
        Self::SqlServer {
            entity: entity.into(),
            column: column.into(),
            source: Box::new(source),
        }
    }
}

impl Display for MappingError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ColumnNotFound {
                entity,
                column,
                backend,
            } => write!(
                formatter,
                "column `{column}` for entity `{entity}` was not found in the {backend} row"
            ),
            Self::UnexpectedNull {
                entity,
                column,
                backend,
            } => write!(
                formatter,
                "column `{column}` for entity `{entity}` unexpectedly contained NULL in the {backend} row"
            ),
            Self::BackendMismatch { expected } => {
                write!(
                    formatter,
                    "row does not belong to the expected {expected} backend"
                )
            }
            Self::UnsupportedRowType => formatter.write_str("unsupported database row type"),
            #[cfg(feature = "postgres")]
            Self::Postgres { entity, column, .. } => write!(
                formatter,
                "failed to map PostgreSQL column `{column}` for entity `{entity}`"
            ),
            #[cfg(feature = "mysql")]
            Self::MySql { entity, column, .. } => write!(
                formatter,
                "failed to map MySQL column `{column}` for entity `{entity}`"
            ),
            #[cfg(feature = "mssql")]
            Self::SqlServer { entity, column, .. } => write!(
                formatter,
                "failed to map SQL Server column `{column}` for entity `{entity}`"
            ),
            Self::Custom { message } => formatter.write_str(message),
        }
    }
}

impl Error for MappingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            #[cfg(feature = "postgres")]
            Self::Postgres { source, .. } => Some(source.as_ref()),
            #[cfg(feature = "mysql")]
            Self::MySql { source, .. } => Some(source.as_ref()),
            #[cfg(feature = "mssql")]
            Self::SqlServer { source, .. } => Some(source.as_ref()),
            Self::ColumnNotFound { .. }
            | Self::UnexpectedNull { .. }
            | Self::BackendMismatch { .. }
            | Self::UnsupportedRowType
            | Self::Custom { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CanyonError, MappingError};
    use std::error::Error;

    #[test]
    fn canyon_error_preserves_its_typed_mapping_error() {
        let error = CanyonError::from(MappingError::custom("could not map row"));

        assert!(matches!(
            error,
            CanyonError::Mapping(MappingError::Custom { .. })
        ));
        assert_eq!(error.to_string(), "could not map row");
        assert_eq!(
            error.source().map(ToString::to_string).as_deref(),
            Some("could not map row")
        );
    }
}
