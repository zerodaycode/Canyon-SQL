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
pub static mut CANYON_MANAGED: Vec<String> = Vec::new();

///! Warning #[NOT WORKING NOR IMPLEMENTED]
pub static mut CREDENTIALS: Option<DatabaseCredentials> = None;

/// Provides a prodecural way of manipulate the internal Canyon dat
///! Warning #[IMPLEMENTED]
pub trait CanyonManager {
    /// Register into the CANYON_MANAGED namaspace data about a structure that should
    /// be completly managed by Canyon
    fn register_entity(entity_identifier: &'static str) {
        unsafe {CANYON_MANAGED.push(entity_identifier.to_string())};
    }

    /// Shows the data owned by the CANYON_MANAGED data structure
    ///! This should be only enabled in development stage
    fn print_managed_structures() {
        unsafe {println!("Managed data: {:?}", CANYON_MANAGED);}
    } 
}
