pub mod postgresql_connector;
pub mod credentials;

use crate::credentials::DatasourceConfig;
pub use crate::credentials::DatabaseCredentials;
use lazy_static::lazy_static;


pub static mut DATASOURCES: Vec<DatasourceConfig<'static>> = Vec::new();
lazy_static! {
    pub static ref CREDENTIALS: DatabaseCredentials = DatabaseCredentials::new();
}