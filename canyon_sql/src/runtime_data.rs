// use std::marker::PhantomData;

// use crate::connector::DatabaseConnection;

/// Holds the data needed by Canyon when the host
/// application it's running.
/// 
/// Takes care about provide a namespace where retrieve the
/// databse credentials in only one place
/// 
/// Also takes care about track what data structures Canyon
/// should be managing

// pub static RUNTIME_DATA: RuntimeData<'static> = RuntimeData::new().await;
static mut CANYON_MANAGED: Vec<&'static str> = Vec::new();

/// Provides a prodecural way of manipulate the internal Canyon data
pub trait CanyonManager {
    /// Register into the CANYON_MANAGED namaspace a structure that should
    /// be completly managed by Canyon
    fn register_entity(entity_identifier: &'static str) {
        unsafe {CANYON_MANAGED.push(entity_identifier)};
    }

    /// Shows the data owned by the CANYON_MANAGED data structure
    ///! This should be only enabled in development stage
    fn print_managed_structures() {
        unsafe {println!("Managed data: {:?}", CANYON_MANAGED);}
    } 
}
// static DATABASE_CREDENTIALS: DatabaseConnection<'static> = DatabaseConnection::new();
// #[warn(dead_code)]
// struct RuntimeData<'a> {
//     db_credentials: DatabaseConnection<'static>,
//     phantom: PhantomData<&'a RuntimeData<'a>>
// }



// unsafe impl Send for RuntimeData<'_> {}
// unsafe impl Sync for RuntimeData<'_> {}

// impl<'a> RuntimeData<'a> {
//     #[warn(dead_code)]
//     pub async fn new() -> RuntimeData<'a> {
//         Self {
//             db_credentials: DatabaseConnection::new().await.unwrap(),
//             phantom: PhantomData,
//         }
//     }
// }