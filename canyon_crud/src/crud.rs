use std::fmt::Display;

use async_trait::async_trait;
use canyon_connection::canyon_database_connector::DatabaseType;
use canyon_connection::CACHED_DATABASE_CONN;

use crate::bounds::QueryParameters;
use crate::mapper::RowMapper;
use crate::query_elements::query_builder::{
    DeleteQueryBuilder, SelectQueryBuilder, UpdateQueryBuilder,
};
use crate::result::DatabaseResult;

/// This traits defines and implements a query against a database given
/// an statemt `stmt` and the params to pass the to the client.
///
/// It returns a [`DatabaseResult`], which is the core Canyon type to wrap
/// the result of the query and, if the user desires,
/// automatically map it to an struct.
#[async_trait]
#[allow(clippy::question_mark)]
pub trait Transaction<T> {
    /// Performs a query against the targeted database by the selected datasource.
    ///
    /// No datasource means take the entry zero
    async fn query<'a, S, Z>(
        stmt: S,
        params: Z,
        datasource_name: &'a str,
    ) -> Result<DatabaseResult<T>, Box<(dyn std::error::Error + Sync + Send + 'static)>>
    where
        S: AsRef<str> + Display + Sync + Send + 'a,
        Z: AsRef<[&'a dyn QueryParameters<'a>]> + Sync + Send + 'a,
    {
        let guarded_cache = CACHED_DATABASE_CONN.lock().await;

        let database_conn = if datasource_name.is_empty() {
            guarded_cache
                .values()
                .next()
                .expect("No default datasource found. Check your `canyon.toml` file")
        } else {
            guarded_cache.get(datasource_name)
                .expect(
                    &format!(
                        "Canyon couldn't find a datasource in the pool with the argument provided: {datasource_name}"
                    )
                )
        };

        match database_conn.database_type {
            DatabaseType::PostgreSql => {
                postgres_query_launcher::launch::<T>(
                    database_conn,
                    stmt.to_string(),
                    params.as_ref(),
                )
                .await
            }
            DatabaseType::SqlServer => {
                sqlserver_query_launcher::launch::<T, Z>(
                    database_conn,
                    &mut stmt.to_string(),
                    params,
                )
                .await
            }
        }
    }
}

/// *CrudOperations* it's the core part of Canyon-SQL.
///
/// Here it's defined and implemented every CRUD operation
/// that the user has available, just by deriving the `CanyonCrud`
/// derive macro when a struct contains the annotation.
///
/// Also, this traits needs that the type T over what it's generified
/// to implement certain types in order to work correctly.
///
/// The most notorious one it's the [`RowMapper<T>`] one, which allows
/// Canyon to directly maps database results into structs.
///
/// See it's definition and docs to see the implementations.
/// Also, you can find the written macro-code that performs the auto-mapping
/// in the *canyon_sql::canyon_macros* crates, on the root of this project.
#[async_trait]
pub trait CrudOperations<T>: Transaction<T>
where
    T: CrudOperations<T> + RowMapper<T>,
{
    async fn find_all<'a>() -> Result<Vec<T>, Box<(dyn std::error::Error + Send + Sync + 'static)>>;

    async fn find_all_datasource<'a>(
        datasource_name: &'a str,
    ) -> Result<Vec<T>, Box<(dyn std::error::Error + Send + Sync + 'static)>>;

    async fn find_all_unchecked<'a>() -> Vec<T>;

    async fn find_all_unchecked_datasource<'a>(datasource_name: &'a str) -> Vec<T>;

    fn select_query<'a>() -> SelectQueryBuilder<'a, T>;

    fn select_query_datasource<'a>(datasource_name: &'a str) -> SelectQueryBuilder<'a, T>;

    async fn count() -> Result<i64, Box<(dyn std::error::Error + Send + Sync + 'static)>>;

    async fn count_datasource<'a>(
        datasource_name: &'a str,
    ) -> Result<i64, Box<(dyn std::error::Error + Send + Sync + 'static)>>;

    async fn find_by_pk<'a>(
        value: &'a dyn QueryParameters<'a>,
    ) -> Result<Option<T>, Box<(dyn std::error::Error + Send + Sync + 'static)>>;

    async fn find_by_pk_datasource<'a>(
        value: &'a dyn QueryParameters<'a>,
        datasource_name: &'a str,
    ) -> Result<Option<T>, Box<(dyn std::error::Error + Send + Sync + 'static)>>;

    async fn insert<'a>(
        &mut self,
    ) -> Result<(), Box<dyn std::error::Error + Sync + std::marker::Send>>;

    async fn insert_datasource<'a>(
        &mut self,
        datasource_name: &'a str,
    ) -> Result<(), Box<dyn std::error::Error + Sync + std::marker::Send>>;

    async fn multi_insert<'a>(
        instances: &'a mut [&'a mut T],
    ) -> Result<(), Box<(dyn std::error::Error + Send + Sync + 'static)>>;

    async fn multi_insert_datasource<'a>(
        instances: &'a mut [&'a mut T],
        datasource_name: &'a str,
    ) -> Result<(), Box<(dyn std::error::Error + Send + Sync + 'static)>>;

    async fn update(&self) -> Result<(), Box<dyn std::error::Error + Sync + std::marker::Send>>;

    async fn update_datasource<'a>(
        &self,
        datasource_name: &'a str,
    ) -> Result<(), Box<dyn std::error::Error + Sync + std::marker::Send>>;

    fn update_query<'a>() -> UpdateQueryBuilder<'a, T>;

    fn update_query_datasource(datasource_name: &str) -> UpdateQueryBuilder<'_, T>;

    async fn delete(&self) -> Result<(), Box<dyn std::error::Error + Sync + std::marker::Send>>;

    async fn delete_datasource<'a>(
        &self,
        datasource_name: &'a str,
    ) -> Result<(), Box<dyn std::error::Error + Sync + std::marker::Send>>;

    fn delete_query<'a>() -> DeleteQueryBuilder<'a, T>;

    fn delete_query_datasource(datasource_name: &str) -> DeleteQueryBuilder<'_, T>;
}

mod postgres_query_launcher {
    use crate::bounds::QueryParameters;
    use crate::result::DatabaseResult;
    use canyon_connection::canyon_database_connector::DatabaseConnection;

    pub async fn launch<'a, T>(
        db_conn: &DatabaseConnection,
        // datasource_name: &str,
        stmt: String,
        params: &'a [&'_ dyn QueryParameters<'_>],
    ) -> Result<DatabaseResult<T>, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
        let mut m_params = Vec::new();
        for param in params {
            m_params.push(param.as_postgres_param());
        }

        let query_result = db_conn
            .postgres_connection
            .as_ref()
            .unwrap()
            .client
            .query(&stmt, m_params.as_slice())
            .await;

        if let Err(error) = query_result {
            Err(Box::new(error))
        } else {
            Ok(DatabaseResult::new_postgresql(
                query_result.expect("A really bad error happened querying PostgreSQL"),
            ))
        }
    }
}

mod sqlserver_query_launcher {
    use std::mem::transmute;

    use canyon_connection::tiberius::Row;

    use crate::{
        bounds::QueryParameters,
        canyon_connection::{canyon_database_connector::DatabaseConnection, tiberius::Query},
        result::DatabaseResult,
    };

    pub async fn launch<'a, T, Z>(
        db_conn: &&mut DatabaseConnection,
        stmt: &mut String,
        params: Z,
    ) -> Result<DatabaseResult<T>, Box<(dyn std::error::Error + Send + Sync + 'static)>>
    where
        Z: AsRef<[&'a dyn QueryParameters<'a>]> + Sync + Send + 'a,
    {
        // Re-generate de insert statement to adecuate it to the SQL SERVER syntax to retrieve the PK value(s) after insert
        if stmt.contains("RETURNING") {
            let c = stmt.clone();
            let temp = c
                .split_once("RETURNING")
                .expect("An error happened generating an INSERT statement for a SQL SERVER client");
            let temp2 = temp.0.split_once("VALUES").expect(
                "An error happened generating an INSERT statement for a SQL SERVER client [1]",
            );

            *stmt = format!(
                "{} OUTPUT inserted.{} VALUES {}",
                temp2.0.trim(),
                temp.1.trim(),
                temp2.1.trim()
            );
        }

        let mut mssql_query = Query::new(stmt.to_owned().replace('$', "@P"));
        params
            .as_ref()
            .iter()
            .for_each(|param| mssql_query.bind(*param));

        #[allow(mutable_transmutes)]
        let _results: Vec<Row> = mssql_query
            .query(
                unsafe { transmute::<&DatabaseConnection, &mut DatabaseConnection>(db_conn) }
                    .sqlserver_connection
                    .as_mut()
                    .expect("Error querying the MSSQL database")
                    .client,
            )
            .await?
            .into_results()
            .await?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        Ok(DatabaseResult::new_sqlserver(_results))
    }
}
