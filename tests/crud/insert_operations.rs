///! Integration tests for the CRUD operations available in `Canyon` that
///! generates and executes *INSERT* statements
use canyon_sql::{crud::CrudOperations, runtime::tokio};

use crate::constants::PSQL_DS;
use crate::tests_models::league::*;

#[tokio::test]
/// Inserts a new record on the database, given an entity that is
/// annotated with `#[canyon_entity]` macro over a *T* type.
///
/// For insert a new record on a database, the *insert* operation needs
/// some special requeriments:
/// > - We need a mutable instance of `T`. If the operation complets
/// succesfully, the insert operation will automatically set the autogenerated
/// value for the `primary_key` annotated field in it.
///
/// > - It's considered a good practice to initialize that concrete field with
/// the `Default` trait, because the value on the primary key field will be
/// ignored at the execution time of the insert, and updated with the autogenerated
/// value by the database.
///
/// By default, the `#[primary_key]` annotation means autogenerated and autoincremental.
/// You can configure not autoincremental via macro annotation parameters (please,
/// refer to the docs [here]() for more info.)
///
/// If the type hasn't a `#[primary_key]` annotation, or the annotation contains
/// an argument specifiying not autoincremental behaviour, all the fields will be
/// inserted on the database and no returning value will be placed in any field.   
async fn test_crud_insert_operation() {
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

    // Now, in the `id` field of the instance, we have the autogenerated
    // value for the primary key field, which is id. So, we can query the
    // database again with the find by primary key operation to check if
    // the value was really inserted
    let inserted_league = League::find_by_pk(&new_league.id)
        .await
        .expect("Failed the query to the database")
        .expect("No entity found for the primary key value passed in");

    assert_eq!(new_league.id, inserted_league.id);
}

/// Same as the insert operation above, but targeting the database defined in
/// the specified datasource
#[tokio::test]
async fn test_crud_insert_datasource_operation() {
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
        .insert_datasource(PSQL_DS)
        .await
        .expect("Failed insert datasource operation");

    // Now, in the `id` field of the instance, we have the autogenerated
    // value for the primary key field, which is id. So, we can query the
    // database again with the find by primary key operation to check if
    // the value was really inserted
    let inserted_league = League::find_by_pk(&new_league.id)
        .await
        .expect("Failed the query to the database")
        .expect("No entity found for the primary key value passed in");

    assert_eq!(new_league.id, inserted_league.id);
}

/// The multi insert operation is a shorthand for insert multiple instances of *T*
/// in the database at once.
///
/// It works pretty much the same that the insert operation, with the same behaviour
/// of the `#[primary_key]` annotation over some field. It will auto set the primary
/// key field with the autogenerated value on the database on the insert operation, but
/// for every entity passed in as an array of mutable instances of `T`.
///
/// The instances without `#[primary_key]` inserts all the values on the instaqce fields
/// on the database.
#[tokio::test]
async fn test_crud_multi_insert_operation() {
    let mut new_league_mi: League = League {
        id: Default::default(),
        ext_id: 54376478_i64,
        slug: "some-new-random-league".to_string(),
        name: "Some New Random League".to_string(),
        region: "Unknown".to_string(),
        image_url: "https://what-a-league.io".to_string(),
    };
    let mut new_league_mi_2: League = League {
        id: Default::default(),
        ext_id: 3475689769678906_i64,
        slug: "new-league-2".to_string(),
        name: "New League 2".to_string(),
        region: "Really unknown".to_string(),
        image_url: "https://what-an-unknown-league.io".to_string(),
    };
    let mut new_league_mi_3: League = League {
        id: Default::default(),
        ext_id: 46756867_i64,
        slug: "a-new-multinsert".to_string(),
        name: "New League 3".to_string(),
        region: "The dark side of the moon".to_string(),
        image_url: "https://interplanetary-league.io".to_string(),
    };

    // Insert the instance as database entities
    new_league_mi
        .insert_datasource(PSQL_DS)
        .await
        .expect("Failed insert datasource operation");
    new_league_mi_2
        .insert_datasource(PSQL_DS)
        .await
        .expect("Failed insert datasource operation");
    new_league_mi_3
        .insert_datasource(PSQL_DS)
        .await
        .expect("Failed insert datasource operation");

    // Recover the inserted data by primary key
    let inserted_league = League::find_by_pk(&new_league_mi.id)
        .await
        .expect("[1] - Failed the query to the database")
        .expect("[1] - No entity found for the primary key value passed in");
    let inserted_league_2 = League::find_by_pk(&new_league_mi_2.id)
        .await
        .expect("[2] - Failed the query to the database")
        .expect("[2] - No entity found for the primary key value passed in");
    let inserted_league_3 = League::find_by_pk(&new_league_mi_3.id)
        .await
        .expect("[3] - Failed the query to the database")
        .expect("[3] - No entity found for the primary key value passed in");

    assert_eq!(new_league_mi.id, inserted_league.id);
    assert_eq!(new_league_mi_2.id, inserted_league_2.id);
    assert_eq!(new_league_mi_3.id, inserted_league_3.id);
}