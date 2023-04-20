#![allow(clippy::nonminimal_bool)]

#[cfg(feature = "mssql")]
use crate::constants::SQL_SERVER_DS;
///! Integration tests for the CRUD operations available in `Canyon` that
///! generates and executes *SELECT* statements
use crate::Error;
use canyon_sql::crud::CrudOperations;

use crate::tests_models::league::*;
use crate::tests_models::player::*;

/// Tests the behaviour of a SELECT * FROM {table_name} within Canyon, through the
/// `::find_all()` associated function derived with the `CanyonCrud` derive proc-macro
/// and using the *default datasource*
#[cfg(feature = "postgres")]
#[canyon_sql::macros::canyon_tokio_test]
fn test_crud_find_all() {
    let find_all_result: Result<Vec<League>, Box<dyn Error + Send + Sync>> =
        League::find_all().await;

    // Connection doesn't return an error
    assert!(!find_all_result.is_err());
    assert!(!find_all_result.unwrap().is_empty());

    let find_all_players: Result<Vec<Player>, Box<dyn Error + Send + Sync>> =
        Player::find_all().await;
    assert!(!find_all_players.unwrap().is_empty());
}

/// Same as the `find_all()`, but with the unchecked variant, which directly returns `Vec<T>` not
/// `Result` wrapped
#[cfg(feature = "postgres")]
#[canyon_sql::macros::canyon_tokio_test]
fn test_crud_find_all_unchecked() {
    let find_all_result: Vec<League> = League::find_all_unchecked().await;
    assert!(!find_all_result.is_empty());
}

/// Tests the behaviour of a SELECT * FROM {table_name} within Canyon, through the
/// `::find_all()` associated function derived with the `CanyonCrud` derive proc-macro
/// and using the specified datasource
#[cfg(feature = "mssql")]
#[canyon_sql::macros::canyon_tokio_test]
fn test_crud_find_all_datasource() {
    let find_all_result: Result<Vec<League>, Box<dyn Error + Send + Sync>> =
        League::find_all_datasource(SQL_SERVER_DS).await;
    // Connection doesn't return an error
    assert!(!find_all_result.is_err());
    assert!(!find_all_result.unwrap().is_empty());
}

/// Same as the `find_all_datasource()`, but with the unchecked variant and the specified dataosource,
/// returning directly `Vec<T>` and not `Result<Vec<T>, Err>`
#[cfg(feature = "mssql")]
#[canyon_sql::macros::canyon_tokio_test]
fn test_crud_find_all_unchecked_datasource() {
    let find_all_result: Vec<League> = League::find_all_unchecked_datasource(SQL_SERVER_DS).await;
    assert!(!find_all_result.is_empty());
}

/// Tests the behaviour of a SELECT * FROM {table_name} WHERE <pk> = <pk_value>, where the pk is
/// defined with the #[primary_key] attribute over some field of the type.
///
/// Uses the *default datasource*.
#[cfg(feature = "postgres")]
#[canyon_sql::macros::canyon_tokio_test]
fn test_crud_find_by_pk() {
    let find_by_pk_result: Result<Option<League>, Box<dyn Error + Send + Sync>> =
        League::find_by_pk(&1).await;
    assert!(find_by_pk_result.as_ref().unwrap().is_some());

    let some_league = find_by_pk_result.unwrap().unwrap();
    assert_eq!(some_league.id, 1);
    assert_eq!(some_league.ext_id, 100695891328981122_i64);
    assert_eq!(some_league.slug, "european-masters");
    assert_eq!(some_league.name, "European Masters");
    assert_eq!(some_league.region, "EUROPE");
    assert_eq!(
        some_league.image_url,
        "http://static.lolesports.com/leagues/EM_Bug_Outline1.png"
    );
}

/// Tests the behaviour of a SELECT * FROM {table_name} WHERE <pk> = <pk_value>, where the pk is
/// defined with the #[primary_key] attribute over some field of the type.
///
/// Uses the *specified datasource* in the second parameter of the function call.
#[cfg(feature = "postgres")]
#[cfg(feature = "mssql")]
#[canyon_sql::macros::canyon_tokio_test]
fn test_crud_find_by_pk_datasource() {
    let find_by_pk_result: Result<Option<League>, Box<dyn Error + Send + Sync>> =
        League::find_by_pk_datasource(&27, SQL_SERVER_DS).await;
    assert!(find_by_pk_result.as_ref().unwrap().is_some());

    let some_league = find_by_pk_result.unwrap().unwrap();
    assert_eq!(some_league.id, 27);
    assert_eq!(some_league.ext_id, 107898214974993351_i64);
    assert_eq!(some_league.slug, "college_championship");
    assert_eq!(some_league.name, "College Championship");
    assert_eq!(some_league.region, "NORTH AMERICA");
    assert_eq!(
        some_league.image_url,
        "http://static.lolesports.com/leagues/1646396098648_CollegeChampionshiplogo.png"
    );
}

/// Counts how many rows contains an entity on the target database.
#[cfg(feature = "postgres")]
#[canyon_sql::macros::canyon_tokio_test]
fn test_crud_count_operation() {
    assert_eq!(
        League::find_all().await.unwrap().len() as i64,
        League::count().await.unwrap()
    );
}

/// Counts how many rows contains an entity on the target database using
/// the specified datasource
#[cfg(feature = "mssql")]
#[canyon_sql::macros::canyon_tokio_test]
fn test_crud_count_datasource_operation() {
    assert_eq!(
        League::find_all_datasource(SQL_SERVER_DS)
            .await
            .unwrap()
            .len() as i64,
        League::count_datasource(SQL_SERVER_DS).await.unwrap()
    );
}
