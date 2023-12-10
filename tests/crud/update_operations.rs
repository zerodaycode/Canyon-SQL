use crate::tests_models::league::*;
// Integration tests for the CRUD operations available in `Canyon` that
/// generates and executes *UPDATE* statements
use canyon_sql::crud::CrudOperations;

#[cfg(feature = "mysql")]
use crate::constants::MYSQL_DS;
#[cfg(feature = "mssql")]
use crate::constants::SQL_SERVER_DS;

/// Update operation is a *CRUD* method defined for some entity `T`, that works by appliying
/// some change to a Rust's entity instance, and persisting them into the database.
///
/// The `t.update(&self)` operation is only enabled for types that
/// has, at least, one of it's fields annotated with a `#[primary_key]`
/// operation, because we use that concrete field to construct the clause that targets
/// that entity.
///
/// Attempt of usage the `t.update(&self)` method on an entity without `#[primary_key]`
/// will raise a runtime error.
#[cfg(feature = "postgres")]
#[canyon_sql::macros::canyon_tokio_test]
fn test_crud_update_method_operation() {
    // We first retrieve some entity from the database. Note that we must make
    // the retrieved instance mutable of clone it to a new mutable resource
    let mut updt_candidate: League = League::find_by_pk(&1)
        .await
        .expect("[1] - Failed the query to the database")
        .expect("[1] - No entity found for the primary key value passed in");

    // The ext_id field value is extracted from the sql scripts under the
    // docker/sql folder. We are retrieving the first entity inserted at the
    // wake up time of the database, and now checking some of its properties.
    assert_eq!(updt_candidate.ext_id, 100695891328981122_i64);

    // Modify the value, and perform the update
    let updt_value: i64 = 593064_i64;
    updt_candidate.ext_id = updt_value;
    updt_candidate
        .update()
        .await
        .expect("Failed the update operation");

    // Retrieve it again, and check if the value was really updated
    let updt_entity: League = League::find_by_pk(&1)
        .await
        .expect("[2] - Failed the query to the database")
        .expect("[2] - No entity found for the primary key value passed in");

    assert_eq!(updt_entity.ext_id, updt_value);

    // We rollback the changes to the initial value to don't broke other tests
    // the next time that will run
    updt_candidate.ext_id = 100695891328981122_i64;
    updt_candidate
        .update()
        .await
        .expect("Failed the restablish initial value update operation");
}

/// Same as the above test, but with the specified datasource.
#[cfg(feature = "mssql")]
#[canyon_sql::macros::canyon_tokio_test]
fn test_crud_update_datasource_mssql_method_operation() {
    // We first retrieve some entity from the database. Note that we must make
    // the retrieved instance mutable of clone it to a new mutable resource
    let mut updt_candidate: League = League::find_by_pk_datasource(&1, SQL_SERVER_DS)
        .await
        .expect("[1] - Failed the query to the database")
        .expect("[1] - No entity found for the primary key value passed in");

    // The ext_id field value is extracted from the sql scripts under the
    // docker/sql folder. We are retrieving the first entity inserted at the
    // wake up time of the database, and now checking some of its properties.
    assert_eq!(updt_candidate.ext_id, 100695891328981122_i64);

    // Modify the value, and perform the update
    let updt_value: i64 = 59306442534_i64;
    updt_candidate.ext_id = updt_value;
    updt_candidate
        .update_datasource(SQL_SERVER_DS)
        .await
        .expect("Failed the update operation");

    // Retrieve it again, and check if the value was really updated
    let updt_entity: League = League::find_by_pk_datasource(&1, SQL_SERVER_DS)
        .await
        .expect("[2] - Failed the query to the database")
        .expect("[2] - No entity found for the primary key value passed in");

    assert_eq!(updt_entity.ext_id, updt_value);

    // We rollback the changes to the initial value to don't broke other tests
    // the next time that will run
    updt_candidate.ext_id = 100695891328981122_i64;
    updt_candidate
        .update_datasource(SQL_SERVER_DS)
        .await
        .expect("Failed to restablish the initial value update operation");
}

/// Same as the above test, but with the specified datasource.
#[cfg(feature = "mysql")]
#[canyon_sql::macros::canyon_tokio_test]
fn test_crud_update_datasource_mysql_method_operation() {
    // We first retrieve some entity from the database. Note that we must make
    // the retrieved instance mutable of clone it to a new mutable resource

    let mut updt_candidate: League = League::find_by_pk_datasource(&1, MYSQL_DS)
        .await
        .expect("[1] - Failed the query to the database")
        .expect("[1] - No entity found for the primary key value passed in");

    // The ext_id field value is extracted from the sql scripts under the
    // docker/sql folder. We are retrieving the first entity inserted at the
    // wake up time of the database, and now checking some of its properties.
    assert_eq!(updt_candidate.ext_id, 100695891328981122_i64);

    // Modify the value, and perform the update
    let updt_value: i64 = 59306442534_i64;
    updt_candidate.ext_id = updt_value;
    updt_candidate
        .update_datasource(MYSQL_DS)
        .await
        .expect("Failed the update operation");

    // Retrieve it again, and check if the value was really updated
    let updt_entity: League = League::find_by_pk_datasource(&1, MYSQL_DS)
        .await
        .expect("[2] - Failed the query to the database")
        .expect("[2] - No entity found for the primary key value passed in");

    assert_eq!(updt_entity.ext_id, updt_value);

    // We rollback the changes to the initial value to don't broke other tests
    // the next time that will run
    updt_candidate.ext_id = 100695891328981122_i64;
    updt_candidate
        .update_datasource(MYSQL_DS)
        .await
        .expect("Failed to restablish the initial value update operation");
}
