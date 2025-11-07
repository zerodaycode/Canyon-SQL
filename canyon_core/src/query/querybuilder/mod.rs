pub mod contracts;
pub mod types;

pub use self::{contracts::*, types::*};

pub struct TableMetadata {
    pub schema: &'a str,
    pub name: &'a str,
}
