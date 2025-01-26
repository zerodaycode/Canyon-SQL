use crate::{
    connection::{
        datasources::DatasourceConfig, db_connector::DatabaseConnection,
        get_database_connection_by_ds,
    },
    query_parameters::QueryParameter,
    rows::CanyonRows,
};
use std::{fmt::Display, future::Future};
use std::error::Error;
use crate::connection::database_type::DatabaseType;
use crate::connection::find_datasource_by_name_or_try_default;
// TODO: in order to avoid the tiberius transmute, we should define other method that takes the db_conn as a mut ref

pub trait DbConnection {
    // TODO: guess that this is the trait that must remain sealed
    fn launch<'a>(
        &self,
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
    ) -> impl Future<Output = Result<CanyonRows, Box<(dyn Error + Sync + Send)>>> + Send;

    // TODO: the querybuilder needs to know the underlying db type associated with self, so provide
    // a method to obtain it
    fn get_database_type(&self) -> Result<DatabaseType, Box<(dyn Error + Sync + Send)>>;
}

/// This impl of [` DbConnection` ] for [`&str`] allows the client to use the exposed input types
/// on the public API that works with a generic parameter to refer to a database connection
/// directly with an [`&str`] that must match one of the datasources defined
/// within the user config file
impl DbConnection for &str {
    fn launch<'a>(&self, stmt: &str, params: &[&'a dyn QueryParameter<'a>])
        -> impl Future<Output=Result<CanyonRows, Box<(dyn Error + Sync + Send)>>> + Send
    {
        async move {
            let sane_ds_name = if !self.is_empty() {
                Some(*self)
            } else {
                None
            };
            let conn = get_database_connection_by_ds(sane_ds_name).await?;
            conn.launch(stmt, params).await
        }
    }

    fn get_database_type(&self) -> Result<DatabaseType, Box<(dyn Error + Sync + Send)>> {
        Ok(find_datasource_by_name_or_try_default(Some(*self))?.get_db_type())
    }
}

pub trait Transaction<T> {
    // provisional name
    /// Performs a query against the targeted database by the selected or
    /// the defaulted datasource, wrapping the resultant collection of entities
    /// in [`super::rows::CanyonRows`]
    fn query<'a, S, Z>(
        stmt: S,
        params: Z,
        input: impl DbConnection + Send + 'a,
    ) -> impl Future<Output = Result<CanyonRows, Box<(dyn Error + Sync + Send)>>> + Send
    where
        S: AsRef<str> + Display + Sync + Send + 'a,
        Z: AsRef<[&'a dyn QueryParameter<'a>]> + Sync + Send + 'a
    {
        async move {
            input.launch(stmt.as_ref(), params.as_ref()).await
        }
    }
}
