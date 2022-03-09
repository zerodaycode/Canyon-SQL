use std::fmt::Debug;
use async_trait::async_trait;
use tokio_postgres::{ToStatement, types::ToSql};

use crate::{connector::DatabaseConnection, results::DatabaseResult};

#[async_trait]
pub trait Transaction<T: Debug> {
    /// Performs the necessary to execute a query against the database
    async fn query<Q>(stmt: &Q, params: &[&(dyn ToSql + Sync)]) -> DatabaseResult<T> 
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
            .expect("An error querying the database happened.")
        
        )
    }
}
#[async_trait]
pub trait CrudOperations<T: Debug>: Transaction<T> {


    /// The implementation of the most basic database usage pattern.
    /// Given a table name, extracts all db records for the table
    /// 
    /// If not columns provided, performs a SELECT *, else, will query only the 
    /// desired columns
    async fn __find_all(table_name: &str, columns: &[&str]) -> DatabaseResult<T> {

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

        Self::query(&sql[..], &[]).await
    }

    /// Queries the database and try to find an item on the most common pk
    async fn __find_by_id(table_name: &str, id: i32) -> DatabaseResult<T> {

        let stmt = format!("SELECT * FROM {} WHERE id = $1", table_name);

        Self::query(&stmt[..], &[&id]).await
    }

    /// Inserts the values of structure in the correlative table
    async fn __insert(table_name: &str, fields: &str, values: &[&(dyn ToSql + Sync)]) -> DatabaseResult<T> {

        let mut field_values = String::new();
        // Construct the String that holds the '$1' placeholders for the values to insert
        let total_values = values.len();
        for num in 1..total_values {
            if num < total_values - 1 {
                field_values.push_str(&("$".to_owned() + &num.to_string() + ","));
            } else {
                field_values.push_str(&("$".to_owned() + &num.to_string()));
            }
        }

        // Removes the id from the insert operation
        let mut fields_without_id_chars = fields.chars();
        fields_without_id_chars.next();
        fields_without_id_chars.next();
        fields_without_id_chars.next();
        fields_without_id_chars.next();

        let stmt = format!(
            "INSERT INTO {} ({}) VALUES ({})", 
            table_name, fields_without_id_chars.as_str(), field_values
        );

        println!("\nINSERT STMT: {}", &stmt);
        println!("FIELDS: {}", &fields);
        
        Self::query(
            &stmt[..], 
            &values[1..]
        ).await
    }


    /// Deletes the entrity from the database that belongs to a current instance
    async fn __delete(table_name: &str, id: i32) -> DatabaseResult<T> {
        
        let stmt = format!("DELETE FROM {} WHERE id = $1", table_name);

        Self::query(&stmt[..], &[&id]).await
    }
}

 