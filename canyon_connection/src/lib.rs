pub mod postgresql_connector;
mod credentials;

use crate::credentials::{DatabaseCredentials, Datasource};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref CREDENTIALS: DatabaseCredentials = DatabaseCredentials::new();
    pub static ref DATASOURCES: Vec<&'static dyn Datasource> = Vec::new();
}