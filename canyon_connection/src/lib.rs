pub extern crate async_std;
pub extern crate tiberius;
pub extern crate tokio;
pub extern crate tokio_postgres;

pub mod canyon_database_connector;
mod datasources;

use std::fs;

use crate::datasources::{CanyonSqlConfig, DatasourceConfig};
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
}
