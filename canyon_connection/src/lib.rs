pub mod postgresql_connector;
mod credentials;

use crate::credentials::DatabaseCredentials;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref CREDENTIALS: DatabaseCredentials = DatabaseCredentials::new();
}