pub mod credentials;

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

// pub static mut CREDENTIALS: Option<DatabaseCredentials> = None;


