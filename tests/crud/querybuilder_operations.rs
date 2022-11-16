///! Tests for the QueryBuilder available operations within Canyon.
///
///! QueryBuilder are the way of obtain more flexibility that with
///! the default generated queries, esentially for build the queries
///! with the SQL filters
///
use canyon_sql::{crud::CrudOperations, runtime::tokio, query::operators::Comp};

use crate::constants::PSQL_DS;
use crate::tests_models::league::*;
use crate::tests_models::player::*;
use crate::tests_models::tournament::*;

/// Builds a new SQL statement for retrieves entities of the `T` type, filtered
/// with the parameters that modifies the base SQL to SELECT * FROM <entity>
#[canyon_sql::macros::canyon_tokio_test]
fn test_crud_find_with_querybuilder() {
    // Find all the leagues with ID less or equals that 7
    // and where it's region column value is equals to 'Korea'
    let filtered_leagues_result: Result<Vec<League>, _> = League::find_query()
        .r#where(LeagueFieldValue::id(50), Comp::Lte)
        .and(LeagueFieldValue::region("KOREA".to_string()), Comp::Eq)
        .query()
        .await;

    let filtered_leagues: Vec<League> = filtered_leagues_result.unwrap();
    assert!(!filtered_leagues.is_empty());

    let league_idx_0 = filtered_leagues.get(0).unwrap();
    assert_eq!(league_idx_0.id, 34);
    assert_eq!(league_idx_0.region, "KOREA");
}

/// Same than the above but with the specified datasource
#[canyon_sql::macros::canyon_tokio_test]
fn test_crud_find_with_querybuilder_datasource() {
    // Find all the players where its ID column value is greater that 50
    let filtered_find_players = Player::find_query_datasource(PSQL_DS)
        .r#where(PlayerFieldValue::id(50), Comp::Gt)
        .query()
        .await;

    assert!(!filtered_find_players.unwrap().is_empty());
}

/// Updates the values of the range on entries defined by the constraint paramenters
/// in the database entity
#[canyon_sql::macros::canyon_tokio_test]
fn test_crud_update_with_querybuilder() {
    // Find all the leagues with ID less or equals that 7
    // and where it's region column value is equals to 'Korea'
    League::update_query()
        .set(&[(LeagueField::slug, "Updated with the QueryBuilder")])
        .r#where(LeagueFieldValue::id(1), Comp::Gt)
        .and(LeagueFieldValue::id(8), Comp::Lt)
        .query()
        .await
        .expect("Failed to update records with the querybuilder");

    let found_updated_values = League::find_query_datasource(PSQL_DS)
        .r#where(LeagueFieldValue::id(1), Comp::Gt)
        .and(LeagueFieldValue::id(7), Comp::Lt)
        .query()
        .await
        .expect("Failed to retrieve database League entries with the querybuilder");

    found_updated_values
        .iter()
        .for_each(|league| assert_eq!(league.slug, "Updated with the QueryBuilder"));
}

/// Same as above, but with the specified datasource
#[canyon_sql::macros::canyon_tokio_test]
fn test_crud_update_with_querybuilder_datasource() {
    // Find all the leagues with ID less or equals that 7
    // and where it's region column value is equals to 'Korea'
    Player::update_query_datasource(PSQL_DS)
        .set(&[
            (PlayerField::summoner_name, "Random updated player name"),
            (PlayerField::first_name, "I am an updated first name"),
        ])
        .r#where(PlayerFieldValue::id(1), Comp::Gt)
        .and(PlayerFieldValue::id(8), Comp::Lt)
        .query()
        .await
        .expect("Failed to update records with the querybuilder");
}

/// Deletes entries from the mapped entity `T` that are in the ranges filtered
/// with the QueryBuilder
///
/// Note if the database is persisted (not created and destroyed on every docker or
/// GitHub Action wake up), it won't delete things that already have been deleted,
/// but this isn't an error. They just don't exists.
#[canyon_sql::macros::canyon_tokio_test]
fn test_crud_delete_with_querybuilder() {
    Tournament::delete_query()
        .r#where(TournamentFieldValue::id(14), Comp::Gt)
        .and(TournamentFieldValue::id(16), Comp::Lt)
        .query()
        .await
        .expect("Error connecting with the database on the delete operation");

    assert_eq!(Tournament::find_by_pk(&15).await.unwrap(), None);
}

/// Same as the above delete, but with the specified datasource
#[canyon_sql::macros::canyon_tokio_test]
fn test_crud_delete_with_querybuilder_datasource() {
    Player::delete_query_datasource(PSQL_DS)
        .r#where(PlayerFieldValue::id(120), Comp::Gt)
        .and(PlayerFieldValue::id(130), Comp::Lt)
        .query()
        .await
        .expect("Error connecting with the database when we are going to delete data! :)");

    assert!(
        Player::find_query_datasource(PSQL_DS)
        .r#where(PlayerFieldValue::id(122), Comp::Eq)
        .query()
        .await
        .unwrap()
        .is_empty()
    );
}
