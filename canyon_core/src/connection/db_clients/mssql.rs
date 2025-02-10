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
    fn query_rows<'a>(
        &self,
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
    ) -> impl Future<Output = Result<CanyonRows, Box<(dyn Error + Sync + Send)>>> + Send
    {
        sqlserver_query_launcher::query_rows(stmt, params, self)
    }

    fn query<'a, S, R: RowMapper<R>>(&self, stmt: S, params: &[&'a (dyn QueryParameter<'_>)])
        -> impl Future<Output=Result<Vec<R>, Box<(dyn Error + Sync + Send)>>> + Send
    where
        S: AsRef<str> + Display + Send
    {
        // sqlserver_query_launcher::query(stmt, params, self)
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
    use tiberius::{ColumnData, QueryStream};
    use crate::mapper::RowMapper;
    use super::*;
    
    #[inline(always)]
    pub(crate) async fn query<'a, S, R: RowMapper<R>>(
        stmt: S,
        params: &[&'a dyn QueryParameter<'_>],
        conn: &SqlServerConnection,
    ) -> Result<Vec<R>, Box<(dyn Error + Sync + Send)>>
    where
        S: AsRef<str> + Display + Send
    {
        Ok(execute_query(stmt.as_ref(), params, conn)
            .await?
            .into_results()
            .await?
            .into_iter()
            .flatten()
            .map(|row| R::deserialize_sqlserver(&row))
            .collect::<Vec<R>>())
    }
    
    #[inline(always)]
    pub(crate) async fn query_rows<'a>(
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
        conn: &SqlServerConnection,
    ) -> Result<CanyonRows, Box<(dyn Error + Sync + Send)>> {
        let result = execute_query(stmt, params, conn)
            .await?
            .into_results()
            .await?
            .into_iter()
            .flatten()
            .collect();

        Ok(CanyonRows::Tiberius(result))
    }

    pub(crate) async fn query_one<'a, R>(
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
        conn: &SqlServerConnection,
    ) -> Result<Option<R>, Box<(dyn Error + Send + Sync)>>
    where
        R: RowMapper<R>
    {
        let result = execute_query(stmt, params, conn)
            .await?
            .into_row()
            .await?;

        match result {
            Some(r) => { Ok(Some(R::deserialize_sqlserver(&r))) }
            None => { Ok(None) }
        }
    }

    async fn execute_query<'a>(stmt: &str, params: &[&'a (dyn QueryParameter<'_>)], conn: &SqlServerConnection)
        -> Result<QueryStream<'a>, Box<(dyn Error + Send + Sync)>>
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
        params.iter().for_each(|param| {
            let column_data = param.as_sqlserver_param();
            match column_data {
                ColumnData::U8(v) => { mssql_query.bind(v) }
                ColumnData::I16(v) => { mssql_query.bind(v) }
                ColumnData::I32(v) => { mssql_query.bind(v) }
                ColumnData::I64(v) => { mssql_query.bind(v) }
                ColumnData::F32(v) => { mssql_query.bind(v) }
                ColumnData::F64(v) => { mssql_query.bind(v) }
                ColumnData::Bit(v) => { mssql_query.bind(v) }
                ColumnData::String(v) => { mssql_query.bind(v) }
                ColumnData::Guid(v) => { mssql_query.bind(v) }
                ColumnData::Binary(v) => { mssql_query.bind(v) }
                ColumnData::Numeric(v) => { mssql_query.bind(v) }
                ColumnData::Xml(v) => { mssql_query.bind(v.as_deref().map(ToString::to_string)) }
                ColumnData::DateTime(v) => { todo!() }
                ColumnData::SmallDateTime(v) => { todo!() }
                ColumnData::Time(v) => { todo!() }
                ColumnData::Date(v) => { todo!() }
                ColumnData::DateTime2(v) => { todo!() }
                ColumnData::DateTimeOffset(v) => { todo!() }
            }
            // mssql_query.bind()
        });

        #[allow(mutable_transmutes)] // TODO: pls solve this elegantly someday :(
        let sqlservconn =
            unsafe { std::mem::transmute::<&SqlServerConnection, &mut SqlServerConnection>(conn) };
        Ok(mssql_query
            .query(sqlservconn.client)
            .await?)
    }
}
