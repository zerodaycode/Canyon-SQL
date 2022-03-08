pub mod credentials;

use credentials::DatabaseCredentials;


/// Holds the data needed by Canyon when the host
/// application it's running.
/// 
/// Takes care about provide a namespace where retrieve the
/// databse credentials in only one place
/// 
/// Also takes care about track what data structures Canyon
/// should be managing
pub static mut CANYON_REGISTER: Vec<String> = Vec::new();

pub static mut CREDENTIALS: Option<DatabaseCredentials> = None;

/// Provides a prodecural way of manipulate the internal Canyon dat
///! Warning #[UNIMPLEMENTED]
pub trait CanyonManager {
    /// Register into the CANYON_REGISTER namaspace data about a structure that should
    /// be completly managed by Canyon
    fn register_entity(entity_identifier: &'static str) {
        unsafe {CANYON_REGISTER.push(entity_identifier.to_string())};
    }

    /// Shows the data owned by the CANYON_REGISTER data structure
    ///! This should be only enabled in development stage
    fn print_managed_structures() {
        unsafe {println!("Managed data: {:?}", CANYON_REGISTER);}
    } 
}
