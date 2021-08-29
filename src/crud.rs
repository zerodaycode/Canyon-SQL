use std::fmt::Debug;

use async_trait::async_trait;

use tokio_postgres::{ToStatement, types::ToSql};

use crate::{connector::DatabaseConnection, results::DatabaseResult};


#[async_trait]
pub trait CrudOperations<T: Debug> {
    
    /// Performs the necessary to execute a query against the database
    async fn query<Q>(&self, stmt: &Q, params: &[&(dyn ToSql + Sync)]) -> DatabaseResult<T> 
    where Q: ?Sized + ToStatement + Sync
    {
        let database_connection = 
            DatabaseConnection::new().await.unwrap();

        let (client, connection) =
            (database_connection.client, database_connection.connection);

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("An error occured while trying to connect to the database: {}", e);
            }
        });

        DatabaseResult::new(
            client.query(
                stmt.into(), 
                params
            )
            .await
            .expect("Error trying to find all the records")
        
        )
    }

    /// The implementation of the most basic database usage pattern.
    /// Given a table name, extracts all db records for the table
    /// 
    /// If not columns provided, performs a SELECT *, else, will query only the 
    /// desired columns
    async fn find_all(&self, table_name: &str, columns: &[&str]) -> DatabaseResult<T> {

        let sql: String = if columns.len() == 0 { // Care, conditional assignment
            String::from(format!("SELECT * FROM {}", table_name))
        } else {
            let mut table_columns = String::new();
            
            let mut counter = 0;
            while counter < columns.len() - 1 {
                table_columns.push_str(
                    (columns.get(counter).unwrap().to_string() + ", ").as_str()
                );
                counter += 1;
            }

            table_columns.push_str(columns.get(counter).unwrap());

            let query = String::from(
                format!("SELECT {} FROM {}", table_columns, table_name
            ));

            query
        };

        self.query(
            &sql[..], 
            &[]
        ).await
        
    }

    /// Queries the database and try to find an item on the most common pk
    async fn find_by_id(&self, table_name: &str, id: i32) -> DatabaseResult<T> {

        let stmt = format!("SELECT * FROM {} WHERE id = $1", table_name);

        self.query(&stmt[..], &[&id]).await
    }
}

