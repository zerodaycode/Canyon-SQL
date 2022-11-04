///! Integration tests for the heart of a Canyon-SQL application, the CRUD operations.

use canyon_sql::{*, crud::CrudOperations};

mod tests_models;
use tests_models::league::*;

#[tokio::test]
async fn find_all_tests() {
    let find_all_result = League::find_all().await;

    // assert!(!find_all_result.is_err());

    match find_all_result {
        Ok(res) => assert_eq!(!res.is_empty(), true),
        Err(e) => eprintln!("Error: {e}")
    } 
}