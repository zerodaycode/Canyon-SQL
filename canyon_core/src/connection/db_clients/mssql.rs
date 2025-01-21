#[cfg(feature = "mssql")]
use async_std::net::TcpStream;

use async_trait::async_trait;
use crate::{query::DbConnection, query_parameters::QueryParameter, rows::CanyonRows};
use tiberius::Query;

/// A connection with a `SqlServer` database
#[cfg(feature = "mssql")]
pub struct SqlServerConnection {
    pub client: &'static mut tiberius::Client<TcpStream>,
}

#[async_trait]
impl DbConnection for SqlServerConnection {
    async fn launch<'a>(
        &self,
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
    ) -> Result<CanyonRows, Box<(dyn std::error::Error + Sync + Send)>> {
        sqlserver_query_launcher::launch(stmt, params, self).await
    }
}

#[cfg(feature = "mssql")]
pub(crate) mod sqlserver_query_launcher {
    use super::*;

    #[inline(always)]

    pub(crate) async fn launch<'a>(
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
        conn: &SqlServerConnection,
    ) -> Result<CanyonRows, Box<(dyn std::error::Error + Sync + Send)>> {
        // Re-generate de insert statement to adequate it to the SQL SERVER syntax to retrieve the PK value(s) after insert
        // TODO: redo this branch into the generated queries, before the MACROS
        // if stmt.contains("RETURNING") {
        //     let c = stmt.clone();
        //     let temp = c.split_once("RETURNING").unwrap();
        //     let temp2 = temp.0.split_once("VALUES").unwrap();
        //
        //     *stmt = format!(
        //         "{} OUTPUT inserted.{} VALUES {}",
        //         temp2.0.trim(),
        //         temp.1.trim(),
        //         temp2.1.trim()
        //     );
        // }

        // TODO: We must address the query generation. Look at the returning example, or the
        // replace below. We may use our own type Query to address this concerns when the query
        // is generated
        let mut mssql_query = Query::new(stmt.to_owned().replace('$', "@P"));
        params.iter().for_each(|param| mssql_query.bind(*param));

        #[allow(mutable_transmutes)] // TODO: pls solve this elegantly someday :(
        let sqlservconn =
            unsafe { std::mem::transmute::<&SqlServerConnection, &mut SqlServerConnection>(conn) };
        let _results = mssql_query
            .query(sqlservconn.client)
            .await?
            .into_results()
            .await?;

        Ok(CanyonRows::Tiberius(
            _results.into_iter().flatten().collect(),
        ))
    }
}
