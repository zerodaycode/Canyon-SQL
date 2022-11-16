pub extern crate async_std;
pub extern crate tiberius;
pub extern crate tokio;
pub extern crate tokio_util;
pub extern crate tokio_postgres;

pub mod canyon_database_connector;
mod datasources;

use std::{fs, collections::HashMap, hash::Hash, thread};

use crate::datasources::{CanyonSqlConfig, DatasourceConfig};
use canyon_database_connector::DatabaseConnection;
use lazy_static::lazy_static;
use tokio::sync::Mutex;

const CONFIG_FILE_IDENTIFIER: &str = "canyon.toml";

lazy_static! {
    pub static ref CANYON_TOKIO_RUNTIME: tokio::runtime::Runtime = 
        tokio::runtime::Runtime::new()
            .expect("Failed initializing the Canyon-SQL Tokio Runtime");

    static ref RAW_CONFIG_FILE: String = fs::read_to_string(CONFIG_FILE_IDENTIFIER)
        .expect("Error opening or reading the Canyon configuration file");
    static ref CONFIG_FILE: CanyonSqlConfig<'static> = toml::from_str(RAW_CONFIG_FILE.as_str())
        .expect("Error generating the configuration for Canyon-SQL");
    pub static ref DATASOURCES: Vec<DatasourceConfig<'static>> =
        CONFIG_FILE.canyon_sql.datasources.clone();
    pub static ref DEFAULT_DATASOURCE: DatasourceConfig<'static> =
        CONFIG_FILE.canyon_sql.datasources.clone()[0];

    static ref IGNITER: () = init_datasources();
    pub static ref CACHED_DATABASE_CONN: Mutex<HashMap<&'static str, &'static mut DatabaseConnection>> =
        Mutex::new(HashMap::new());
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
pub fn init_datasources() 
    // -> HashMap<&'static str, &'static mut DatabaseConnection> 
{   // tokio::runtime::Handle::current().block_on(async {
    println!("Datasources available: {}", DATASOURCES.len());
    CANYON_TOKIO_RUNTIME.block_on(async {
        let mut pool: HashMap<&'static str, &'static mut DatabaseConnection> = HashMap::new();
        println!("Datasources available: {}", DATASOURCES.len());
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
    
    *CACHED_DATABASE_CONN.blocking_lock() = pool;
});
println!("Datasources available: {}", DATASOURCES.len());
    
    // pool
}