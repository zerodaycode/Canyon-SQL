//! The datasources module of Canyon-SQL.
//!
//! This module defines the configuration and authentication mechanisms for database datasources.
//! It includes support for multiple database backends and provides utilities for managing
//! datasource properties.

use serde::{Deserialize, Deserializer};

use super::database_type::DatabaseType;
use crate::error::{CanyonResult, ConfigurationError};

const DEFAULT_POOL_MIN_SIZE: u32 = 2;
const DEFAULT_POOL_MAX_SIZE: u32 = 10;

#[derive(Deserialize, Debug, Clone)]
/// Root structure deserialized from `canyon.toml`.
pub struct CanyonSqlConfig {
    /// Canyon datasource configuration.
    pub canyon_sql: Datasources,
}

#[derive(Debug, Clone)]
/// Datasources enabled by the crate's selected backend features.
pub struct Datasources {
    /// Configured datasource entries, in declaration order.
    pub datasources: Vec<DatasourceConfig>,
}

impl<'de> Deserialize<'de> for Datasources {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawDatasources::deserialize(deserializer)?;

        let datasources = raw
            .datasources
            .into_iter()
            .filter_map(DatasourceConfig::from_raw)
            .collect();

        Ok(Self { datasources })
    }
}

#[derive(Deserialize)]
struct RawDatasources {
    datasources: Vec<RawDatasourceConfig>,
}

#[derive(Deserialize)]
struct RawDatasourceConfig {
    name: String,
    auth: RawAuth,
    properties: DatasourceProperties,
}

#[derive(Deserialize)]
enum RawAuth {
    #[serde(alias = "PostgresSQL", alias = "postgresql", alias = "postgres")]
    Postgres(RawPostgresAuth),

    #[serde(alias = "SqlServer", alias = "sqlserver", alias = "mssql")]
    SqlServer(RawSqlServerAuth),

    #[serde(alias = "MYSQL", alias = "mysql", alias = "MySQL")]
    MySQL(RawMySQLAuth),
}

#[cfg(feature = "postgres")]
type RawPostgresAuth = PostgresAuth;

#[cfg(not(feature = "postgres"))]
type RawPostgresAuth = serde::de::IgnoredAny;

#[cfg(feature = "mssql")]
type RawSqlServerAuth = SqlServerAuth;

#[cfg(not(feature = "mssql"))]
type RawSqlServerAuth = serde::de::IgnoredAny;

#[cfg(feature = "mysql")]
type RawMySQLAuth = MySQLAuth;

#[cfg(not(feature = "mysql"))]
type RawMySQLAuth = serde::de::IgnoredAny;

#[derive(Debug, Clone)]
/// Complete configuration for one named datasource.
pub struct DatasourceConfig {
    /// Name used to select this datasource at runtime.
    pub name: String,
    /// Backend-specific authentication settings.
    pub auth: Auth,
    /// Backend-independent connection properties.
    pub properties: DatasourceProperties,
}

impl DatasourceConfig {
    fn from_raw(raw: RawDatasourceConfig) -> Option<Self> {
        let RawDatasourceConfig {
            name,
            auth,
            properties,
        } = raw;

        let auth = match auth {
            #[cfg(feature = "postgres")]
            RawAuth::Postgres(auth) => Auth::Postgres(auth),

            #[cfg(not(feature = "postgres"))]
            RawAuth::Postgres(_) => return None,

            #[cfg(feature = "mssql")]
            RawAuth::SqlServer(auth) => Auth::SqlServer(auth),

            #[cfg(not(feature = "mssql"))]
            RawAuth::SqlServer(_) => return None,

            #[cfg(feature = "mysql")]
            RawAuth::MySQL(auth) => Auth::MySQL(auth),

            #[cfg(not(feature = "mysql"))]
            RawAuth::MySQL(_) => return None,
        };

        Some(Self {
            name,
            auth,
            properties,
        })
    }

    /// Returns the database backend configured for this datasource.
    pub fn get_db_type(&self) -> DatabaseType {
        self.auth.get_db_type()
    }

    /// Reports whether experimental migrations are enabled for this datasource.
    pub fn has_migrations_enabled(&self) -> bool {
        self.properties
            .migrations
            .is_some_and(|migrations| migrations.has_migrations_enabled())
    }

    /// Returns the configured port or the standard port for the selected backend.
    pub fn get_port_or_default_by_db(&self) -> u16 {
        self.properties
            .port
            .unwrap_or_else(|| match self.get_db_type() {
                #[cfg(feature = "postgres")]
                DatabaseType::PostgreSql => 5432,

                #[cfg(feature = "mssql")]
                DatabaseType::SqlServer => 1433,

                #[cfg(feature = "mysql")]
                DatabaseType::MySQL => 3306,
            })
    }

    /// Returns the validated connection-pool settings for this datasource.
    pub fn pool_config(&self) -> CanyonResult<PoolConfig> {
        let pool = self.properties.pool;

        if !pool.is_valid() {
            return Err(ConfigurationError::InvalidPoolConfiguration {
                backend: self.get_db_type(),
                min_size: pool.min_size,
                max_size: pool.max_size,
            }
            .into());
        }

        Ok(pool)
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Authentication settings for a supported database backend.
pub enum Auth {
    #[cfg(feature = "postgres")]
    /// PostgreSQL authentication.
    Postgres(PostgresAuth),

    #[cfg(feature = "mssql")]
    /// SQL Server authentication.
    SqlServer(SqlServerAuth),

    #[cfg(feature = "mysql")]
    /// MySQL authentication.
    MySQL(MySQLAuth),
}

impl Auth {
    /// Returns the backend associated with this authentication variant.
    pub fn get_db_type(&self) -> DatabaseType {
        match self {
            #[cfg(feature = "postgres")]
            Self::Postgres(_) => DatabaseType::PostgreSql,

            #[cfg(feature = "mssql")]
            Self::SqlServer(_) => DatabaseType::SqlServer,

            #[cfg(feature = "mysql")]
            Self::MySQL(_) => DatabaseType::MySQL,
        }
    }
}

#[cfg(feature = "postgres")]
#[derive(Deserialize, Debug, Clone, PartialEq)]
/// Supported PostgreSQL authentication methods.
pub enum PostgresAuth {
    #[serde(alias = "Basic", alias = "basic")]
    /// Username and password authentication.
    Basic { username: String, password: String },
}

#[cfg(feature = "mssql")]
#[derive(Deserialize, Debug, Clone, PartialEq)]
/// Supported SQL Server authentication methods.
pub enum SqlServerAuth {
    #[serde(alias = "Basic", alias = "basic")]
    /// Username and password authentication.
    Basic { username: String, password: String },
}

/// TLS policy for connections managed by the SQL Server connector.
#[cfg(feature = "mssql")]
#[derive(Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SqlServerTlsMode {
    /// Require encryption and validate the server certificate against the system trust store.
    #[default]
    #[serde(alias = "Required")]
    Required,
    /// Require encryption but accept the server certificate without validating it.
    #[serde(alias = "TrustServerCertificate", alias = "trust-server-certificate")]
    TrustServerCertificate,
    /// Disable encryption. Intended only for explicitly configured local test environments.
    #[serde(alias = "Disabled")]
    Disabled,
}

#[cfg(feature = "mysql")]
#[derive(Deserialize, Debug, Clone, PartialEq)]
/// Supported MySQL authentication methods.
pub enum MySQLAuth {
    #[serde(alias = "Basic", alias = "basic")]
    /// Username and password authentication.
    Basic { username: String, password: String },
}

#[derive(Deserialize, Debug, Clone)]
/// Connection properties shared by all database backends.
pub struct DatasourceProperties {
    /// Database server hostname or IP address.
    pub host: String,
    /// Server port. When omitted, Canyon uses the backend's standard port.
    pub port: Option<u16>,
    /// Database name used by this datasource.
    pub db_name: String,
    /// Experimental migration setting for this datasource.
    pub migrations: Option<Migrations>,
    /// Connection-pool limits. Missing settings use [`PoolConfig::default`].
    #[serde(default)]
    pub pool: PoolConfig,
    /// SQL Server TLS policy. Missing values default to [`SqlServerTlsMode::Required`].
    #[cfg(feature = "mssql")]
    #[serde(default)]
    pub mssql_tls: SqlServerTlsMode,
}

/// Connection-pool limits shared by every supported database backend.
#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoolConfig {
    /// Minimum number of connections retained by the pool.
    #[serde(default = "default_pool_min_size")]
    pub min_size: u32,
    /// Maximum number of connections that the pool may open.
    #[serde(default = "default_pool_max_size")]
    pub max_size: u32,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_size: DEFAULT_POOL_MIN_SIZE,
            max_size: DEFAULT_POOL_MAX_SIZE,
        }
    }
}

impl PoolConfig {
    const fn is_valid(self) -> bool {
        self.max_size > 0 && self.min_size <= self.max_size
    }
}

const fn default_pool_min_size() -> u32 {
    DEFAULT_POOL_MIN_SIZE
}

const fn default_pool_max_size() -> u32 {
    DEFAULT_POOL_MAX_SIZE
}

#[cfg(test)]
mod pool_tests {
    use super::PoolConfig;
    #[cfg(feature = "postgres")]
    use super::{Auth, DatasourceConfig, DatasourceProperties, PostgresAuth};
    #[cfg(feature = "postgres")]
    use crate::error::{CanyonError, ConfigurationError};

    #[test]
    fn pool_config_deserializes_defaults_and_overrides() {
        assert_eq!(
            toml::from_str::<PoolConfig>("").unwrap(),
            PoolConfig::default()
        );
        assert_eq!(
            toml::from_str::<PoolConfig>("min_size = 4\nmax_size = 24").unwrap(),
            PoolConfig {
                min_size: 4,
                max_size: 24,
            }
        );
    }

    #[test]
    fn pool_config_rejects_zero_maximum_and_inverted_bounds() {
        assert!(
            !PoolConfig {
                min_size: 0,
                max_size: 0,
            }
            .is_valid()
        );
        assert!(
            !PoolConfig {
                min_size: 11,
                max_size: 10,
            }
            .is_valid()
        );
        assert!(
            PoolConfig {
                min_size: 0,
                max_size: 1,
            }
            .is_valid()
        );
    }

    #[cfg(feature = "postgres")]
    #[test]
    fn datasource_reports_invalid_pool_bounds_as_configuration_errors() {
        let datasource = DatasourceConfig {
            name: "invalid_pool".into(),
            auth: Auth::Postgres(PostgresAuth::Basic {
                username: "postgres".into(),
                password: "postgres".into(),
            }),
            properties: DatasourceProperties {
                host: "localhost".into(),
                port: None,
                db_name: "postgres".into(),
                migrations: None,
                pool: PoolConfig {
                    min_size: 12,
                    max_size: 4,
                },
                #[cfg(feature = "mssql")]
                mssql_tls: Default::default(),
            },
        };

        assert!(matches!(
            datasource.pool_config(),
            Err(CanyonError::Configuration(
                ConfigurationError::InvalidPoolConfiguration {
                    min_size: 12,
                    max_size: 4,
                    ..
                }
            ))
        ));
    }
}

/// Represents the enabled or disabled migrations for a whole datasource.
#[derive(Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Migrations {
    /// Enables the experimental migrations subsystem.
    #[serde(alias = "Enabled", alias = "enabled")]
    Enabled,

    /// Disables migrations for the datasource.
    #[serde(alias = "Disabled", alias = "disabled")]
    Disabled,
}

impl Migrations {
    /// Returns `true` when migrations are enabled.
    pub fn has_migrations_enabled(&self) -> bool {
        matches!(self, Self::Enabled)
    }
}

#[cfg(all(test, feature = "mssql"))]
mod tests {
    use super::{CanyonSqlConfig, PoolConfig, SqlServerTlsMode};

    fn parse_mssql_tls(value: Option<&str>) -> Result<SqlServerTlsMode, toml::de::Error> {
        let tls = value
            .map(|value| format!("mssql_tls = '{value}'"))
            .unwrap_or_default();
        let config = format!(
            r#"
                [canyon_sql]

                [[canyon_sql.datasources]]
                name = 'sqlserver'

                [canyon_sql.datasources.auth]
                sqlserver = {{ basic = {{ username = 'sa', password = 'password' }} }}

                [canyon_sql.datasources.properties]
                host = 'localhost'
                db_name = 'master'
                {tls}
            "#
        );

        let parsed = toml::from_str::<CanyonSqlConfig>(&config)?;
        Ok(parsed.canyon_sql.datasources[0].properties.mssql_tls)
    }

    #[test]
    fn mssql_tls_defaults_to_required() {
        assert_eq!(parse_mssql_tls(None).unwrap(), SqlServerTlsMode::Required);
    }

    #[test]
    fn mssql_tls_deserializes_each_explicit_mode() {
        assert_eq!(
            parse_mssql_tls(Some("required")).unwrap(),
            SqlServerTlsMode::Required
        );
        assert_eq!(
            parse_mssql_tls(Some("trust_server_certificate")).unwrap(),
            SqlServerTlsMode::TrustServerCertificate
        );
        assert_eq!(
            parse_mssql_tls(Some("disabled")).unwrap(),
            SqlServerTlsMode::Disabled
        );
    }

    #[test]
    fn mssql_tls_rejects_unknown_modes() {
        assert!(parse_mssql_tls(Some("opportunistic")).is_err());
    }

    #[test]
    fn pool_config_uses_backwards_compatible_defaults() {
        assert_eq!(PoolConfig::default().min_size, 2);
        assert_eq!(PoolConfig::default().max_size, 10);
    }
}
