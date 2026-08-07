//! The datasources module of Canyon-SQL.
//!
//! This module defines the configuration and authentication mechanisms for database datasources.
//! It includes support for multiple database backends and provides utilities for managing
//! datasource properties.

use serde::{Deserialize, Deserializer};

use super::database_type::DatabaseType;

#[derive(Deserialize, Debug, Clone)]
pub struct CanyonSqlConfig {
    pub canyon_sql: Datasources,
}

#[derive(Debug, Clone)]
pub struct Datasources {
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
pub struct DatasourceConfig {
    pub name: String,
    pub auth: Auth,
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

    pub fn get_db_type(&self) -> DatabaseType {
        self.auth.get_db_type()
    }

    pub fn has_migrations_enabled(&self) -> bool {
        self.properties
            .migrations
            .is_some_and(|migrations| migrations.has_migrations_enabled())
    }

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
}

#[derive(Debug, Clone, PartialEq)]
pub enum Auth {
    #[cfg(feature = "postgres")]
    Postgres(PostgresAuth),

    #[cfg(feature = "mssql")]
    SqlServer(SqlServerAuth),

    #[cfg(feature = "mysql")]
    MySQL(MySQLAuth),
}

impl Auth {
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
pub enum PostgresAuth {
    #[serde(alias = "Basic", alias = "basic")]
    Basic { username: String, password: String },
}

#[cfg(feature = "mssql")]
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub enum SqlServerAuth {
    #[serde(alias = "Basic", alias = "basic")]
    Basic { username: String, password: String },
}

#[cfg(feature = "mysql")]
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub enum MySQLAuth {
    #[serde(alias = "Basic", alias = "basic")]
    Basic { username: String, password: String },
}

#[derive(Deserialize, Debug, Clone)]
pub struct DatasourceProperties {
    pub host: String,
    pub port: Option<u16>,
    pub db_name: String,
    pub migrations: Option<Migrations>,
}

/// Represents the enabled or disabled migrations for a whole datasource.
#[derive(Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Migrations {
    #[serde(alias = "Enabled", alias = "enabled")]
    Enabled,

    #[serde(alias = "Disabled", alias = "disabled")]
    Disabled,
}

impl Migrations {
    pub fn has_migrations_enabled(&self) -> bool {
        matches!(self, Self::Enabled)
    }
}
