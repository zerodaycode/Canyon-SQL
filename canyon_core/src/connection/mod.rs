#[cfg(feature = "postgres")]
pub extern crate tokio_postgres;

#[cfg(feature = "mssql")]
pub extern crate async_std;
#[cfg(feature = "mssql")]
pub extern crate tiberius;

#[cfg(feature = "mysql")]
pub extern crate mysql_async;

pub extern crate futures;
pub extern crate lazy_static;
pub extern crate tokio;
pub extern crate tokio_util;

pub mod conn_errors;
pub mod database_type;
pub mod datasources;
pub mod db_clients;
pub mod db_connector;

use std::path::PathBuf;
use std::{error::Error, fs};

use conn_errors::DatasourceNotFound;
use datasources::{CanyonSqlConfig, DatasourceConfig};
use db_connector::DatabaseConnection;
use indexmap::IndexMap;
use lazy_static::lazy_static;
use tokio::sync::Mutex;
use walkdir::WalkDir;

lazy_static! {
    pub static ref CANYON_TOKIO_RUNTIME: tokio::runtime::Runtime =
        tokio::runtime::Runtime::new()  // TODO Make the config with the builder
            .expect("Failed initializing the Canyon-SQL Tokio Runtime");

    static ref CONFIG_FILE: CanyonSqlConfig = toml::from_str(&fs::read_to_string(find_canyon_config_file())
        .expect("Error opening or reading the Canyon configuration file"))
        .expect("Error generating the configuration for Canyon-SQL");

    pub static ref DATASOURCES: Vec<DatasourceConfig> =
        CONFIG_FILE.canyon_sql.datasources.clone();

    pub static ref CACHED_DATABASE_CONN: Mutex<IndexMap<&'static str, DatabaseConnection>> =
        Mutex::new(IndexMap::new());
}

fn find_canyon_config_file() -> PathBuf {
    for e in WalkDir::new(".")
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let filename = e.file_name().to_str().unwrap(); // TODO: remove the .unwrap(). Use
                                                        // lowercase to allow Canyon.toml
        if e.metadata().unwrap().is_file()
            && filename.starts_with("canyon")
            && filename.ends_with(".toml")
        {
            return e.path().to_path_buf();
        }
    }

    panic!() // TODO: get rid out of this panic and return Err instead
}

/// Convenient free function to initialize a kind of connection pool based on the datasources present defined
/// in the configuration file.
///
/// This avoids Canyon to create a new connection to the database on every query, potentially avoiding bottlenecks
/// coming from the instantiation of that new conn every time.
///
/// Note: We noticed with the integration tests that the [`tokio_postgres`] crate (PostgreSQL) is able to work in an async environment
/// with a new connection per query without no problem, but the [`tiberius`] crate (MSSQL) suffers a lot when it has continuous
/// statements with multiple queries, like and insert followed by a find by id to check if the insert query has done its
/// job done.
pub async fn init_connections_cache() {
    for datasource in DATASOURCES.iter() {
        CACHED_DATABASE_CONN.lock().await.insert(
            &datasource.name,
            DatabaseConnection::new(datasource)
                .await
                .unwrap_or_else(|_| {
                    panic!(
                        "Error pooling a new connection for the datasource: {:?}",
                        datasource.name
                    )
                }),
        );
    }
}

// TODO: idea. Should we leak the datasources config pull to the user, so we can be more flexible and let the
// user code determine whenever you can find a valid datasource via a concrete type instead of an string?

// TODO: doc (main way for the user to obtain a db connection given a datasource identifier)
pub async fn get_database_connection_by_ds(
    datasource_name: Option<&str>,
) -> Result<DatabaseConnection, Box<dyn Error + Send + Sync>> {
    let ds = find_datasource_by_name_or_try_default(datasource_name)?;
    DatabaseConnection::new(ds).await
}

pub fn find_datasource_by_name_or_try_default(
    datasource_name: Option<&str>, // TODO: with the new inputs, we don't want anymore this as Option
) -> Result<&DatasourceConfig, DatasourceNotFound> {
    let datasource_name = datasource_name.filter(|&ds_name| !ds_name.is_empty());

    datasource_name
        .map_or_else(
            || DATASOURCES.first(),
            |ds_name| DATASOURCES.iter().find(|ds| ds.name.eq(ds_name)),
        )
        .ok_or_else(|| DatasourceNotFound::from(datasource_name))
}