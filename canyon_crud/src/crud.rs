use std::fmt::Debug;

use async_trait::async_trait;
use tokio_postgres::{ToStatement, types::ToSql};

use crate::mapper::RowMapper;
use crate::result::DatabaseResult;
use crate::query_elements::query::Query;
use crate::query_elements::query_builder::QueryBuilder;

use canyon_connection::{
    CREDENTIALS,
    postgresql_connector::DatabaseConnection
};


///! TODO DOCS
#[async_trait]
pub trait Transaction<T: Debug> {
    /// Performs the necessary to execute a query against the database
    async fn query<Q>(stmt: &Q, params: &[&(dyn ToSql + Sync)]) -> DatabaseResult<T> 
        where Q: ?Sized + ToStatement + Sync
    {
        let database_connection = 
            DatabaseConnection::new(&(*CREDENTIALS)).await.unwrap();

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

///! TODO DOCS
#[async_trait]
pub trait CrudOperations<T: Debug + CrudOperations<T> + RowMapper<T>>: Transaction<T> {

    /// The implementation of the most basic database usage pattern.
    /// Given a table name, extracts all db records for the table
    async fn __find_all(table_name: &str) -> DatabaseResult<T> {

        let stmt = format!("SELECT * FROM {}", table_name);

        Self::query(&stmt[..], &[]).await
    }

    fn __find_all_query(table_name: &str) -> QueryBuilder<T> {
        Query::new(format!("SELECT * FROM {}", table_name), &[])
    }

    /// Queries the database and try to find an item on the most common pk
    async fn __find_by_id(table_name: &str, id: i32) -> DatabaseResult<T> {

        let stmt = format!("SELECT * FROM {} WHERE id = $1", table_name);

        Self::query(&stmt[..], &[&id]).await
    }

    async fn __count(table_name: &str) -> i64 {
        let count = Self::query(
            &format!("SELECT COUNT (*) FROM {}", table_name)[..], 
            &[]
        ).await;
        
        // TODO Continue with the impl
        count.wrapper.get(0).unwrap().get("count")
    }

    /// Inserts the values of an structure in the desired table
    /// 
    /// The insert operation over some type it's id agnostic. So, when it's called
    /// over T, gets all data on every field that T has but removing the id field,
    /// because the insert operation by default in Canyon leads to a place 
    /// where the ID it's created by the database as a unique element being
    /// autoincremental for every new record inserted on the table.
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
        
        Self::query(
            &stmt[..], 
            &values[1..]
        ).await
    }

    /// Same as the [`fn@__insert`], but as an associated function of some T type.
    async fn __insert_multi(
        table_name: &str, 
        fields: &str, 
        values_arr: &mut Vec<Vec<Box<&(dyn ToSql + Sync)>>>
    ) -> DatabaseResult<T> {

        // Removes the id from the insert operation
        let mut fields_without_id_chars = fields.chars();
        fields_without_id_chars.next();
        fields_without_id_chars.next();
        fields_without_id_chars.next();
        fields_without_id_chars.next();

        let mut fields_values = String::new();

        let mut elements_counter = 0;
        let mut values_counter = 1;
        let values_arr_len = values_arr.len();

        for vector in values_arr.iter_mut() {
            
            let mut inner_counter = 0;
            fields_values.push('(');
            vector.remove(0); // Removes the $ID value in the container
            
            for _value in vector.iter() {
                if inner_counter < vector.len() - 1 {
                    fields_values.push_str(&("$".to_owned() + &values_counter.to_string() + ","));
                } else {
                    fields_values.push_str(&("$".to_owned() + &values_counter.to_string()));
                }

                inner_counter += 1;
                values_counter += 1;
            }

            elements_counter += 1;

            if elements_counter < values_arr_len {
                fields_values.push_str("), ");
            } else {
                fields_values.push(')');
            }
        }


        let stmt = format!(
            "INSERT INTO {} ({}) VALUES {}", table_name, fields_without_id_chars.as_str(), fields_values
        );

        println!("\nTremenda chambonada: {:?}\n", &stmt);

        // Converts the array of array of values in an array of correlated values
        // with it's correspondents $X
        let mut values: Vec<&(dyn ToSql + Sync)> = Vec::new();
        for arr in values_arr.into_iter() { 
            for value in arr.into_iter() {
                values.push(*(*value).to_owned());
            }
        };

        println!("\nValues: {:?}\n", &values);
        
        Self::query(
            &stmt[..], 
            &values[..]
        ).await
    }

    /// Updates an entity from the database that belongs to a current instance
    async fn __update(table_name: &str, fields: &str, values: &[&(dyn ToSql + Sync)]) -> DatabaseResult<T> {
        
        let mut vec_columns_values:Vec<String> = Vec::new();
        
        for (i, column_name) in fields.split(',').enumerate() {
            let column_equal_value = format!(
                "{} = ${}", column_name.to_owned(), i + 1
            );
            vec_columns_values.push(column_equal_value)
        }

        vec_columns_values.remove(0);
        let str_columns_values = vec_columns_values.join(",");

        let stmt = format!(
            "UPDATE {} SET {} WHERE id = $1",
            table_name, str_columns_values
        );

        Self::query(
            &stmt[..],
            values
        ).await
    }
    
    /// Deletes the entity from the database that belongs to a current instance
    async fn __delete(table_name: &str, id: i32) -> DatabaseResult<T> {
        
        let stmt = format!("DELETE FROM {} WHERE id = $1", table_name);

        Self::query(&stmt[..], &[&id]).await
    }
    
    /// Performs a search over some table pointed with a ForeignKey annotation
    async fn __search_by_foreign_key(
        related_table: &str, 
        related_column: &str,
        lookage_value: &str
    ) -> DatabaseResult<T> {

        let stmt = format!(
            "SELECT * FROM {} WHERE {} = {}", 
            related_table,
            related_table.to_owned() + "." + related_column,
            lookage_value
        );

        Self::query(
            &stmt[..],
            &[]
        ).await
    }

    /// Performs a search over the side that contains the ForeignKey annotation
    async fn __search_by_reverse_side_foreign_key(
        table: &str, 
        column: &str,
        lookage_value: String
    ) -> DatabaseResult<T> {

        let stmt = format!(
            "SELECT * FROM {} WHERE {} = {}", 
            table,
            column,
            lookage_value
        );

        Self::query(
            &stmt[..],
            &[]
        ).await
    }
}

 