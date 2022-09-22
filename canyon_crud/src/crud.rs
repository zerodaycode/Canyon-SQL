use std::fmt::Debug;

use async_trait::async_trait;
use canyon_connection::canyon_database_connector::DatabaseType;

use crate::bounds::QueryParameters;
use crate::mapper::RowMapper;
use crate::result::DatabaseResult;
use crate::query_elements::query::Query;
use crate::query_elements::query_builder::QueryBuilder;

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
    async fn query<'a, Z>(stmt: String, params: Z, datasource_name: &'a str) 
        -> Result<DatabaseResult<T>, Box<(dyn std::error::Error + Sync + Send + 'static)>>
        where Z: AsRef<[&'a dyn QueryParameters<'a>]> + Sync + Send + 'a
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
                    postgres_query_launcher::launch::<T>(db_conn, stmt, params.as_ref()).await,
                DatabaseType::SqlServer =>
                    sqlserver_query_launcher::launch::<T, Z>(db_conn, stmt, params).await
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
    /// The implementation of the most basic database usage pattern.
    /// Given a table name, extracts all db records for the table
    async fn __find_all<'a>(table_name: &'a str, datasource_name: &'a str)
        -> Result<DatabaseResult<T>, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
        let stmt = format!("SELECT * FROM {}", table_name);
        println!(
            "Database type: {:?}", 
            &DATASOURCES.iter()
                .find( |ds| ds.name == datasource_name)
                .expect(&format!("No datasource found with the specified parameter: `{}`", datasource_name))
                .properties
                .db_type
        );
        Self::query(stmt, &[], datasource_name).await
    }

    fn __find_all_query<'a>(table_name: &str, datasource_name: &'a str) -> QueryBuilder<'a, T> {
        Query::new(format!("SELECT * FROM {}", table_name), &[], datasource_name)
    }

    /// Queries the database and try to find an item on the most common pk
    async fn __find_by_pk<'a>(table_name: &'a str, pk: &'a str, pk_value: &'a dyn QueryParameters<'a>, datasource_name: &'a str) 
        -> Result<DatabaseResult<T>, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
        let stmt = format!("SELECT * FROM {} WHERE {} = $1", table_name, pk);
        Self::query(stmt, vec![pk_value], datasource_name).await
    }

    /// Counts the total entries (rows) of elements of a database table
    async fn __count(table_name: &str, datasource_name: &str) -> Result<i64, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
        let count = Self::query(
            format!("SELECT COUNT (*) FROM {}", table_name), 
            &[],
            datasource_name
        ).await;
        
        if let Err(error) = count {
            Err(error)
        } else {
            Ok(count.ok().unwrap().wrapper.get(0).unwrap().get("count"))
        }
    }

    /// Inserts the values of an structure in the desired table
    /// 
    /// The insert operation over some type it's primary key agnostic unless
    /// there's a primary key in the model (and if it's configured as autoincremental). 
    /// So, if there's a slice different of he empty one, we must remove the primary key value.
    /// 
    /// When it's called over T, gets all data on every field that T has but,
    ///  removing the pk field, because the insert operation by default in Canyon leads to a place 
    /// where the primary key it's created by the database as a unique element being
    /// autoincremental for every new record inserted on the table, if the attribute
    /// is configured to support this case. If there's no PK, or it's configured as NOT autoincremental,
    /// just performns an insert.
    async fn __insert<'a>(
        table_name: &'a str,
        primary_key: &'a str,
        fields: &'a str,
        params: &'a[&'a dyn QueryParameters<'a>],
        datasource_name: &'a str
    ) -> Result<DatabaseResult<T>, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
        println!("HEre primo!");
        
        // Making sense of the primary_key attributte.
        let mut fields = fields.to_string();
        let mut values = params.to_vec();

        if primary_key != "" { 
            let mut splitted = fields.split(", ")
                .map( |column_name| format!("\"{}\"", column_name))
                .collect::<Vec<String>>();

            let index = splitted.iter()
                .position(|pk| *pk == format!("\"{primary_key}\""))
                .unwrap();
            values.remove(index);

            splitted.retain(|pk| *pk != format!("\"{primary_key}\""));
            fields = splitted.join(", ").to_string();
        } else {
            // Converting the fields column names to case-insensitive
            fields = fields
                .split(", ")
                .map( |column_name| format!("\"{}\"", column_name))
                .collect::<Vec<String>>()
                .join(", ");
        }

        let mut field_values_placeholders = String::new();
        crud_algorythms::generate_insert_placeholders(&mut field_values_placeholders, &values.len());

        let stmt = format!(
            "INSERT INTO {} ({}) VALUES ({}) RETURNING {}", 
            table_name, 
            fields, 
            field_values_placeholders,
            primary_key
        );

        let result = Self::query(
            stmt, 
            values,
            datasource_name
        ).await;

        if let Err(error) = result {
            Err(error)
        } else {
            Ok(result.ok().unwrap())
        }
    }

    /// Same as the [`fn@__insert`], but as an associated function of some T type.
    async fn __insert_multi<'a>(
        table_name: &'a str,
        primary_key: &'a str,
        fields: &'a str, 
        values_arr: &mut Vec<Vec<&'a dyn QueryParameters<'a>>>,
        datasource_name: &'a str
    ) -> Result<DatabaseResult<T>, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
        
        // Removes the pk from the insert operation if there's some
        // autogenerated primary_key on the table
        let mut fields = fields.to_string();
        // Converting the fields column names to case-insensitive
        fields = fields
            .split(", ")
            .map( |column_name| format!("\"{}\"", column_name))
            .collect::<Vec<String>>()
            .join(", ");

        let mut splitted = fields.split(", ")
            .collect::<Vec<&str>>();
        
        let pk_value_index = splitted.iter()
            .position(|pk| *pk == format!("\"{}\"", primary_key).as_str())
            .expect("Error. No primary key found when should be there");
        splitted.retain(|pk| *pk != format!("\"{}\"", primary_key).as_str());
        fields = splitted.join(", ").to_string();

        let mut fields_values = String::new();

        let mut elements_counter = 0;
        let mut values_counter = 1;
        let values_arr_len = values_arr.len();

        for vector in values_arr.iter_mut() {
            let mut inner_counter = 0;
            fields_values.push('(');
            vector.remove(pk_value_index);
            
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
            "INSERT INTO {} ({}) VALUES {} RETURNING {}", 
            table_name, 
            &fields, 
            fields_values,
            primary_key
        );

        let mut v_arr = Vec::new();
        for arr in values_arr.iter() {
            for value in arr {
                v_arr.push(*value)
            }
        }

        let result = Self::query(
            stmt, 
            v_arr,
            datasource_name
        ).await;

        if let Err(error) = result {
            Err(error)
        } else { Ok(result.ok().unwrap()) }
    }

    /// Updates an entity from the database that belongs to a current instance
    async fn __update<'a>(
        table_name: &'a str,
        primary_key: &'a str,
        fields: &'a str, 
        values: &'a [&'a dyn QueryParameters<'a>],
        datasource_name: &'a str
    ) -> Result<DatabaseResult<T>, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
        let mut vec_columns_values:Vec<String> = Vec::new();
        
        for (i, column_name) in fields.split(", ").enumerate() {
            let column_equal_value = format!(
                "\"{}\" = ${}", column_name.to_owned(), i + 1
            );
            vec_columns_values.push(column_equal_value)
        }

        let pk_index = fields.split(", ")
            .collect::<Vec<&str>>()
            .iter()
            .position(|pk| *pk == primary_key)
            .unwrap();

        vec_columns_values.remove(pk_index);

        let str_columns_values = vec_columns_values.join(",");

        let stmt = format!(
            "UPDATE {} SET {} WHERE {} = ${:?}",
            table_name, str_columns_values, primary_key, pk_index + 1
        );

        let result = Self::query(
            stmt, 
            values.to_vec(),
            datasource_name
        ).await;

        if let Err(error) = result {
            Err(error)
        } else { Ok(result.ok().unwrap()) }
    }

    /// Performns an UPDATE CRUD operation over some table. It is constructed
    /// as a [QueryBuilder], so the conditions will be appended with the builder
    /// if the user desires
    /// 
    /// Implemented as an associated function, not dependent on an instance
    fn __update_query<'a>(table_name: &'a str, datasource_name: &'a str) -> QueryBuilder<'a, T> {
        Query::new(format!("UPDATE {}", table_name), &[], datasource_name)
    }
    
    /// Deletes the entity from the database that belongs to a current instance
    async fn __delete<'a>(
        table_name: &str, 
        pk_column_name: &'a str, 
        pk_value: &'a [&'a dyn QueryParameters<'a>], 
        datasource_name: &'a str
    ) -> Result<DatabaseResult<T>, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
        let stmt = format!("DELETE FROM {} WHERE {:?} = $1", table_name, pk_column_name);

        let result = Self::query(
            stmt, 
            pk_value,
            datasource_name
        ).await;

        if let Err(error) = result {
            Err(error)
        } else { Ok(result.ok().unwrap()) }
    }

    /// Performns a DELETE CRUD operation over some table. It is constructed
    /// as a [QueryBuilder], so the conditions will be appended with the builder
    /// if the user desires
    /// 
    /// Implemented as an associated function, not dependent on an instance
    fn __delete_query<'a>(table_name: &'a str, datasource_name: &'a str) -> QueryBuilder<'a, T> {
        Query::new(format!("DELETE FROM {}", table_name), &[], datasource_name)
    }
    
    /// Performs a search over some table pointed with a ForeignKey annotation
    async fn __search_by_foreign_key<'a>(
        related_table: &'a str, 
        related_column: &'a str,
        lookage_value: &'a str,
        datasource_name: &'a str
    ) -> Result<DatabaseResult<T>, Box<(dyn std::error::Error + Send + Sync + 'static)>> {

        let stmt = format!(
            "SELECT * FROM {} WHERE {} = {}", 
            related_table,
            format!("\"{}\"", related_column).as_str(),
            lookage_value
        );

        let result = Self::query(
            stmt, 
            &[],
            datasource_name
        ).await;

        if let Err(error) = result {
            Err(error)
        } else { Ok(result.ok().unwrap()) }
    }

    /// Performs a search over the side that contains the ForeignKey annotation
    async fn __search_by_reverse_side_foreign_key<'a>(
        table: &'a str,
        column: &'a str,
        lookage_value: String,
        datasource_name: &'a str
    ) -> Result<DatabaseResult<T>, Box<(dyn std::error::Error + Send + Sync + 'static)>> {

        let stmt = format!(
            "SELECT * FROM {} WHERE \"{}\" = {}", 
            table,
            column,
            lookage_value
        );

        println!("Reverse FK: {:?}", &stmt);

        let result = Self::query(
            stmt, 
            &[],
            datasource_name
        ).await;

        if let Err(error) = result {
            Err(error)
        } else { Ok(result.ok().unwrap()) }
    }
}


/// Utilities for adecuating some data coming from macros to the generated SQL
mod crud_algorythms {
    use canyon_connection::{tokio_postgres::types::ToSql, tiberius::IntoSql};

    /// Operates over the data of the insert operations to generate the insert
    /// SQL depending of it's a `primary_key` annotation, if it's setted as 
    /// autogenerated (discards the pk field and value from the insert)
    pub fn _manage_primary_key<'a>(
        primary_key: &'a str,
        fields: &'a mut String,
        values: &'a mut Vec<impl ToSql + IntoSql<'a> + Clone + Sync + Send>
    ) { 
        let mut splitted = fields.split(", ")
            .collect::<Vec<&str>>();
        let index = splitted.iter()
            .position(|pk| *pk == primary_key)
            .unwrap();
        values.remove(index);

        splitted.retain(|pk| *pk != primary_key);
        *fields = splitted.join(", ").to_string();
    }

    /// Construct the String that holds the '$num' placeholders for the values to insert
    pub fn generate_insert_placeholders<'a>(placeholders: &'a mut String, total_values: &usize) {
        for num in 0..*total_values {
            if num < total_values - 1 {
                placeholders.push_str(&("$".to_owned() + &(num + 1).to_string() + ","));
            } else {
                placeholders.push_str(&("$".to_owned() + &(num + 1).to_string()));
            }
        }
    }

    /// Construct the String that holds the '$num' placeholders for the multi values to insert
    pub fn _generate_multi_insert_placeholders<'a>(
        // args
    ) {
        todo!() // TODO impl for unit test
    }
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
        where T: Debug
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
            Ok(DatabaseResult::new(query_result.expect("A really bad error happened")))
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
        stmt: String,
        params: Z,
    ) -> Result<DatabaseResult<T>, Box<(dyn std::error::Error + Send + Sync + 'static)>> 
        where 
            T: Debug,
            Z: AsRef<[&'a dyn QueryParameters<'a>]> + Sync + Send + 'a
    {
        let mut sql_server_query = Query::new(stmt);
        params.as_ref().into_iter().for_each( |param| sql_server_query.bind( *param ));

        let client: &mut Client<TcpStream> = &mut db_conn.sqlserver_connection
            .expect("Error querying the SqlServer database") // TODO Better msg?
            .client;

        let _results: Vec<Row> = sql_server_query.query(client).await?
            .into_results().await?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        println!("Sql Server rows: {:?}", &_results);

        Ok(DatabaseResult::new_sqlserver(_results))
    }
}