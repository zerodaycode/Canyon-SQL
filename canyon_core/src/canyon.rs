// ...existing code...

use crate::connection::conn_errors::DatasourceNotFound;
use crate::connection::datasources::{CanyonSqlConfig, DatasourceConfig, Datasources};
use crate::connection::{db_connector, get_canyon_tokio_runtime, CANYON_INSTANCE};
use db_connector::DatabaseConnection;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::{error::Error, fs};
use tokio::sync::Mutex;
use walkdir::WalkDir;

pub type SharedConnection = Arc<Mutex<DatabaseConnection>>;

/// The `Canyon` struct provides the main entry point for interacting with the Canyon-SQL context.
///
/// This struct is responsible for managing database connections, configuration, and datasources.
/// It acts as a singleton, ensuring that only one instance of the Canyon context exists throughout
/// the application lifecycle. The `Canyon` struct provides methods for initializing the context,
/// accessing datasources, and retrieving database connections.
///
/// # Features
/// - Singleton access to the Canyon context.
/// - Automatic discovery and loading of configuration files.
/// - Management of multiple database connections.
/// - Support for retrieving connections by name or default.
///
/// # Examples
/// ```no_run
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Initialize the Canyon context
///     let canyon = Canyon::init().await?;
///
///     // Access datasources
///     let datasources = canyon.datasources();
///     for ds in datasources {
///         println!("Datasource: {}", ds.name);
///     }
///
///     // Retrieve a connection by name
///     let connection = canyon.get_connection("MyDatasource").await?;
///     // Use the connection...
///
///     Ok(())
/// }
/// ```
///
/// # Methods
/// - `init`: Initializes the Canyon context by loading configuration and setting up connections.
/// - `instance`: Provides singleton access to the Canyon context.
/// - `datasources`: Returns a list of configured datasources.
/// - `find_datasource_by_name_or_default`: Finds a datasource by name or returns the default.
/// - `get_connection`: Retrieves a read-only connection from the cache.
/// - `get_mut_connection`: Retrieves a mutable connection from the cache.
pub struct Canyon {
    config: Datasources,
    connections: HashMap<&'static str, SharedConnection>,
    default: Option<SharedConnection>,
}

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
