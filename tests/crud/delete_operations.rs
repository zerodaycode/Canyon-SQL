///! Integration tests for the CRUD operations available in `Canyon` that
///! generates and executes *INSERT* statements
use canyon_sql::crud::CrudOperations;

#[cfg(feature = "mysql")]
use crate::constants::MYSQL_DS;
#[cfg(feature = "postgres")]
use crate::constants::PSQL_DS;
#[cfg(feature = "mssql")]
use crate::constants::SQL_SERVER_DS;

use crate::tests_models::league::*;

/// Deletes a row from the database that is mapped into some instance of a `T` entity.
///
/// The `t.delete(&self)` operation is only enabled for types that
/// has, at least, one of it's fields annotated with a `#[primary_key]`
/// operation, because we use that concrete field to construct the clause that targets
/// that entity.
///
/// Attempt of usage the `t.delete(&self)` method on an entity without `#[primary_key]`
/// will raise a runtime error.
#[cfg(feature = "postgres")]
#[canyon_sql::macros::canyon_tokio_test]
fn test_crud_delete_method_operation() {
    // For test the delete, we will insert a new instance of the database, and then,
    // after inspect it, we will proceed to delete it
    let mut new_league: League = League {
        id: Default::default(),
        ext_id: 7892635306594_i64,
        slug: "some-new-league".to_string(),
        name: "Some New League".to_string(),
        region: "Bahía de cochinos".to_string(),
        image_url: "https://nobodyspectsandimage.io".to_string(),
    };

    // We insert the instance on the database, on the `League` entity
    new_league.insert().await.expect("Failed insert operation");

    assert_eq!(
        new_league.id,
        League::find_by_pk_datasource(&new_league.id, PSQL_DS)
            .await
            .expect("Request error")
            .expect("None value")
            .id
    );

    // Now that we have an instance mapped to some entity by a primary key, we can now
    // remove that entry from the database with the delete operation
    new_league
        .delete()
        .await
        .expect("Failed to delete the operation");

    // To check the success, we can query by the primary key value and check if, after unwrap()
    // the result of the operation, the find by primary key contains Some(v) or None
    // Remember that `find_by_primary_key(&dyn QueryParameter<'a>) -> Result<Option<T>>, Err>
    assert_eq!(
        League::find_by_pk(&new_league.id)
            .await
            .expect("Unwrapping the result, letting the Option<T>"),
        None
    );
}

/// Same as the delete test, but performing the operations with the specified datasource
#[cfg(feature = "mssql")]
#[canyon_sql::macros::canyon_tokio_test]
fn test_crud_delete_datasource_mssql_method_operation() {
    // For test the delete, we will insert a new instance of the database, and then,
    // after inspect it, we will proceed to delete it
    let mut new_league: League = League {
        id: Default::default(),
        ext_id: 7892635306594_i64,
        slug: "some-new-league".to_string(),
        name: "Some New League".to_string(),
        region: "Bahía de cochinos".to_string(),
        image_url: "https://nobodyspectsandimage.io".to_string(),
    };

    // We insert the instance on the database, on the `League` entity
    new_league
        .insert_datasource(SQL_SERVER_DS)
        .await
        .expect("Failed insert operation");
    assert_eq!(
        new_league.id,
        League::find_by_pk_datasource(&new_league.id, SQL_SERVER_DS)
            .await
            .expect("Request error")
            .expect("None value")
            .id
    );

    // Now that we have an instance mapped to some entity by a primary key, we can now
    // remove that entry from the database with the delete operation
    new_league
        .delete_datasource(SQL_SERVER_DS)
        .await
        .expect("Failed to delete the operation");

    // To check the success, we can query by the primary key value and check if, after unwrap()
    // the result of the operation, the find by primary key contains Some(v) or None
    // Remember that `find_by_primary_key(&dyn QueryParameter<'a>) -> Result<Option<T>>, Err>
    assert_eq!(
        League::find_by_pk_datasource(&new_league.id, SQL_SERVER_DS)
            .await
            .expect("Unwrapping the result, letting the Option<T>"),
        None
    );
}

/// Same as the delete test, but performing the operations with the specified datasource
#[cfg(feature = "mysql")]
#[canyon_sql::macros::canyon_tokio_test]
fn test_crud_delete_datasource_mysql_method_operation() {
    // For test the delete, we will insert a new instance of the database, and then,
    // after inspect it, we will proceed to delete it

    let mut new_league: League = League {
        id: Default::default(),
        ext_id: 7892635306594_i64,
        slug: "some-new-league".to_string(),
        name: "Some New League".to_string(),
        region: "Bahía de cochinos".to_string(),
        image_url: "https://nobodyspectsandimage.io".to_string(),
    };

    // We insert the instance on the database, on the `League` entity
    new_league
        .insert_datasource(MYSQL_DS)
        .await
        .expect("Failed insert operation");

    // In mysql now cant get primary key
    let result_delete = new_league.delete_datasource(MYSQL_DS).await;

    // Cant get last insert with need require call in now connection, if call in new connection return zero all time
    assert!(result_delete.is_ok(), "Failed to delete the operation");
}
