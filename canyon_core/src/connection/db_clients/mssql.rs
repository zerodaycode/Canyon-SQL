#[cfg(feature = "mssql")]
use async_std::net::TcpStream;
use std::error::Error;
use std::fmt::Display;
use std::future::Future;
use crate::connection::database_type::DatabaseType;
use crate::connection::db_connector::DbConnection;
use crate::{query_parameters::QueryParameter, rows::CanyonRows};
use tiberius::Query;
use crate::mapper::RowMapper;

/// A connection with a `SqlServer` database
#[cfg(feature = "mssql")]
pub struct SqlServerConnection {
    pub client: &'static mut tiberius::Client<TcpStream>,
}

impl DbConnection for SqlServerConnection {
    fn query<'a>(
        &self,
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
    ) -> impl Future<Output = Result<CanyonRows, Box<(dyn Error + Sync + Send)>>> + Send
    {
        sqlserver_query_launcher::launch(stmt, params, self)
    }

    fn query_rows<'a, S, Z, R: RowMapper<R>>(&self, stmt: S, params: Z) -> impl Future<Output=Result<Vec<R>, Box<(dyn Error + Sync + Send + 'a)>>> + Send
    where
        S: AsRef<str> + Display + Sync + Send + 'a,
        Z: AsRef<[&'a dyn QueryParameter<'a>]> + Sync + Send + 'a
    {
        async move { todo!() }
    }

    fn query_one<'a, R>(&self, stmt: &str, params: &[&'a (dyn QueryParameter<'a>)])
                        -> impl Future<Output=Result<Option<R>, Box<(dyn Error + Send + Sync)>>> + Send
    where
        R: RowMapper<R>
    {
        sqlserver_query_launcher::query_one(stmt, params, self)
    }
    
    fn get_database_type(&self) -> Result<DatabaseType, Box<(dyn Error + Sync + Send)>> {
        Ok(DatabaseType::SqlServer)
    }
}

#[cfg(feature = "mssql")]
pub(crate) mod sqlserver_query_launcher {
    use crate::mapper::RowMapper;
    use super::*;

    #[inline(always)]
    pub(crate) async fn launch<'a>(
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
        conn: &SqlServerConnection,
    ) -> Result<CanyonRows, Box<(dyn Error + Sync + Send)>> {
        // Re-generate de insert statement to adequate it to the SQL SERVER syntax to retrieve the PK value(s) after insert
        // TODO: redo this branch into the generated queries, before the MACROS
        let mut stmt = String::from(stmt);
        if stmt.contains("RETURNING") {
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

    pub(crate) async fn query_one<'a, R>(
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
        conn: &SqlServerConnection,
    )
                        -> Result<Option<R>, Box<(dyn Error + Send + Sync)>>
    where
        R: RowMapper<R>
    {
        let mut stmt = String::from(stmt);
        if stmt.contains("RETURNING") {
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

        // TODO: We must address the query generation. Look at the returning example, or the
        // replace below. We may use our own type Query to address this concerns when the query
        // is generated
        let mut mssql_query = Query::new(stmt.to_owned().replace('$', "@P"));
        params.iter().for_each(|param| mssql_query.bind(*param));

        #[allow(mutable_transmutes)] // TODO: pls solve this elegantly someday :(
        let sqlservconn =
            unsafe { std::mem::transmute::<&SqlServerConnection, &mut SqlServerConnection>(conn) };
        let result = mssql_query
            .query(sqlservconn.client)
            .await?
            .into_row()
            .await?;

        match result {
            Some(r) => { Ok(Some(R::deserialize_sqlserver(&r))) }
            None => { Ok(None) }
        }
    }
}
