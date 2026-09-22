use crate::connection::database_type::DatabaseType;
use crate::connection::datasources::{CanyonSqlConfig, DatasourceConfig, Datasources};
use crate::connection::{CANYON_INSTANCE, db_connector, get_canyon_tokio_runtime};
use crate::error::{CanyonResult, ConfigurationError, ConnectionError};
use db_connector::DatabaseConnector;
use std::collections::HashMap;
use std::fs;

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
/// async fn main() -> canyon_core::error::CanyonResult<()> {
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
    connections: HashMap<&'static str, DatabaseConnector>,
    default_connection: Option<DatabaseConnector>,
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
    pub fn instance() -> CanyonResult<&'static Self> {
        CANYON_INSTANCE
            .get()
            .ok_or_else(|| ConnectionError::CanyonNotInitialized.into())
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
    /// async fn main() -> canyon_core::error::CanyonResult<()> {
    ///     let canyon = Canyon::init().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn init() -> CanyonResult<&'static Self> {
        if CANYON_INSTANCE.get().is_some() {
            return Canyon::instance(); // Already initialized, no need to do it again
        }

        let path = __impl::find_config_path()?;
        let config_content =
            fs::read_to_string(&path).map_err(|source| ConfigurationError::Read {
                path: path.clone(),
                source,
            })?;
        let config: Datasources = toml::from_str::<CanyonSqlConfig>(&config_content)
            .map_err(ConfigurationError::Deserialize)?
            .canyon_sql;

        let mut connections: HashMap<&str, DatabaseConnector> = HashMap::new();
        let mut default_connection: Option<DatabaseConnector> = None;
        let mut default_db_type: Option<DatabaseType> = None;

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

    #[inline(always)]
    pub fn datasources(&self) -> &[DatasourceConfig] {
        &self.config.datasources
    }

    // Retrieve a datasource by name or returns the first one declared in the configuration file
    // or added by the user via the builder interface as the default one (if exists at least one)
    pub fn find_datasource_by_name_or_default(
        &self,
        name: &str,
    ) -> CanyonResult<&DatasourceConfig> {
        if name.is_empty() {
            self.datasources()
                .first()
                .ok_or_else(|| ConnectionError::DatasourceNotFound { name: None }.into())
        } else {
            self.datasources()
                .iter()
                .find(|ds| ds.name == name)
                .ok_or_else(|| {
                    ConnectionError::DatasourceNotFound {
                        name: Some(name.to_owned()),
                    }
                    .into()
                })
        }
    }

    pub fn get_default_db_type(&self) -> CanyonResult<DatabaseType> {
        self.default_db_type
            .ok_or_else(|| ConnectionError::DatasourceNotFound { name: None }.into())
    }

    // Retrieves a connector to the configured connection as the default connection by the user
    // (the first defined in the configuration file)
    pub fn get_default_connection(&self) -> CanyonResult<&DatabaseConnector> {
        self.default_connection
            .as_ref()
            .ok_or_else(|| ConnectionError::DatasourceNotFound { name: None }.into())
    }

    // Retrieve a read-only connection from the cache
    pub fn get_connection(&self, name: &str) -> CanyonResult<&DatabaseConnector> {
        if name.is_empty() {
            return self.get_default_connection();
        }

        let conn =
            self.connections
                .get(name)
                .ok_or_else(|| ConnectionError::DatasourceNotFound {
                    name: Some(name.to_owned()),
                })?;

        Ok(conn)
    }
}

mod __impl {
    use crate::connection::database_type::DatabaseType;
    use crate::connection::datasources::DatasourceConfig;
    use crate::connection::db_connector::DatabaseConnector;
    use crate::error::{CanyonResult, ConfigurationError};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use walkdir::WalkDir;

    // Internal helper to locate the config file
    pub(crate) fn find_config_path() -> CanyonResult<PathBuf> {
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
            .ok_or_else(|| ConfigurationError::NotFound.into())
    }

    pub(crate) async fn process_new_conn_by_datasource(
        ds: &DatasourceConfig,
        connections: &mut HashMap<&str, DatabaseConnector>,
        default: &mut Option<DatabaseConnector>,
        default_db_type: &mut Option<DatabaseType>,
    ) -> CanyonResult<()> {
        if default.is_none() {
            let cloned_ds_for_default = ds.clone();
            *default = Some(DatabaseConnector::new(&cloned_ds_for_default).await?); // Only cloning the smart pointer
        }
        let conn = DatabaseConnector::new(ds).await?;
        let name: &'static str = Box::leak(ds.name.clone().into_boxed_str());

        if default_db_type.is_none() {
            *default_db_type = Some(conn.get_db_type());
        }

        let connection_sp = conn;
        connections.insert(name, connection_sp);

        Ok(())
    }
}
