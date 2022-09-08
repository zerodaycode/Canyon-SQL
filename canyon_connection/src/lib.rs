pub mod postgresql_connector;
pub mod credentials;

use std::fs;

use crate::credentials::{DatasourceConfig, CanyonSqlConfig};
pub use crate::credentials::DatabaseCredentials;
use lazy_static::lazy_static;

const CONFIG_FILE_IDENTIFIER: &'static str = "canyon.toml";


lazy_static! {
    static ref RAW_CONFIG_FILE: String = fs::read_to_string(CONFIG_FILE_IDENTIFIER)
        .expect("Error opening or reading the Canyon configuration file");
    static ref CONFIG_FILE: CanyonSqlConfig<'static> = toml::from_str(RAW_CONFIG_FILE.as_str())
        .expect("Error generating the configuration for Canyon-SQL");

    pub static ref CREDENTIALS: DatabaseCredentials = DatabaseCredentials::new();
    pub static ref DATASOURCES: Vec<DatasourceConfig<'static>> = CONFIG_FILE.canyon_sql.datasources.clone();
    pub static ref DEFAULT_DATASOURCE: DatasourceConfig<'static> = CONFIG_FILE.canyon_sql.datasources.clone()[0];
}