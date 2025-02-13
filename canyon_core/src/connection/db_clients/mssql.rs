use crate::connection::database_type::DatabaseType;
use crate::connection::db_connector::DbConnection;
use crate::mapper::RowMapper;
use crate::rows::FromSqlOwnedValue;
use crate::{query_parameters::QueryParameter, rows::CanyonRows};
#[cfg(feature = "mssql")]
use async_std::net::TcpStream;
use std::error::Error;
use std::fmt::Display;
use std::future::Future;
use tiberius::Query;

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
    ) -> impl Future<Output = Result<CanyonRows, Box<(dyn Error + Sync + Send)>>> + Send {
        sqlserver_query_launcher::query_rows(stmt, params, self)
    }

    fn query<'a, S, R>(
        &self,
        stmt: S,
        params: &[&'a (dyn QueryParameter<'a>)],
    ) -> impl Future<Output = Result<Vec<R>, Box<(dyn Error + Sync + Send)>>> + Send
    where
        S: AsRef<str> + Display + Send,
        R: RowMapper<Output = R>,
        Vec<R>: FromIterator<<R as RowMapper>::Output>
    {
        sqlserver_query_launcher::query(stmt, params, self)
    }

    fn query_one<'a, R>(
        &self,
        stmt: &str,
        params: &[&'a (dyn QueryParameter<'a>)],
    ) -> impl Future<Output = Result<Option<R>, Box<(dyn Error + Send + Sync)>>> + Send
    where
        R: RowMapper<Output = R>
    {
        sqlserver_query_launcher::query_one(stmt, params, self)
    }

    fn query_one_for<'a, T: FromSqlOwnedValue<T>>(
        &self,
        stmt: &str,
        params: &[&'a (dyn QueryParameter<'a>)],
    ) -> impl Future<Output = Result<T, Box<(dyn Error + Send + Sync)>>> + Send {
        sqlserver_query_launcher::query_one_for(stmt, params, self)
    }

    fn execute<'a>(
        &self,
        stmt: &str,
        params: &[&'a (dyn QueryParameter<'a>)],
    ) -> impl Future<Output = Result<u64, Box<(dyn Error + Send + Sync)>>> + Send {
        sqlserver_query_launcher::execute(stmt, params, self)
    }

    fn get_database_type(&self) -> Result<DatabaseType, Box<(dyn Error + Sync + Send)>> {
        Ok(DatabaseType::SqlServer)
    }
}

#[cfg(feature = "mssql")]
pub(crate) mod sqlserver_query_launcher {
    use super::*;
    use crate::mapper::RowMapper;
    use crate::rows::FromSqlOwnedValue;
    use tiberius::QueryStream;

    #[inline(always)]
    pub(crate) async fn query<'a, S, R>(
        stmt: S,
        params: &[&'a dyn QueryParameter<'a>],
        conn: &SqlServerConnection,
    ) -> Result<Vec<R>, Box<(dyn Error + Sync + Send)>>
    where
        S: AsRef<str> + Display + Send,
        R: RowMapper<Output = R>,
        Vec<R>: FromIterator<<R as RowMapper>::Output>
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
        R: RowMapper<Output = R>,
    {
        let result = execute_query(stmt, params, conn).await?.into_row().await?;

        match result {
            Some(r) => Ok(Some(R::deserialize_sqlserver(&r))),
            None => Ok(None),
        }
    }

    pub(crate) async fn query_one_for<'a, T: FromSqlOwnedValue<T>>(
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
        conn: &SqlServerConnection,
    ) -> Result<T, Box<(dyn Error + Send + Sync)>> {
        let row = execute_query(stmt, params, conn)
            .await?
            .into_row()
            .await?
            .ok_or_else(|| format!("Failure executing 'query_one_for' while retrieving the first row with stmt: {:?}", stmt))?;

        Ok(row
            .into_iter()
            .map(T::from_sql_owned)
            .collect::<Vec<_>>()
            .remove(0)?
            .ok_or_else(|| format!("Failure executing 'query_one_for' while retrieving the first column value on the first row with stmt: {:?}", stmt))?
        )
    }

    pub(crate) async fn execute<'a>(
        stmt: &str,
        params: &[&'a dyn QueryParameter<'a>],
        conn: &SqlServerConnection,
    ) -> Result<u64, Box<(dyn Error + Send + Sync)>> {
        let mssql_query = generate_mssql_stmt(stmt, params).await;

        #[allow(mutable_transmutes)] // TODO: pls solve this elegantly someday :(
        let sqlservconn =
            unsafe { std::mem::transmute::<&SqlServerConnection, &mut SqlServerConnection>(conn) };

        mssql_query
            .execute(sqlservconn.client)
            .await
            .map(|r| r.total())
            .map_err(From::from)
    }

    async fn execute_query<'a>(
        stmt: &str,
        params: &[&'a (dyn QueryParameter<'a>)],
        conn: &SqlServerConnection,
    ) -> Result<QueryStream<'a>, Box<(dyn Error + Send + Sync)>> {
        let mssql_query = generate_mssql_stmt(stmt, params).await;

        #[allow(mutable_transmutes)] // TODO: pls solve this elegantly someday :(
        let sqlservconn =
            unsafe { std::mem::transmute::<&SqlServerConnection, &mut SqlServerConnection>(conn) };
        Ok(mssql_query.query(sqlservconn.client).await?)
    }

    async fn generate_mssql_stmt<'a>(
        stmt: &str,
        params: &[&'a (dyn QueryParameter<'a>)],
    ) -> Query<'a> {
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

        // TODO: We must address the query generation
        let mut mssql_query = Query::new(stmt.to_owned().replace('$', "@P"));
        params.iter().for_each(|param| { mssql_query.bind(*param); });
       
        mssql_query
    }
}
