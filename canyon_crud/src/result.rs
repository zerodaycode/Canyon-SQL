use std::{marker::PhantomData, fmt::Debug};

use tokio_postgres::Row;

use crate::{mapper::RowMapper, crud::Transaction};

/// Represents a database result after a query, by wrapping the `tokio::postgres` result
/// and providing methods to deserialize this result into a **user defined struct**
#[derive(Debug)]
pub struct DatabaseResult<T: Debug> {
    pub wrapper: Vec<Row>,
    _phantom_data: std::marker::PhantomData<T>
}

impl<T: Debug> DatabaseResult<T> {
    
    pub fn new(result: Vec<Row>) -> Self {
        Self {
            wrapper: result,
            _phantom_data: PhantomData  // type T need to be used
        }
    }

    /// Returns a Vec<T> filled with instances of the type T.
    /// Z param it's used to constrait the types that can call this method.
    /// 
    /// Also, provides a way to statically call `Z::deserialize` method,
    /// which it's a complex implementation used by the macros to automatically
    /// map database columns into the fields for T.
    pub fn to_entity<Z: RowMapper<T> + Debug>(&self) -> Vec<T>
        where T: Transaction<T> {

        let mut results = Vec::new();
        
        self.wrapper.iter().for_each( |row| {
            results.push( Z::deserialize( row ) )
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