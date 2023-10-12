use async_trait::async_trait;
use std::fmt::Display;

use canyon_connection::canyon_database_connector::DatabaseConnection;
use canyon_connection::{get_database_connection, CACHED_DATABASE_CONN};

use crate::bounds::QueryParameter;
use crate::mapper::RowMapper;
use crate::query_elements::query_builder::{
    DeleteQueryBuilder, SelectQueryBuilder, UpdateQueryBuilder,
};
use crate::rows::CanyonRows;

#[cfg(feature = "mysql")]
pub const DETECT_PARAMS_IN_QUERY: &str = r"\$([\d])+";
#[cfg(feature = "mysql")]
pub const DETECT_QUOTE_IN_QUERY: &str = r#"\"|\\"#;

/// This traits defines and implements a query against a database given
/// an statement `stmt` and the params to pass the to the client.
///
/// Returns [`std::result::Result`] of [`CanyonRows`], which is the core Canyon type to wrap
/// the result of the query provide automatic mappings and deserialization
#[async_trait]
pub trait Transaction<T> {
    /// Performs a query against the targeted database by the selected or
    /// the defaulted datasource, wrapping the resultant collection of entities
    /// in [`super::rows::CanyonRows`]
    async fn query<'a, S, Z>(
        stmt: S,
        params: Z,
        datasource_name: &'a str,
    ) -> Result<CanyonRows<T>, Box<(dyn std::error::Error + Sync + Send + 'static)>>
    where
        S: AsRef<str> + Display + Sync + Send + 'a,
        Z: AsRef<[&'a dyn QueryParameter<'a>]> + Sync + Send + 'a,
    {
        let mut guarded_cache = CACHED_DATABASE_CONN.lock().await;
        let database_conn = get_database_connection(datasource_name, &mut guarded_cache);

        match *database_conn {
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
/// in the *canyon_sql_root::canyon_macros* crates, on the root of this project.
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

    fn select_query_datasource(datasource_name: &str) -> SelectQueryBuilder<'_, T>;

    async fn count() -> Result<i64, Box<(dyn std::error::Error + Send + Sync + 'static)>>;

    async fn count_datasource<'a>(
        datasource_name: &'a str,
    ) -> Result<i64, Box<(dyn std::error::Error + Send + Sync + 'static)>>;

    async fn find_by_pk<'a>(
        value: &'a dyn QueryParameter<'a>,
    ) -> Result<Option<T>, Box<(dyn std::error::Error + Send + Sync + 'static)>>;

    async fn find_by_pk_datasource<'a>(
        value: &'a dyn QueryParameter<'a>,
        datasource_name: &'a str,
    ) -> Result<Option<T>, Box<(dyn std::error::Error + Send + Sync + 'static)>>;

    async fn insert<'a>(&mut self) -> Result<(), Box<dyn std::error::Error + Sync + Send>>;

    async fn insert_datasource<'a>(
        &mut self,
        datasource_name: &'a str,
    ) -> Result<(), Box<dyn std::error::Error + Sync + Send>>;

    async fn multi_insert<'a>(
        instances: &'a mut [&'a mut T],
    ) -> Result<(), Box<(dyn std::error::Error + Send + Sync + 'static)>>;

    async fn multi_insert_datasource<'a>(
        instances: &'a mut [&'a mut T],
        datasource_name: &'a str,
    ) -> Result<(), Box<(dyn std::error::Error + Send + Sync + 'static)>>;

    async fn update(&self) -> Result<(), Box<dyn std::error::Error + Sync + Send>>;

    async fn update_datasource<'a>(
        &self,
        datasource_name: &'a str,
    ) -> Result<(), Box<dyn std::error::Error + Sync + Send>>;

    fn update_query<'a>() -> UpdateQueryBuilder<'a, T>;

    fn update_query_datasource(datasource_name: &str) -> UpdateQueryBuilder<'_, T>;

    async fn delete(&self) -> Result<(), Box<dyn std::error::Error + Sync + Send>>;

    async fn delete_datasource<'a>(
        &self,
        datasource_name: &'a str,
    ) -> Result<(), Box<dyn std::error::Error + Sync + Send>>;

    fn delete_query<'a>() -> DeleteQueryBuilder<'a, T>;

    fn delete_query_datasource(datasource_name: &str) -> DeleteQueryBuilder<'_, T>;
}

#[cfg(feature = "postgres")]
mod postgres_query_launcher {
    use canyon_connection::canyon_database_connector::DatabaseConnection;

    use crate::bounds::QueryParameter;
    use crate::rows::CanyonRows;

    pub async fn launch<'a, T>(
        db_conn: &DatabaseConnection,
        stmt: String,
        params: &'a [&'_ dyn QueryParameter<'_>],
    ) -> Result<CanyonRows<T>, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
        let mut m_params = Vec::new();
        for param in params {
            m_params.push(param.as_postgres_param());
        }

        let r = db_conn
            .postgres_connection()
            .client
            .query(&stmt, m_params.as_slice())
            .await?;

        Ok(CanyonRows::Postgres(r))
    }
}

#[cfg(feature = "mssql")]
mod sqlserver_query_launcher {
    use crate::rows::CanyonRows;
    use crate::{
        bounds::QueryParameter,
        canyon_connection::{canyon_database_connector::DatabaseConnection, tiberius::Query},
    };

    pub async fn launch<'a, T, Z>(
        db_conn: &mut DatabaseConnection,
        stmt: &mut String,
        params: Z,
    ) -> Result<CanyonRows<T>, Box<(dyn std::error::Error + Send + Sync + 'static)>>
    where
        Z: AsRef<[&'a dyn QueryParameter<'a>]> + Sync + Send + 'a,
    {
        // Re-generate de insert statement to adequate it to the SQL SERVER syntax to retrieve the PK value(s) after insert
        if stmt.contains("RETURNING") {
            let c = stmt.clone();
            let temp = c.split_once("RETURNING").unwrap();
            let temp2 = temp.0.split_once("VALUES").unwrap();

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

        let _results = mssql_query
            .query(db_conn.sqlserver_connection().client)
            .await?
            .into_results()
            .await?;

        Ok(CanyonRows::Tiberius(
            _results.into_iter().flatten().collect(),
        ))
    }
}

#[cfg(feature = "mysql")]
mod mysql_query_launcher {
    use std::sync::Arc;

    use mysql_async::prelude::Query;
    use mysql_async::QueryWithParams;
    use mysql_async::Value;

    use canyon_connection::canyon_database_connector::DatabaseConnection;

    use crate::bounds::QueryParameter;
    use crate::rows::CanyonRows;
    use mysql_async::Row;
    use mysql_common::constants::ColumnType;
    use mysql_common::row;

    use super::reorder_params;
    use crate::crud::{DETECT_PARAMS_IN_QUERY, DETECT_QUOTE_IN_QUERY};
    use regex::Regex;

    pub async fn launch<'a, T>(
        db_conn: &DatabaseConnection,
        stmt: String,
        params: &'a [&'_ dyn QueryParameter<'_>],
    ) -> Result<CanyonRows<T>, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
        let mysql_connection = db_conn.mysql_connection().client.get_conn().await?;

        let stmt_with_escape_characters = regex::escape(&stmt);
        let query_string =
            Regex::new(DETECT_PARAMS_IN_QUERY)?.replace_all(&stmt_with_escape_characters, "?");

        let mut query_string = Regex::new(DETECT_QUOTE_IN_QUERY)?
            .replace_all(&query_string, "")
            .to_string();

        let mut is_insert = false;
        if let Some(index_start_clausule_returning) = query_string.find(" RETURNING") {
            query_string.truncate(index_start_clausule_returning);
            is_insert = true;
        }

        let params_query: Vec<Value> =
            reorder_params(&stmt, params, |f| f.as_mysql_param().to_value());

        let query_with_params = QueryWithParams {
            query: query_string,
            params: params_query,
        };

        let mut query_result = query_with_params
            .run(mysql_connection)
            .await
            .expect("Error executing query in mysql");

        let result_rows = if is_insert {
            let last_insert = query_result
                .last_insert_id()
                .map(Value::UInt)
                .expect("Error getting pk id in insert");

            vec![row::new_row(
                vec![last_insert],
                Arc::new([mysql_async::Column::new(ColumnType::MYSQL_TYPE_UNKNOWN)]),
            )]
        } else {
            query_result
                .collect::<Row>()
                .await
                .expect("Error resolved trait FromRow in mysql")
        };

        Ok(CanyonRows::MySQL(result_rows))
    }
}

#[cfg(feature = "mysql")]
fn reorder_params<T>(
    stmt: &str,
    params: &[&'_ dyn QueryParameter<'_>],
    fn_parser: impl Fn(&&dyn QueryParameter<'_>) -> T,
) -> Vec<T> {
    let mut ordered_params = vec![];
    let rg = regex::Regex::new(DETECT_PARAMS_IN_QUERY).expect(
        format!(
            "Error create regex with detect params pattern expression: {:?} ",
            DETECT_PARAMS_IN_QUERY
        )
        .as_str(),
    );

    for positional_param in rg.find_iter(stmt) {
        let pp: &str = positional_param.as_str();
        let pp_index = pp[1..] // param $1 -> get 1
            .parse::<usize>()
            .expect("Error parse mapped parameter to usized.")
            - 1;

        let element = params
            .get(pp_index)
            .expect("Error obtaining the element of the mapping against parameters.");
        ordered_params.push(fn_parser(element));
    }

    ordered_params
}
