use crate::connection::clients::mssql::sqlserver_query_launcher::execute_query;
use crate::{
    connection::{
        clients::mssql::SqlServerConnection, contracts::DbConnection, database_type::DatabaseType,
    },
    mapper::RowMapper,
    query::parameters::QueryParameter,
    rows::{CanyonRows, FromSqlOwnedValue},
};
use std::error::Error;

impl DbConnection for SqlServerConnection {
    async fn query_rows(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<CanyonRows, Box<dyn Error + Send + Sync>> {
        let mut conn = self.get_pooled().await?;
        let result = execute_query(stmt, params, &mut conn)
            .await?
            .into_results()
            .await?
            .into_iter()
            .flatten()
            .collect();

        Ok(CanyonRows::Tiberius(result))
    }

    async fn query<S, R>(
        &self,
        stmt: S,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<Vec<R>, Box<dyn Error + Send + Sync>>
    where
        S: AsRef<str> + Send,
        R: RowMapper,
        Vec<R>: FromIterator<<R as RowMapper>::Output>,
    {
        let mut conn = self.get_pooled().await?;
        Ok(execute_query(stmt.as_ref(), params, &mut conn)
            .await?
            .into_results()
            .await?
            .into_iter()
            .flatten()
            .flat_map(|row| R::deserialize_sqlserver(&row))
            .collect::<Vec<R>>())
    }

    async fn query_one<R>(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<Option<R::Output>, Box<dyn Error + Send + Sync>>
    where
        R: RowMapper,
    {
        let mut conn = self.get_pooled().await?;

        let result = execute_query(stmt, params, &mut conn)
            .await?
            .into_row()
            .await?;

        match result {
            Some(r) => Ok(Some(R::deserialize_sqlserver(&r)?)),
            None => Ok(None),
        }
    }

    async fn query_one_for<T: FromSqlOwnedValue<T>>(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<T, Box<dyn Error + Send + Sync>> {
        let mut conn = self.get_pooled().await?;
        let row = crate::connection::clients::mssql::sqlserver_query_launcher::execute_query(
            stmt, params, &mut conn,
        )
        .await?
        .into_row()
        .await?
        .ok_or_else(|| {
            format!(
                "Failure executing 'query_one_for' while retrieving the first row with stmt: {:?}",
                stmt
            )
        })?;

        Ok(row
            .into_iter()
            .map(T::from_sql_owned)
            .collect::<Vec<_>>()
            .remove(0)?
            .ok_or_else(|| format!("Failure executing 'query_one_for' while retrieving the first column value on the first row with stmt: {:?}", stmt))?
        )
    }

    async fn execute(
        &self,
        stmt: &str,
        params: &[&'_ dyn QueryParameter],
    ) -> Result<u64, Box<dyn Error + Send + Sync>> {
        let mssql_query = crate::connection::clients::mssql::sqlserver_query_launcher::generate_mssql_query_client(stmt, params).await;
        let mut conn = self.get_pooled().await?;

        mssql_query
            .execute(&mut conn)
            .await
            .map(|r| r.total())
            .map_err(From::from)
    }

    fn get_database_type(&self) -> Result<DatabaseType, Box<dyn Error + Send + Sync>> {
        Ok(DatabaseType::SqlServer)
    }
}
