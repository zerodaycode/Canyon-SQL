use std::fmt::Display;

pub trait DatabaseQuery<T> { // provisional name
    /// Performs a query against the targeted database by the selected or
    /// the defaulted datasource, wrapping the resultant collection of entities
    /// in [`super::rows::CanyonRows`]
    async fn query<'a, S, Z>(
        stmt: S,
        params: Z,
        database_conn: impl DatabaseConnection,
    ) -> Result<CanyonRows<T>, Box<(dyn std::error::Error + Sync + Send + 'static)>>
    where
        S: AsRef<str> + Display + Sync + Send + 'a,
        Z: AsRef<[&'a dyn QueryParameter<'a>]> + Sync + Send + 'a,
    {
        match *database_conn { // TODO: this query launch should be implemented in DatabaseClient on
                               // canyon_connection, after the actual DatabaseConnection struct in
                               // canyon_connection is renamed to DatabaseClient, so DatabaseClient
                               // implements DatabaseConnection from this crate, and we will be
                               // happy
            #[cfg(feature = "postgres")]
            DatabaseConnection::Postgres(_) => {
                postgres_query_launcher::launch::<T>(
                    database_conn,
                    stmt.to_string(),
                    params.as_ref(),
                )
                .await
            }
            #[cfg(feature = "mssql")]
            DatabaseConnection::SqlServer(_) => {
                sqlserver_query_launcher::launch::<T, Z>(
                    database_conn,
                    &mut stmt.to_string(),
                    params,
                )
                .await
            }
            #[cfg(feature = "mysql")]
            DatabaseConnection::MySQL(_) => {
                mysql_query_launcher::launch::<T>(database_conn, stmt.to_string(), params.as_ref())
                    .await
            }
        }
    }
}
