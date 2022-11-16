pub extern crate async_std;
pub extern crate tiberius;
pub extern crate tokio;
pub extern crate tokio_util;
pub extern crate tokio_postgres;

pub mod canyon_database_connector;
mod datasources;

use std::{fs, collections::HashMap};

use crate::datasources::{CanyonSqlConfig, DatasourceConfig};
use canyon_database_connector::DatabaseConnection;
use datasources::DatasourceProperties;
use std::ops::DerefMut;
use lazy_static::lazy_static;

const CONFIG_FILE_IDENTIFIER: &str = "canyon.toml";

lazy_static! {
    static ref RAW_CONFIG_FILE: String = fs::read_to_string(CONFIG_FILE_IDENTIFIER)
        .expect("Error opening or reading the Canyon configuration file");
    static ref CONFIG_FILE: CanyonSqlConfig<'static> = toml::from_str(RAW_CONFIG_FILE.as_str())
        .expect("Error generating the configuration for Canyon-SQL");
    pub static ref DATASOURCES: Vec<DatasourceConfig<'static>> =
        CONFIG_FILE.canyon_sql.datasources.clone();
    pub static ref DEFAULT_DATASOURCE: DatasourceConfig<'static> =
        CONFIG_FILE.canyon_sql.datasources.clone()[0];

    pub static ref CACHED_DATABASE_CONN: HashMap<&'static str, &'static mut DatabaseConnection> =
        init_datasources();

    pub static ref CACHED_DATABASE_CONN_VEC: Vec<&'static mut DatabaseConnection> =
        init_pool();
}

fn init_pool() -> Vec<&'static mut DatabaseConnection> {
    let mut v = Vec::new();

    tokio::runtime::Runtime::new().unwrap().block_on(async {
        for datasource in DATASOURCES.iter() {
            v.push(
                Box::leak(
                    Box::new(DatabaseConnection::new(&datasource.properties)
                        .await
                        .expect(&format!("Error pooling a new connection for the datasource: {:?}", datasource.name))
                    )
                )
            );
        }
    });

    v
}

/// Convenient free function to initialize a kind of connection pool based on the datasources present defined
/// in the configuration file.
/// 
/// This avoids Canyon to create a new connection to the database on every query, potentially avoiding bottlenecks
/// derivated from the instanciation of that new conn every time.
/// 
/// Note: We noticed with the integration tests that the [`tokio_postgres`] crate (PostgreSQL) is able to work in an async environment
/// with a new connection per query without no problem, but the [`tiberius`] crate (MSSQL) sufferes a lot when it has continuous
/// statements with multiple queries, like and insert followed by a find by id to check if the insert query has done its
/// job done.
fn init_datasources() -> HashMap<&'static str, &'static mut DatabaseConnection> {
    let mut pool: HashMap<&'static str, &'static mut DatabaseConnection> = HashMap::new();
    
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        for datasource in DATASOURCES.iter() {
            pool.insert(
                datasource.name,
                Box::leak(
                    Box::new(DatabaseConnection::new(&datasource.properties)
                        .await
                        .expect(&format!("Error pooling a new connection for the datasource: {:?}", datasource.name))
                    )
                )
            );
        }
    });

    pool
}