//! Defines the Canyon-SQL custom connection error types

/// Raised when a [`crate::connection::datasources::DatasourceConfig`] isn't found given a user input
#[derive(Debug, Clone)]
pub struct DatasourceNotFound {
    pub datasource_name: String,
}
impl From<Option<&str>> for DatasourceNotFound {
    fn from(value: Option<&str>) -> Self {
        DatasourceNotFound {
            datasource_name: value
                .map(String::from)
                .unwrap_or_else(|| String::from("No datasource name was provided"))
        }
    }
}
impl std::fmt::Display for DatasourceNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Unable to found a datasource that matches: {:?}",
            self.datasource_name
        )
    }
}
impl std::error::Error for DatasourceNotFound {}
