use crate::connection::conn_errors::DatasourceNotFound;
use crate::connection::database_type::DatabaseType;
use crate::connection::datasources::{CanyonSqlConfig, DatasourceConfig, Datasources};
use crate::connection::{CANYON_INSTANCE, db_connector, get_canyon_tokio_runtime};
use db_connector::DatabaseConnection;
use std::collections::HashMap;
use std::sync::Arc;
use std::{error::Error, fs};
use tokio::sync::Mutex;

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
/// ```ignore
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
    default_connection: Option<SharedConnection>,
    default_db_type: Option<DatabaseType>,
}

impl Canyon {
    /// Returns the global singleton instance of `Canyon`.
    ///
    /// This function allows access to the singleton instance of the Canyon engine
    /// after it has been initialized through [`Canyon::init`]. It returns a shared,
    /// read-only reference to the internal `Canyon` state.
    ///
    /// # Errors
    ///
    /// Returns an error if the `Canyon` instance has not yet been initialized.
    /// In that case, the user must call [`Canyon::init`] before accessing the singleton.
    pub fn instance() -> Result<&'static Self, Box<dyn Error + Send + Sync>> {
        Ok(CANYON_INSTANCE.get().ok_or_else(|| {
            // TODO: just call Canyon::init()? Why should we raise this error?
            // I guess that there's no point in making it fail for the user to manually start Canyon when we can handle everything
            // internally
            Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Canyon not initialized. Call `Canyon::init()` first.",
            ))
        })?)
    }

    /// Initializes the global `Canyon` instance from a configuration file.
    ///
    /// Loads the `Datasources` configuration from the expected `canyon.toml` file (or another
    /// discoverable location), establishes one or more database connections, and sets up the default
    /// connection and database type.
    ///
    /// This function is idempotent: calling it multiple times will reuse the already-initialized instance.
    ///
    /// # Errors
    ///
    /// - If the configuration file is missing or malformed.
    /// - If deserialization into `CanyonSqlConfig` fails.
    /// - If any configured datasource fails to initialize.
    ///
    /// # Example
    ///
    /// ```ignore
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    ///     let canyon = Canyon::init().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn init() -> Result<&'static Self, Box<dyn Error + Send + Sync>> {
        if CANYON_INSTANCE.get().is_some() {
            return Canyon::instance(); // Already initialized, no need to do it again
        }

        let path = __impl::find_config_path()?;
        let config_content = fs::read_to_string(&path)?;
        let config: Datasources = toml::from_str::<CanyonSqlConfig>(&config_content)?.canyon_sql;

        let mut connections = HashMap::new();
        let mut default_connection = None;
        let mut default_db_type = None;

        for ds in config.datasources.iter() {
            __impl::process_new_conn_by_datasource(
                ds,
                &mut connections,
                &mut default_connection,
                &mut default_db_type,
            )
            .await?;
        }

        let canyon = Canyon {
            config,
            connections,
            default_connection,
            default_db_type,
        };

        get_canyon_tokio_runtime(); // Just ensuring that is initialized in manual-mode
        Ok(CANYON_INSTANCE.get_or_init(|| canyon))
    }

    /// Returns an immutable slice containing all configured datasources.
    ///
    /// This slice represents the datasources defined in your `canyon.toml` configuration.
    ///
    /// # Example
    ///
    /// ```
    /// use canyon_core::canyon::Canyon;
    /// for ds in Canyon::instance()?.datasources() {
    ///     println!("Datasource name: {}", ds.name);
    /// }
    /// ```
    #[inline(always)]
    pub fn datasources(&self) -> &[DatasourceConfig] {
        &self.config.datasources
    }

    // Retrieve a datasource by name or returns the first one declared in the configuration file
    // or added by the user via the builder interface as the default one (if exists at least one)
    pub fn find_datasource_by_name_or_default(
        &self,
        name: &str,
    ) -> Result<&DatasourceConfig, DatasourceNotFound> {
        if name.is_empty() {
            self.datasources()
                .first()
                .ok_or_else(|| DatasourceNotFound::from(None))
        } else {
            self.datasources()
                .iter()
                .find(|ds| ds.name == name)
                .ok_or_else(|| DatasourceNotFound::from(Some(name)))
        }
    }

    pub fn get_default_db_type(&self) -> Result<DatabaseType, DatasourceNotFound> {
        self.default_db_type
            .ok_or_else(|| DatasourceNotFound::from(None))
    }

    // Retrieve a read-only connection from the cache
    pub fn get_default_connection(&self) -> Result<SharedConnection, DatasourceNotFound> {
        self.default_connection
            .clone()
            .ok_or_else(|| DatasourceNotFound::from(None))
    }

    // Retrieve a read-only connection from the cache
    pub fn get_connection(&self, name: &str) -> Result<SharedConnection, DatasourceNotFound> {
        if name.is_empty() {
            return self.get_default_connection();
        }

        let conn = self
            .connections
            .get(name)
            .ok_or_else(|| DatasourceNotFound::from(Some(name)))?;

        Ok(conn.clone())
    }
}

mod __impl {
    use crate::canyon::SharedConnection;
    use crate::connection::database_type::DatabaseType;
    use crate::connection::datasources::DatasourceConfig;
    use crate::connection::db_connector::DatabaseConnection;
    use std::collections::HashMap;
    use std::error::Error;
    use std::path::PathBuf;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use walkdir::WalkDir;

    // Internal helper to locate the config file
    pub(crate) fn find_config_path() -> Result<PathBuf, std::io::Error> {
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

    pub(crate) async fn process_new_conn_by_datasource(
        ds: &DatasourceConfig,
        connections: &mut HashMap<&str, SharedConnection>,
        default: &mut Option<SharedConnection>,
        default_db_type: &mut Option<DatabaseType>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let conn = DatabaseConnection::new(ds).await?;
        let name: &'static str = Box::leak(ds.name.clone().into_boxed_str());

        if default_db_type.is_none() {
            *default_db_type = Some(conn.get_db_type());
        }

        let connection_sp = Arc::new(Mutex::new(conn));

        if default.is_none() {
            *default = Some(connection_sp.clone()); // Only cloning the smart pointer
        }

        connections.insert(name, connection_sp);

        Ok(())
    }
}
