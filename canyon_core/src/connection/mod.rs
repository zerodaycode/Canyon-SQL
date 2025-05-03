#[cfg(feature = "postgres")]
pub extern crate tokio_postgres;

#[cfg(feature = "mssql")]
pub extern crate async_std;
#[cfg(feature = "mssql")]
pub extern crate tiberius;

#[cfg(feature = "mysql")]
pub extern crate mysql_async;

pub extern crate futures;
pub extern crate tokio;
pub extern crate tokio_util;

pub mod conn_errors;
pub mod database_type;
pub mod datasources;
pub mod db_clients;
pub mod db_connector;

use crate::connection::datasources::Datasources;
use conn_errors::DatasourceNotFound;
use datasources::{CanyonSqlConfig, DatasourceConfig};
use db_connector::DatabaseConnection;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::{error::Error, fs};
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use walkdir::WalkDir;

//
// // TODO's: DatabaseConnection and DataSource can implement default, so there's no need to use str and &str
// // as defaults anymore, since the can load as the default the first one defined in the config file, or have more
// // complex workflows that are deferred to initialization time
//
// // TODO: Crud Operations should be split into two different derives, splitting the automagic from the _with ones

// TODO: move Canyon struct to core

// Use OnceLock for the Tokio runtime
static CANYON_TOKIO_RUNTIME: OnceLock<Runtime> = OnceLock::new();

// Function to get the runtime (lazy initialization)
pub fn get_canyon_tokio_runtime() -> &'static Runtime {
    CANYON_TOKIO_RUNTIME
        .get_or_init(|| Runtime::new().expect("Failed initializing the Canyon-SQL Tokio Runtime"))
}

pub type SharedConnection = Arc<Mutex<DatabaseConnection>>;

pub struct Canyon {
    config: Datasources,
    connections: HashMap<&'static str, SharedConnection>,
    default: Option<SharedConnection>,
}

static CANYON_INSTANCE: OnceLock<Canyon> = OnceLock::new();

impl Canyon {
    // Singleton access
    pub fn instance() -> Result<&'static Self, Box<dyn Error + Send + Sync>> {
        Ok(CANYON_INSTANCE.get().ok_or_else(|| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Canyon not initialized. Call `Canyon::init()` first.",
            ))
        })?)
    }

    // Initializes Canyon instance
    pub async fn init() -> Result<&'static Self, Box<dyn Error + Send + Sync>> {
        if CANYON_INSTANCE.get().is_some() {
            return Canyon::instance(); // Already initialized, no need to do it again
        }

        let path = Canyon::find_config_path()?;
        let config_content = fs::read_to_string(&path)?;
        let config: Datasources = toml::from_str::<CanyonSqlConfig>(&config_content)?.canyon_sql;

        let mut connections = HashMap::new();
        let mut default = None;

        for ds in config.datasources.iter() {
            let conn = DatabaseConnection::new(ds).await?;
            let name: &'static str = Box::leak(ds.name.clone().into_boxed_str());
            let conn = Arc::new(Mutex::new(conn));

            if default.is_none() {
                default = Some(conn.clone()); // Only cloning the smart pointer
            }

            connections.insert(name, conn);
        }

        let canyon = Canyon {
            config,
            connections,
            default,
        };

        get_canyon_tokio_runtime(); // Just ensuring that is initialized in manual-mode
        Ok(CANYON_INSTANCE.get_or_init(|| canyon))
    }

    // Internal helper to locate the config file
    fn find_config_path() -> Result<PathBuf, std::io::Error> {
        WalkDir::new(".")
            .max_depth(2)
            .into_iter()
            .filter_map(Result::ok)
            .find_map(|e| {
                let filename = e.file_name().to_string_lossy().to_lowercase();
                if e.metadata().ok()?.is_file()
                    && filename.starts_with("canyon")
                    && filename.ends_with(".toml")
                {
                    Some(e.path().to_path_buf())
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "No Canyon config found")
            })
    }

    // Public accessor for datasources
    pub fn datasources(&self) -> &[DatasourceConfig] {
        &self.config.datasources
    }

    // Retrieve a datasource by name or default to the first
    pub fn find_datasource_by_name_or_default(
        &self,
        name: &str,
    ) -> Result<&DatasourceConfig, DatasourceNotFound> {
        if name.is_empty() {
            self.config
                .datasources
                .first()
                .ok_or_else(|| DatasourceNotFound::from(None))
        } else {
            self.config
                .datasources
                .iter()
                .find(|ds| ds.name == name)
                .ok_or_else(|| DatasourceNotFound::from(Some(name)))
        }
    }

    // Retrieve a read-only connection from the cache
    pub async fn get_connection(
        &self,
        name: &str,
    ) -> Result<tokio::sync::MutexGuard<'_, DatabaseConnection>, DatasourceNotFound> {
        if name.is_empty() {
            return Ok(self
                .default
                .as_ref()
                .ok_or_else(|| DatasourceNotFound::from(None))?
                .lock()
                .await);
        }

        let conn = self
            .connections
            .get(name)
            .ok_or_else(|| DatasourceNotFound::from(Some(name)))?;

        Ok(conn.lock().await)
    }

    // Retrieve a mutable connection from the cache
    pub async fn get_mut_connection(
        &self,
        name: &str,
    ) -> Result<tokio::sync::MutexGuard<'_, DatabaseConnection>, DatasourceNotFound> {
        self.get_connection(name).await
    }
}
