use crate::register_types::CanyonRegisterEntity;
use std::sync::Mutex;

pub mod entity;
pub mod entity_fields;
pub mod field_annotation;
pub mod manager_builder;
pub mod register_types;

pub static CANYON_REGISTER_ENTITIES: Mutex<Vec<CanyonRegisterEntity<'static>>> =
    Mutex::new(Vec::new());
