use std::{marker::PhantomData, fmt::Debug};

use canyon_connection::{tokio_postgres::Row, tiberius, canyon_database_connector::DatabaseType};

use crate::{mapper::RowMapper, crud::Transaction};

pub trait DatabaseRow {}
impl DatabaseRow for Row {}

/// Represents a database result after a query, by wrapping the `Vec<Row>` types that comes with the
/// results after the query.
/// and providing methods to deserialize this result into a **user defined struct**
#[derive(Debug)]
pub struct DatabaseResult<T: Debug> {
    pub wrapper: Vec<Row>,
    pub sqlserver: Vec<tiberius::Row>,
    pub active_ds: DatabaseType,
    _phantom_data: std::marker::PhantomData<T>
}

impl<T: Debug> DatabaseResult<T> {
    
    pub fn new(result: Vec<Row>) -> Self {
        Self {
            wrapper: result,
            sqlserver: vec![],
            active_ds: DatabaseType::PostgreSql,
            _phantom_data: PhantomData
        }
    }

    pub fn new_sqlserver(results: Vec<tiberius::Row>) -> Self {
        println!("Sql Server new called!");
        Self {
            wrapper: vec![],
            sqlserver: results,
            active_ds: DatabaseType::SqlServer,
            _phantom_data: PhantomData
        }
    }

    /// Returns a Vec<T> filled with instances of the type T.
    /// Z param it's used to constrait the types that can call this method.
    /// 
    /// Also, provides a way to statically call `Z::deserialize` method,
    /// which it's a complex implementation used by the macros to automatically
    /// map database columns into the fields for T.
    pub fn to_entity<Z: RowMapper<T> + Debug>(&self) -> Vec<T>
        where T: Transaction<T> 
    {
        let mut results = Vec::new();
        
        self.wrapper.iter().for_each( |row| {
            results.push( Z::deserialize( row ) )
        });

        results
    }

    /// Returns a Vec<T> filled with instances of the type T.
    /// Z param it's used to constrait the types that can call this method.
    /// 
    /// Also, provides a way to statically call `Z::deserialize` method,
    /// which it's a complex implementation used by the macros to automatically
    /// map database columns into the fields for T.
    pub fn from_sql_server<Z: RowMapper<T> + Debug>(&self) -> Vec<T>
        where T: Transaction<T> 
    {
        println!("FROM Sql Server new called!");
        let mut results = Vec::new();
        
        self.sqlserver.iter().for_each( |row| {
            results.push( Z::deserialize_sqlserver( row ) )
        });

        results
    }

    /// Literally returns the same results as the `tokio::postgres` crate would do.
    pub fn get_results(self) -> Vec<Row> {
        let mut results = Vec::new();
        
        self.wrapper.into_iter().for_each( |row| {
            results.push( row )
        });

        results
    }

    /// Returns how many rows contains the result of the query
    pub fn get_number_of_results(&self) -> i32 {
        self.wrapper.len() as i32
    }
}