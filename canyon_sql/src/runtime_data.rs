use std::marker::PhantomData;

use crate::connector::DatabaseConnection;

/// Holds the data needed by Canyon when the host
/// application it's running.
/// 
/// Takes care about provide a namespace where retrieve the
/// databse credentials in only one place
/// 
/// Also takes care about track what data structures Canyon
/// should be managing

// pub static RUNTIME_DATA: RuntimeData<'static> = RuntimeData::new().await;
pub static CANYON_MANAGED: Vec<&'static str> = Vec::new();
// static DATABASE_CREDENTIALS: DatabaseConnection<'static> = DatabaseConnection::new();
struct RuntimeData<'a> {
    db_credentials: DatabaseConnection<'static>,
    phantom: PhantomData<&'a RuntimeData<'a>>
}

unsafe impl Send for RuntimeData<'_> {}
unsafe impl Sync for RuntimeData<'_> {}

impl<'a> RuntimeData<'a> {
    pub async fn new() -> RuntimeData<'a> {
        Self {
            db_credentials: DatabaseConnection::new().await.unwrap(),
            phantom: PhantomData,
        }
    }
}