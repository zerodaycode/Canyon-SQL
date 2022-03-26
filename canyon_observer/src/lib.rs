pub mod credentials;
pub mod handler;

extern crate canyon_crud;

use canyon_manager::manager::entity::CanyonEntity;
use handler::CanyonRegisterEntity;

// use credentials::DatabaseCredentials;

/// Holds the data needed by Canyon when the host
/// application it's running.
/// 
/// Takes care about provide a namespace where retrieve the
/// database credentials in only one place
/// 
/// Also takes care about track what data structures Canyon
/// should be managing
pub static mut CANYON_REGISTER: Vec<String> = Vec::new();
// pub static mut CANYON_REGISTER_ENTITIES: Vec<CanyonEntity> = Vec::new();
pub static mut CANYON_REGISTER_ENTITIES: Vec<CanyonRegisterEntity> = Vec::new();
// pub static REGISTER: *const Vec<CanyonEntity> = unsafe { &CANYON_REGISTER_ENTITIES as *const Vec<CanyonEntity> };
// pub static mut CREDENTIALS: Option<DatabaseCredentials> = None;