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

use std::fmt::Debug;
use std::path::PathBuf;
use std::{error::Error, fs};

use crate::datasources::{CanyonSqlConfig, DatasourceConfig};
use conn_errors::DatasourceNotFound;
use db_connector::DatabaseConnection;
use indexmap::IndexMap;
use lazy_static::lazy_static;
use tokio::sync::{Mutex, MutexGuard};
use walkdir::WalkDir;

lazy_static! {
    pub static ref CANYON_TOKIO_RUNTIME: tokio::runtime::Runtime =
        tokio::runtime::Runtime::new()  // TODO Make the config with the builder
            .expect("Failed initializing the Canyon-SQL Tokio Runtime");

    static ref RAW_CONFIG_FILE: String = fs::read_to_string(find_canyon_config_file())
        .expect("Error opening or reading the Canyon configuration file");
    static ref CONFIG_FILE: CanyonSqlConfig = toml::from_str(RAW_CONFIG_FILE.as_str())
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
pub async fn get_database_connection_by_ds<
    'a,
    T: AsRef<str> + Copy + Debug + Default + Send + Sync + 'a,
>(
    datasource_name: Option<T>,
) -> Result<db_connector::DatabaseConnection, Box<dyn Error + Send + Sync>> {
    let ds = find_datasource_by_name_or_try_default(datasource_name)?;
    DatabaseConnection::new(ds).await
}

fn find_datasource_by_name_or_try_default<'a, T: AsRef<str> + Copy + Debug + Default>(
    datasource_name: Option<T>,
) -> Result<&'a DatasourceConfig, DatasourceNotFound<T>> {
    datasource_name
        .map_or_else(
            || DATASOURCES.first(),
            |ds_name| DATASOURCES.iter().find(|ds| ds.name.eq(ds_name.as_ref())),
        )
        .ok_or_else(|| DatasourceNotFound::from(datasource_name))
}

// TODO: create a new one that just receives a str and tries to find the ds config on the vec, so we can make a facade over the one above

// TODO: get_cached_database_connection
// the idea behind this is that we can have a #cfg feature that offers the end user to let Canyon to automagically manage the connections
// to the db servers, instead of being the default behaviour
pub fn get_database_connection<'a>(
    datasource_name: &str,
    guarded_cache: &'a mut MutexGuard<IndexMap<&str, DatabaseConnection>>,
) -> &'a mut DatabaseConnection {
    if datasource_name.is_empty() {
        guarded_cache
            .get_mut(
                DATASOURCES
                    .first()
                    .expect("We didn't found any valid datasource configuration. Check your `canyon.toml` file")
                    .name
                    .as_str()
            ).unwrap_or_else(|| panic!("No default datasource found. Check your `canyon.toml` file"))
    } else {
        guarded_cache.get_mut(datasource_name)
            .unwrap_or_else(||
                panic!("Canyon couldn't find a datasource in the pool with the argument provided: {datasource_name}")
            )
    }
}

pub fn get_database_config<'a>(
    datasource_name: &str,
    datasources_config: &'a [DatasourceConfig],
) -> &'a DatasourceConfig {
    if datasource_name.is_empty() {
        datasources_config
            .first()
            .unwrap_or_else(|| panic!("Not exist datasource"))
    } else {
        datasources_config
            .iter()
            .find(|dc| dc.name == datasource_name)
            .unwrap_or_else(|| panic!("Not found datasource expected {datasource_name}"))
    }
}
