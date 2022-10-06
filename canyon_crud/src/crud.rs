use std::fmt::{Debug, Display};

use async_trait::async_trait;
use canyon_connection::canyon_database_connector::DatabaseType;

use crate::{bounds::QueryParameters, query_elements::query_builder::QueryBuilder};
use crate::mapper::RowMapper;
use crate::result::DatabaseResult;

use canyon_connection::{
    DATASOURCES,
    DEFAULT_DATASOURCE,
    canyon_database_connector::DatabaseConnection, 
};


/// This traits defines and implements a query against a database given
/// an statemt `stmt` and the params to pass the to the client.
/// 
/// It returns a [`DatabaseResult`], which is the core Canyon type to wrap
/// the result of the query and, if the user desires,
/// automatically map it to an struct.
#[async_trait]
pub trait Transaction<T: Debug> {
    /// Performs the necessary to execute a query against the database
    async fn query<'a, S, Z>(stmt: S, params: Z, datasource_name: &'a str) 
        -> Result<DatabaseResult<T>, Box<(dyn std::error::Error + Sync + Send + 'static)>>
        where
            S: AsRef<str> + Display + Sync + Send + 'a, 
            Z: AsRef<[&'a dyn QueryParameters<'a>]> + Sync + Send + 'a
    {
        let database_connection = if datasource_name == "" {
            DatabaseConnection::new(&DEFAULT_DATASOURCE.properties).await
        } else { // Get the specified one
            DatabaseConnection::new(
                &DATASOURCES.iter()
                .find( |ds| ds.name == datasource_name)
                .expect(&format!("No datasource found with the specified parameter: `{}`", datasource_name))
                .properties
            ).await
        };

        if let Err(_db_conn) = database_connection {
            todo!();
        } else {
            // No errors
            let db_conn = database_connection.ok().unwrap();

            match db_conn.database_type {
                DatabaseType::PostgreSql => 
                    postgres_query_launcher::launch::<T>(db_conn, stmt.to_string(), params.as_ref()).await,
                DatabaseType::SqlServer =>
                    sqlserver_query_launcher::launch::<T, Z>(db_conn, &mut stmt.to_string(), params).await
            }
        }
    }
}

/// [`CrudOperations`] it's one of the core parts of Canyon.
/// 
/// Here it's defined and implemented every CRUD operation that Canyon
/// makes available to the user, directly derived with a `CanyonCrud`
/// derive macro when a struct contains the annotation.
/// 
/// Also, this traits needs that the type T over what it's generified 
/// to implement certain types in order to work correctly.
/// 
/// The most notorious one it's the [`RowMapper<T>`] one, which allows
/// Canyon to directly maps database results into structs.
/// 
/// See it's definition and docs to see the real implications.
/// Also, you can find the written macro-code that performs the auto-mapping
/// in the [`canyon_macros`] crates, on the root of this project. 
#[async_trait]
pub trait CrudOperations<T>: Transaction<T> 
    where T: Debug + CrudOperations<T> + RowMapper<T>
{
    async fn find_all<'a>() -> Result<Vec<T>, Box<(dyn std::error::Error + Send + Sync + 'static)>>;
    
    async fn find_all_datasource<'a>(datasource_name: &'a str) -> Result<Vec<T>, Box<(dyn std::error::Error + Send + Sync + 'static)>>;
    
    async fn find_all_unchecked<'a>() -> Vec<T>;
    
    async fn find_all_unchecked_datasource<'a>(datasource_name: &'a str) -> Vec<T>;

    fn find_all_query<'a>() -> QueryBuilder<'a, T>;
    
    fn find_all_query_datasource<'a>(datasource_name: &'a str) -> QueryBuilder<'a, T>;
    
    async fn count() -> Result<i64, Box<(dyn std::error::Error + Send + Sync + 'static)>>;
    
    async fn count_datasource<'a>(datasource_name: &'a str) -> Result<i64, Box<(dyn std::error::Error + Send + Sync + 'static)>>;

    async fn find_by_pk<'a>(value: &'a dyn QueryParameters<'a>)
        -> Result<Option<T>, Box<(dyn std::error::Error + Send + Sync + 'static)>>;
    
    async fn find_by_pk_datasource<'a>(
        value: &'a dyn QueryParameters<'a>,
        datasource_name: &'a str
    ) -> Result<Option<T>, Box<(dyn std::error::Error + Send + Sync + 'static)>>;

    async fn insert<'a>(&mut self) 
        -> Result<(), Box<dyn std::error::Error + Sync + std::marker::Send>>;

    async fn insert_datasource<'a>(&mut self, datasource_name: &'a str)
        -> Result<(), Box<dyn std::error::Error + Sync + std::marker::Send>>;

    async fn multi_insert<'a>(instances: &'a mut [&'a mut T])
        -> Result<(), Box<(dyn std::error::Error + Send + Sync + 'static)>>;

    async fn multi_insert_datasource<'a>(
        instances: &'a mut [&'a mut T],
        datasource_name: &'a str
    ) -> Result<(), Box<(dyn std::error::Error + Send + Sync + 'static)>>;
    
    async fn update(&self);

    async fn update_datasource<'a>(&self, datasource_name: &'a str);
    
    async fn update_result(&self) 
        -> Result<(), Box<dyn std::error::Error + Sync + std::marker::Send>>;
    
    async fn update_result_datasource<'a>(&self, datasource_name: &'a str) 
        -> Result<(), Box<dyn std::error::Error + Sync + std::marker::Send>>;
    
    fn update_query<'a>() -> QueryBuilder<'a, T>;

    fn update_query_datasource<'a>(datasource_name: &'a str) -> QueryBuilder<'a, T>;

    async fn delete(&self) -> Result<(), Box<dyn std::error::Error + Sync + std::marker::Send>>;

    async fn delete_datasource<'a>(&self, datasource_name: &'a str)
        -> Result<(), Box<dyn std::error::Error + Sync + std::marker::Send>>;
    
    fn delete_query<'a>() -> QueryBuilder<'a, T>;

    fn delete_query_datasource<'a>(datasource_name: &'a str) -> QueryBuilder<'a, T>;

    // /// Performs a search over some table pointed with a ForeignKey annotation
    // async fn __search_by_foreign_key<'a>(
    //     related_table: &'a str, 
    //     related_column: &'a str,
    //     lookage_value: &'a str,
    //     datasource_name: &'a str
    // ) -> Result<DatabaseResult<T>, Box<(dyn std::error::Error + Send + Sync + 'static)>> {

    //     let stmt = format!(
    //         "SELECT * FROM {} WHERE {} = {}", 
    //         related_table,
    //         format!("\"{}\"", related_column).as_str(),
    //         lookage_value
    //     );

    //     let result = Self::query(
    //         stmt, 
    //         &[],
    //         datasource_name
    //     ).await;

    //     if let Err(error) = result {
    //         Err(error)
    //     } else { Ok(result.ok().unwrap()) }
    // }

    // /// Performs a search over the side that contains the ForeignKey annotation
    // async fn __search_by_reverse_side_foreign_key<'a>(
    //     table: &'a str,
    //     column: &'a str,
    //     lookage_value: String,
    //     datasource_name: &'a str
    // ) -> Result<DatabaseResult<T>, Box<(dyn std::error::Error + Send + Sync + 'static)>> {

    //     let stmt = format!(
    //         "SELECT * FROM {} WHERE \"{}\" = {}", 
    //         table,
    //         column,
    //         lookage_value
    //     );

    //     let result = Self::query(
    //         stmt, 
    //         &[],
    //         datasource_name
    //     ).await;

    //     if let Err(error) = result {
    //         Err(error)
    //     } else { Ok(result.ok().unwrap()) }
    // }
}

mod postgres_query_launcher {
    use std::fmt::Debug;
    use canyon_connection::canyon_database_connector::DatabaseConnection;
    use crate::bounds::QueryParameters;
    use crate::result::DatabaseResult;

    pub async fn launch<'a, T>(
        db_conn: DatabaseConnection,
        stmt: String,
        params: &'a [&'a dyn QueryParameters<'_>],
    ) -> Result<DatabaseResult<T>, Box<(dyn std::error::Error + Send + Sync + 'static)>> 
        where 
            T: Debug,

    {
        let postgres_connection = db_conn.postgres_connection.unwrap();
        let (client, connection) =
            (postgres_connection.client, postgres_connection.connection);

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("An error occured while trying to connect to the database: {}", e);
            }
        });

        let mut m_params = Vec::new();
        for param in params {
            m_params.push(param.as_postgres_param());
        }

        let query_result = client.query(&stmt, m_params.as_slice()).await;

        if let Err(error) = query_result { 
            Err(Box::new(error)) 
        } else {
            Ok(DatabaseResult::new_postgresql(query_result.expect("A really bad error happened")))
        }
    }
}

mod sqlserver_query_launcher {
    use std::fmt::Debug;

    use crate::{
        canyon_connection::{
            async_std::net::TcpStream,
            tiberius::{Query, Row, Client},
            canyon_database_connector::DatabaseConnection
        }, 
        result::DatabaseResult, 
        bounds::QueryParameters
    };

    pub async fn launch<'a, T, Z>(
        db_conn: DatabaseConnection,
        stmt: &mut String,
        params: Z,
    ) -> Result<DatabaseResult<T>, Box<(dyn std::error::Error + Send + Sync + 'static)>> 
        where 
            T: Debug,
            Z: AsRef<[&'a dyn QueryParameters<'a>]> + Sync + Send + 'a
    {
        // Re-generate de insert statement to adecuate it to the SQL SERVER syntax to retrieve the PK value(s) after insert
        if stmt.contains("RETURNING") {
            let c = stmt.clone();
            let temp = c.split_once("RETURNING")
                .expect("An error happened generating an INSERT statement for a SQL SERVER client");
            let temp2 = temp.0.split_once("VALUES")
                .expect("An error happened generating an INSERT statement for a SQL SERVER client [1]");

            *stmt = format!("{} OUTPUT inserted.{} VALUES {}", temp2.0.trim(), temp.1.trim(), temp2.1.trim());
        }

        let mut sql_server_query = Query::new(stmt.to_owned().replace("$", "@P"));
        params.as_ref().into_iter().for_each( |param| sql_server_query.bind( *param ));

        let client: &mut Client<TcpStream> = &mut db_conn.sqlserver_connection
            .expect("Error querying the SqlServer database") // TODO Better msg?
            .client;

        let _results: Vec<Row> = sql_server_query.query(client).await?
            .into_results().await?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        Ok(DatabaseResult::new_sqlserver(_results))
    }
}