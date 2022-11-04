use std::error::Error;

///! Integration tests for the heart of a Canyon-SQL application, the CRUD operations.
/// 
///! This tests will tests mostly the whole source code of Canyon, due to its integration nature
/// 
/// Guide-style: Almost every operation in Canyon is `Result` wrapped (without the) unckecked
/// variants of the `find_all` implementations. We will go to directly `.unwrap()` the results
/// because, if there's something wrong in the code reported by the tests, we want to *panic*
/// and abort the execution.
/// 
/// # TODO We must use, for example, the datasource versions of every CRUD method to test
/// agains *sql server* databases, and use the default datasource for test against *postgresql*

use canyon_sql::{*, crud::CrudOperations};

mod tests_models;
use tests_models::league::*;

const PSQL_DS: &str = "postgres_docker";

#[tokio::test]
/// Tests the behaviour of a SELECT * FROM {table_name} within Canyon, through the
/// `::find_all()` associated function derived with the `CanyonCrud` derive proc-macro
/// and using the *default datasource*
async fn test_crud_find_all() {
    let find_all_result: Result<Vec<League>, Box<dyn Error + Send + Sync>> = League::find_all().await;
    // Connection doesn't return an error
    assert!(!find_all_result.is_err());
    assert!(!find_all_result.unwrap().is_empty()); 
}

#[tokio::test]
/// Same as the `find_all()`, but with the unchecked variant, which directly returns `Vec<T>` not
/// `Result` wrapped
async fn test_crud_find_all_unchecked() {
    let find_all_result: Vec<League> = League::find_all_unchecked().await;
    assert!(!find_all_result.is_empty()); 
}

#[tokio::test]
/// Tests the behaviour of a SELECT * FROM {table_name} within Canyon, through the
/// `::find_all()` associated function derived with the `CanyonCrud` derive proc-macro
/// and using the specified datasource
async fn test_crud_find_all_datasource() {
    let find_all_result: Result<Vec<League>, Box<dyn Error + Send + Sync>> = League::find_all_datasource(PSQL_DS).await;
    // Connection doesn't return an error
    assert!(!find_all_result.is_err());
    assert!(!find_all_result.unwrap().is_empty()); 
}

#[tokio::test]
/// Same as the `find_all_datasource()`, but with the unchecked variant and the specified dataosource,
/// returning directly `Vec<T>` and not `Result<Vec<T>, Err>`
async fn test_crud_find_all_unchecked_datasource() {
    let find_all_result: Vec<League> = League::find_all_unchecked_datasource(PSQL_DS).await;
    assert!(!find_all_result.is_empty()); 
}


#[tokio::test]
/// Tests the behaviour of a SELECT * FROM {table_name} WHERE <pk> = <pk_value>, where the pk is
/// defined with the #[primary_key] attribute over some field of the type.
/// 
/// Uses the *default datasource*.
async fn test_crud_find_by_pk() {
    let find_by_pk_result: Result<Option<League>, Box<dyn Error + Send + Sync>> = League::find_by_pk(&1).await;
    assert!(find_by_pk_result.as_ref().unwrap().is_some());
    
    let some_league = find_by_pk_result.unwrap().unwrap();
    assert_eq!(some_league.id, 1);
    assert_eq!(some_league.ext_id, 100695891328981122 as i64);
    assert_eq!(some_league.slug, "european-masters");
    assert_eq!(some_league.name, "European Masters");
    assert_eq!(some_league.region, "EUROPE");
    assert_eq!(some_league.image_url, "http://static.lolesports.com/leagues/EM_Bug_Outline1.png");
}

#[tokio::test]
/// Tests the behaviour of a SELECT * FROM {table_name} WHERE <pk> = <pk_value>, where the pk is
/// defined with the #[primary_key] attribute over some field of the type.
/// 
/// Uses the *specified datasource* in the second parameter of the function call.
async fn test_crud_find_by_pk_datasource() {
    let find_by_pk_result: Result<Option<League>, Box<dyn Error + Send + Sync>> = League::find_by_pk_datasource(&2, PSQL_DS).await;
    assert!(find_by_pk_result.as_ref().unwrap().is_some());
    
    let some_league = find_by_pk_result.unwrap().unwrap();
    assert_eq!(some_league.id, 2);
    assert_eq!(some_league.ext_id, 101097443346691685 as i64);
    assert_eq!(some_league.slug, "turkey-academy-league");
    assert_eq!(some_league.name, "TAL");
    assert_eq!(some_league.region, "TURKEY");
    assert_eq!(some_league.image_url, "http://static.lolesports.com/leagues/1592516072459_TAL-01-FullonDark.png");
}