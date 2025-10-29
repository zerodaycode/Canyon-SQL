use crate::connection::{MsManager, SqlServerConnectionPool};
use crate::query::parameters::QueryParameter;
use bb8::PooledConnection;
use std::error::Error;
#[cfg(feature = "mssql")]
use tiberius::Query;

/// A connection with a `SqlServer` database
#[cfg(feature = "mssql")] // TODO: remove the local cfg and put them at module level
pub struct SqlServerConnection(SqlServerConnectionPool);

impl SqlServerConnection {
    pub fn new(pool: SqlServerConnectionPool) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(Self(pool))
    }
    pub async fn get_pooled(
        &self,
    ) -> Result<PooledConnection<'_, MsManager>, Box<dyn Error + Send + Sync>> {
        Ok(self.0.get().await?)
    }
}

#[cfg(feature = "mssql")]
pub(crate) mod sqlserver_query_launcher {
    use super::*;
    use tiberius::QueryStream;

    pub(crate) async fn execute_query<'a>(
        stmt: &str,
        params: &[&dyn QueryParameter],
        conn: &'a mut bb8::PooledConnection<'_, bb8_tiberius::ConnectionManager>,
    ) -> Result<QueryStream<'a>, Box<dyn Error + Send + Sync>> {
        let mssql_query = generate_mssql_query_client(stmt, params).await;
        mssql_query.query(conn).await.map_err(From::from)
    }

    pub(crate) async fn generate_mssql_query_client<'a>(
        stmt: &str,
        params: &[&'a dyn QueryParameter],
    ) -> Query<'a> {
        let mut stmt = String::from(stmt);
        if stmt.contains("RETURNING") {
            // TODO: when the InsertQuerybuilder with a api on the builder for the returning clause
            let c = stmt.clone();
            let temp = c.split_once("RETURNING").unwrap();
            let temp2 = temp.0.split_once("VALUES").unwrap();

            stmt = format!(
                "{} OUTPUT inserted.{} VALUES {}",
                temp2.0.trim(),
                temp.1.trim(),
                temp2.1.trim()
            );
        }

        let stmt = stmt.replace('$', "@P"); // TODO: this should be solved by the querybuilder
        generate_query_and_bind_params(stmt, params)
    }

    // Query and parameters are generated in this procedure together to avoid lifetime errors
    fn generate_query_and_bind_params<'a>(
        stmt: String,
        params: &[&'a (dyn QueryParameter + 'a)],
    ) -> Query<'a> {
        let mut mssql_query = Query::new(stmt);
        params.iter().for_each(|param| {
            mssql_query.bind(*param);
        });
        mssql_query
    }
}
