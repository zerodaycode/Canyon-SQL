//! Defines the Canyon-SQL custom connection error types

/// Raised when a [`crate::datasources::DatasourceConfig`] isn't found given a user input
#[derive(Debug, Clone)]
pub struct DatasourceNotFound<T: AsRef<str> + ?Sized + std::fmt::Debug + Default> {
    pub datasource_name: T
}
impl<T: AsRef<str> + std::fmt::Debug + Default> From<Option<T>> for DatasourceNotFound<T> {
    fn from(value: Option<T>) -> Self {
        DatasourceNotFound { datasource_name: value.unwrap_or_default() }
    }
}
impl<T: AsRef<str> + std::fmt::Debug + Default> std::fmt::Display for DatasourceNotFound<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Unable to found a datasource that matches: {:?}", self.datasource_name)
    }
}
impl<T: AsRef<str> + std::fmt::Debug + Default> std::error::Error for DatasourceNotFound<T> {}