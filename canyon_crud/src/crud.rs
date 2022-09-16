use std::fmt::Debug;

use async_trait::async_trait;
use canyon_connection::canyon_database_connector::DatabaseType;
use tokio_postgres::{ToStatement, types::ToSql, Error};

use crate::{mapper::RowMapper, bounds::PrimaryKey};
use crate::result::DatabaseResult;
use crate::query_elements::query::Query;
use crate::query_elements::query_builder::QueryBuilder;

use canyon_connection::{
    DATASOURCES,
    DEFAULT_DATASOURCE,
    canyon_database_connector::DatabaseConnection, 
};

trait ToCanyonSql {}
impl ToCanyonSql for dyn tokio_postgres::types::ToSql {}


/// This traits defines and implements a query against a database given
/// an statemt `stmt` and the params to pass the to the client.
/// 
/// It returns a [`DatabaseResult`], which is the core Canyon type to wrap
/// the result of the query and, if the user desires,
/// automatically map it to an struct.
#[async_trait]
pub trait Transaction<T: Debug> {
    #[allow(unreachable_code)]
    /// Performs the necessary to execute a query against the database
    async fn query<'a, Q>(stmt: &Q, params: &[&(dyn ToSql + Sync)], datasource_name: &'a str) -> Result<DatabaseResult<T>, Error> 
        where Q: ?Sized + ToStatement + Sync
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
            todo!(); // panic for now, we must change the return type of all the Crud methods
            // and the return types expected on the macros 
            // return Err(db_conn);
            // Box<(dyn std::error::Error + Send + Sync + 'static)>
        } else {
            // No errors
            let db_conn = database_connection.ok().unwrap();
             
            match db_conn.database_type {
                DatabaseType::PostgreSql => 
                    postgres_query_launcher::launch::<Q, T>(db_conn, stmt, params).await,
                DatabaseType::SqlServer => todo!()
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
    async fn __find_all(table_name: &str, datasource_name: &str) -> Result<DatabaseResult<T>, Error> {
        let stmt = format!("SELECT * FROM {}", table_name);
        Self::query(&stmt[..], &[], datasource_name).await
    }

    fn __find_all_query<'a>(table_name: &str, datasource_name: &'a str) -> QueryBuilder<'a, T> {
        Query::new(format!("SELECT * FROM {}", table_name), &[], datasource_name)
    }

    /// Queries the database and try to find an item on the most common pk
    async fn __find_by_pk<P>(table_name: &str, pk: &str, pk_value: P, datasource_name: &str) 
        -> Result<DatabaseResult<T>, Error> where P: PrimaryKey
    {
        let stmt = format!("SELECT * FROM {} WHERE {} = $1", table_name, pk);
        Self::query(&stmt[..], &[&pk_value], datasource_name).await
    }

    /// Counts the total entries (rows) of elements of a database table
    async fn __count(table_name: &str, datasource_name: &str) -> Result<i64, Error> {
        let count = Self::query(
            &format!("SELECT COUNT (*) FROM {}", table_name)[..], 
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
    async fn __insert<'a, P: PrimaryKey>(
        table_name: &'a str,
        primary_key: &'a str,
        fields: &'a str,
        values: &'a[&'a(dyn ToSql + Sync)],
        datasource_name: &'a str
    ) -> Result<DatabaseResult<T>, Error> {
        // Making sense of the primary_key attributte.
        let mut fields = fields.to_string();
        let mut values = values.to_vec();
        if primary_key != "" { 
            crud_algorythms::manage_primary_key(primary_key, &mut fields, &mut values)
        }
        // Converting the fields column names to case-insensitive
        fields = fields
            .split(", ")
            .map( |column_name| format!("\"{}\"", column_name))
            .collect::<Vec<String>>()
            .join(", ");

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
            &stmt[..], 
            &values,
            datasource_name
        ).await;

        if let Err(error) = result {
            Err(error)
        } else {
            Ok(result.ok().unwrap())
        }
    }

    /// Same as the [`fn@__insert`], but as an associated function of some T type.
    async fn __insert_multi<'a, P: PrimaryKey>(
        table_name: &'a str,
        primary_key: &'a str,
        fields: &'a str, 
        values_arr: &'a mut Vec<Vec<Box<&'a (dyn ToSql + Sync)>>>,
        datasource_name: &'a str
    ) -> Result<DatabaseResult<T>, Error> {

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

        // Converts the array of array of values in an array of correlated values
        // with it's correspondents $X
        let mut values: Vec<&(dyn ToSql + Sync)> = Vec::new();
        for arr in values_arr.into_iter() { 
            for value in arr.into_iter() {
                values.push(*(*value).to_owned());
            }
        };

        let result = Self::query(
            &stmt[..], 
            &values[..],
            datasource_name
        ).await;

        if let Err(error) = result {
            Err(error)
        } else { Ok(result.ok().unwrap()) }
    }

    /// Updates an entity from the database that belongs to a current instance
    async fn __update(
        table_name: &str,
        primary_key: &str,
        fields: &str, 
        values: &[&(dyn ToSql + Sync)],
        datasource_name: &str
    ) -> Result<DatabaseResult<T>, Box<(dyn std::error::Error + Send + Sync + 'static)>> {

        // TODO Detatch this error from here and just not compile the macro
        if primary_key == "" {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    "Canyon does not allow the use of the `.update(&self)` method \
                    on entities that does not contains a primary key. \
                    Please, use instead the T::update(...) associated function \
                    provided as a QueryBuilder."
                ).into_inner().unwrap()
            )
        }
        
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
            &stmt[..], 
            values,
            datasource_name
        ).await;

        if let Err(error) = result {
            Err(error.into_source().unwrap())
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
    async fn __delete<'a, P>(
        table_name: &str, 
        pk_column_name: &'a str, 
        pk_value: P, 
        datasource_name: &str
    ) -> Result<DatabaseResult<T>, Box<(dyn std::error::Error + Send + Sync + 'static)>> 
        where P: PrimaryKey
    {
        let stmt = format!("DELETE FROM {} WHERE {:?} = $1", table_name, pk_column_name);

        let result = Self::query(
            &stmt[..], 
            &[&pk_value],
            datasource_name
        ).await;

        if let Err(error) = result {
            Err(error.into_source().unwrap())
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
    async fn __search_by_foreign_key(
        related_table: &str, 
        related_column: &str,
        lookage_value: &str,
        datasource_name: &str
    ) -> Result<DatabaseResult<T>, Error> {

        let stmt = format!(
            "SELECT * FROM {} WHERE {} = {}", 
            related_table,
            related_table.to_owned() + "." + "\"" + related_column + "\"",
            lookage_value
        );

        let result = Self::query(
            &stmt[..], 
            &[],
            datasource_name
        ).await;

        if let Err(error) = result {
            Err(error)
        } else { Ok(result.ok().unwrap()) }
    }

    /// Performs a search over the side that contains the ForeignKey annotation
    async fn __search_by_reverse_side_foreign_key(
        table: &str, 
        column: &str,
        lookage_value: String,
        datasource_name: &str
    ) -> Result<DatabaseResult<T>, Error> {

        let stmt = format!(
            "SELECT * FROM {} WHERE \"{}\" = {}", 
            table,
            column,
            lookage_value
        );

        let result = Self::query(
            &stmt[..], 
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
    use tokio_postgres::types::ToSql;

    /// Operates over the data of the insert operations to generate the insert
    /// SQL depending of it's a `primary_key` annotation, if it's setted as 
    /// autogenerated (discards the pk field and value from the insert)
    pub fn manage_primary_key<'a>(
        primary_key: &'a str,
        fields: &'a mut String,
        values: &'a mut Vec<&(dyn ToSql + Sync)>
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
    use tokio_postgres::{Error, types::ToSql, ToStatement};
    use crate::result::DatabaseResult;

    pub async fn launch<Q, T>(
        db_conn: DatabaseConnection,
        stmt: &Q,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<DatabaseResult<T>, Error> 
        where Q: ?Sized + ToStatement + Sync, T: Debug
    {
        let postgres_connection = db_conn.postgres_connection.unwrap();
        let (client, connection) =
            (postgres_connection.client, postgres_connection.connection);

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("An error occured while trying to connect to the database: {}", e);
            }
        });

        let query_result = client.query(stmt.into(), params).await;

        if let Err(error) = query_result { Err(error) } else {
            Ok(
                DatabaseResult::new(query_result.expect("A really bad error happened"))
            )
        }
    }
}

mod sqlserver_query_launcher {
    use std::{fmt::Debug, borrow::Cow};
    use canyon_connection::canyon_database_connector::DatabaseConnection;
    use crate::{canyon_connection::tiberius::*, result::DatabaseResult};

    use super::ToCanyonSql;

    trait NewTrait: ToCanyonSql + for<'r> canyon_connection::tiberius::IntoSql<'r> {}

    pub async fn launch_sqlserver_query<'a, Q, T>(
        db_conn: DatabaseConnection,
        stmt: Q,
        params: &[&(impl IntoSql<'a> + 'a + Clone)],
    ) -> Result<DatabaseResult<T>> 
        where Q: Into<Cow<'a, str>> + From<&'a Q> + 'a, T: Debug
    {
        let mut sql_server_query = Query::new(stmt);
        params.into_iter().for_each( |param| sql_server_query.bind( (**param).clone()) );

        let client = &mut db_conn.sqlserver_connection
            .expect("Error querying the SqlServer database") // TODO Better msg
            .client;

        let results = sql_server_query.query(client).await?
            .into_results().await?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        Ok(DatabaseResult::new(vec![]))
    }
}