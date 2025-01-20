//! Defines the Canyon-SQL custom connection error types

/// Raised when a [`crate::datasources::DatasourceConfig`] isn't found given a user input
#[derive(Debug, Clone)]
pub struct DatasourceNotFound<'a> {
    pub datasource_name: &'a str,
}
impl<'a> From<Option<&'a str>> for DatasourceNotFound<'a> {
    fn from(value: Option<&'a str>) -> Self {
        DatasourceNotFound {
            datasource_name: value.unwrap_or_default(), // TODO: not default
        }
    }
}
impl<'a> std::fmt::Display for DatasourceNotFound<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Unable to found a datasource that matches: {:?}",
            self.datasource_name
        )
    }
}
impl<'a> std::error::Error for DatasourceNotFound<'a> {}
