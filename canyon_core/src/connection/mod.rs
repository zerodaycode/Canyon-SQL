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

// TODO's: DatabaseConnection and DataSource can implement default, so there's no need to use str and &str
// as defaults anymore, since the can load as the default the first one defined in the config file, or have more
// complex workflows that are deferred to initialization time

// TODO: Crud Operations should be split into two different derives, splitting the automagic from the _with ones

// Use OnceLock for the Tokio runtime
static CANYON_TOKIO_RUNTIME: OnceLock<Runtime> = OnceLock::new();

// Function to get the runtime (lazy initialization)
pub fn get_canyon_tokio_runtime() -> &'static Runtime {
    CANYON_TOKIO_RUNTIME
        .get_or_init(|| Runtime::new().expect("Failed initializing the Canyon-SQL Tokio Runtime"))
}

static CONFIG_FILE_PATH: OnceLock<PathBuf> = OnceLock::new();
static CONFIG: OnceLock<CanyonSqlConfig> = OnceLock::new();
static DATASOURCES: OnceLock<Vec<DatasourceConfig>> = OnceLock::new();

// Safer connection wrapper: each conn has its own async mutex
pub type SharedConnection = Arc<Mutex<DatabaseConnection>>;

static CACHED_DATABASE_CONN: OnceLock<HashMap<&'static str, SharedConnection>> = OnceLock::new();
static DEFAULT_CONNECTION: OnceLock<SharedConnection> = OnceLock::new();

/// Attempts to locate a canyon config file.
/// Returns Ok(None) if not found, or Ok(Some(PathBuf)) if found.
pub fn find_canyon_config_file() -> Result<Option<PathBuf>, std::io::Error> {
    let result = WalkDir::new(".")
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
        });

    Ok(result)
}

/// Initializes shared config state by loading the config file if found.
///
/// - Used by macro/automatic path to enforce config presence.
/// - Can be used optionally in manual mode.
///
/// Returns:
/// - `Ok(Some(()))` => config loaded
/// - `Ok(None)` => config not found
/// - `Err` => parsing or IO error
pub fn try_init_config() -> Result<Option<()>, Box<dyn Error + Send + Sync + 'static>> {
    let Some(path) = find_canyon_config_file()? else {
        return Ok(None); // Not an error!
    };

    let content = fs::read_to_string(&path)?;
    let config: CanyonSqlConfig = toml::from_str(&content)?;

    CONFIG_FILE_PATH.set(path).ok();
    CONFIG.set(config).ok();

    let datasources = CONFIG
        .get()
        .map(|cfg| cfg.canyon_sql.datasources.clone())
        .unwrap_or_default();

    DATASOURCES.set(datasources).ok();

    Ok(Some(()))
}

/// Required in macro mode only: forcefully load or panic
pub fn force_init_config() {
    match try_init_config() {
        Ok(Some(())) => {}
        Ok(None) => panic!("Canyon config file not found but required in macro mode."),
        Err(e) => panic!("Failed to load Canyon config: {}", e),
    }
}

/// Public accessor for datasources, safe even if uninitialized
pub fn get_datasources() -> &'static [DatasourceConfig] {
    DATASOURCES.get().map(Vec::as_slice).unwrap_or_default()
}

pub async fn init_connections_cache() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    try_init_config()?;

    let datasources = get_datasources();
    if datasources.is_empty() {
        return Err("No datasources found for connection pool".into());
    }

    let mut cache = HashMap::new();
    for ds in datasources {
        let conn = DatabaseConnection::new(ds).await?;
        let name: &'static str = Box::leak(ds.name.clone().into_boxed_str());
        let conn_arc = Arc::new(Mutex::new(conn));

        if cache.is_empty() {
            DEFAULT_CONNECTION.set(conn_arc.clone()).ok(); // Store direct ref
        }

        cache.insert(name, conn_arc);
    }

    CACHED_DATABASE_CONN.set(cache).ok();
    Ok(())
}

/// Borrow a connection for read-only use (if immutable suffices)
pub async fn get_cached_connection(
    name: &str,
) -> Result<tokio::sync::MutexGuard<'_, DatabaseConnection>, DatasourceNotFound> {
    if name.is_empty() {
        let default = DEFAULT_CONNECTION
            .get()
            .ok_or_else(|| DatasourceNotFound::from(None))?;
        return Ok(default.lock().await);
    }

    let cache = CACHED_DATABASE_CONN
        .get()
        .expect("Connection cache not initialized");

    let conn = cache
        .get(name)
        .ok_or_else(|| DatasourceNotFound::from(Some(name)))?;

    Ok(conn.lock().await)
}

/// Mutable access — same as above (just aliasing for clarity)
pub async fn get_mut_cached_connection(
    name: &str,
) -> Result<tokio::sync::MutexGuard<'_, DatabaseConnection>, DatasourceNotFound> {
    get_cached_connection(name).await
}
pub fn find_datasource_by_name_or_try_default(
    name: &str,
) -> Result<&DatasourceConfig, DatasourceNotFound> {
    let configs = DATASOURCES
        .get()
        .expect("Datasources cache not initialized");

    if name.is_empty() {
        return configs
            .first()
            .ok_or_else(|| DatasourceNotFound::from(None));
    }

    configs
        .iter()
        .find(|ds| ds.name == name)
        .ok_or_else(|| DatasourceNotFound::from(Some(name)))
}
