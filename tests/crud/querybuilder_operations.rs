///! Tests for the QueryBuilder available operations within Canyon.
///
///! QueryBuilder are the way of obtain more flexibility that with
///! the default generated queries, essentially for build the queries
///! with the SQL filters
///
use canyon_sql::{
    crud::CrudOperations,
    query::{operators::Comp, ops::QueryBuilder},
};

#[cfg(feature = "mssql")]
use crate::constants::SQL_SERVER_DS;
use crate::tests_models::league::*;
#[cfg(feature = "mssql")]
use crate::tests_models::player::*;
use crate::tests_models::tournament::*;

/// Builds a new SQL statement for retrieves entities of the `T` type, filtered
/// with the parameters that modifies the base SQL to SELECT * FROM <entity>
#[canyon_sql::macros::canyon_tokio_test]
fn test_generated_sql_by_the_select_querybuilder() {
    let mut select_with_joins = League::select_query();
    select_with_joins
        .inner_join("tournament", "league.id", "tournament.league_id")
        .left_join("team", "tournament.id", "player.tournament_id")
        .r#where(LeagueFieldValue::id(&7), Comp::Gt)
        .and(LeagueFieldValue::name(&"KOREA"), Comp::Eq)
        .and_values_in(LeagueField::name, &["LCK", "STRANGER THINGS"]);
    // .query()
    // .await;
    // NOTE: We don't have in the docker the generated relationships
    // with the joins, so for now, we are just going to check that the
    // generated SQL by the SelectQueryBuilder<T> is the spected
    assert_eq!(
        select_with_joins.read_sql(),
        "SELECT * FROM league INNER JOIN tournament ON league.id = tournament.league_id LEFT JOIN team ON tournament.id = player.tournament_id WHERE id > $1 AND name = $2  AND name IN ($2, $3) "
    )
}

/// Builds a new SQL statement for retrieves entities of the `T` type, filtered
/// with the parameters that modifies the base SQL to SELECT * FROM <entity>
#[cfg(feature = "postgres")]
#[canyon_sql::macros::canyon_tokio_test]
fn test_crud_find_with_querybuilder() {
    // Find all the leagues with ID less or equals that 7
    // and where it's region column value is equals to 'Korea'
    let filtered_leagues_result: Result<Vec<League>, _> = League::select_query()
        .r#where(LeagueFieldValue::id(&50), Comp::LtEq)
        .and(LeagueFieldValue::region(&"KOREA"), Comp::Eq)
        .query()
        .await;

    let filtered_leagues: Vec<League> = filtered_leagues_result.unwrap();
    assert!(!filtered_leagues.is_empty());

    let league_idx_0 = filtered_leagues.get(0).unwrap();
    assert_eq!(league_idx_0.id, 34);
    assert_eq!(league_idx_0.region, "KOREA");
}

/// Same than the above but with the specified datasource
#[cfg(feature = "mssql")]
#[canyon_sql::macros::canyon_tokio_test]
fn test_crud_find_with_querybuilder_datasource() {
    // Find all the players where its ID column value is greater that 50
    let filtered_find_players = Player::select_query_datasource(SQL_SERVER_DS)
        .r#where(PlayerFieldValue::id(&50), Comp::Gt)
        .query()
        .await;

    assert!(!filtered_find_players.unwrap().is_empty());
}

/// Updates the values of the range on entries defined by the constraint parameters
/// in the database entity
#[cfg(feature = "postgres")]
#[canyon_sql::macros::canyon_tokio_test]
fn test_crud_update_with_querybuilder() {
    // Find all the leagues with ID less or equals that 7
    // and where it's region column value is equals to 'Korea'
    let mut q = League::update_query();
    q.set(&[
        (LeagueField::slug, "Updated with the QueryBuilder"),
        (LeagueField::name, "Random"),
    ])
    .r#where(LeagueFieldValue::id(&1), Comp::Gt)
    .and(LeagueFieldValue::id(&8), Comp::Lt);

    /*  NOTE: Family of QueryBuilders are clone, useful in case of need to read the generated SQL
        let qpr = q.clone();
        println!("PSQL: {:?}", qpr.read_sql());
    */

    // We can now back to the original an throw the query
    q.query()
        .await
        .expect("Failed to update records with the querybuilder");

    let found_updated_values = League::select_query()
        .r#where(LeagueFieldValue::id(&1), Comp::Gt)
        .and(LeagueFieldValue::id(&7), Comp::Lt)
        .query()
        .await
        .expect("Failed to retrieve database League entries with the querybuilder");

    found_updated_values
        .iter()
        .for_each(|league| assert_eq!(league.slug, "Updated with the QueryBuilder"));
}

/// Same as above, but with the specified datasource
#[cfg(feature = "mssql")]
#[canyon_sql::macros::canyon_tokio_test]
fn test_crud_update_with_querybuilder_datasource() {
    // Find all the leagues with ID less or equals that 7
    // and where it's region column value is equals to 'Korea'
    let mut q = Player::update_query_datasource(SQL_SERVER_DS);
    q.set(&[
        (PlayerField::summoner_name, "Random updated player name"),
        (PlayerField::first_name, "I am an updated first name"),
    ])
    .r#where(PlayerFieldValue::id(&1), Comp::Gt)
    .and(PlayerFieldValue::id(&8), Comp::Lt)
    .query()
    .await
    .expect("Failed to update records with the querybuilder");

    let found_updated_values = Player::select_query_datasource(SQL_SERVER_DS)
        .r#where(PlayerFieldValue::id(&1), Comp::Gt)
        .and(PlayerFieldValue::id(&7), Comp::LtEq)
        .query()
        .await
        .expect("Failed to retrieve database League entries with the querybuilder");

    found_updated_values.iter().for_each(|player| {
        assert_eq!(player.summoner_name, "Random updated player name");
        assert_eq!(player.first_name, "I am an updated first name");
    });
}

/// Deletes entries from the mapped entity `T` that are in the ranges filtered
/// with the QueryBuilder
///
/// Note if the database is persisted (not created and destroyed on every docker or
/// GitHub Action wake up), it won't delete things that already have been deleted,
/// but this isn't an error. They just don't exists.
#[cfg(feature = "postgres")]
#[canyon_sql::macros::canyon_tokio_test]
fn test_crud_delete_with_querybuilder() {
    Tournament::delete_query()
        .r#where(TournamentFieldValue::id(&14), Comp::Gt)
        .and(TournamentFieldValue::id(&16), Comp::Lt)
        .query()
        .await
        .expect("Error connecting with the database on the delete operation");

    assert_eq!(Tournament::find_by_pk(&15).await.unwrap(), None);
}

/// Same as the above delete, but with the specified datasource
#[cfg(feature = "mssql")]
#[canyon_sql::macros::canyon_tokio_test]
fn test_crud_delete_with_querybuilder_datasource() {
    Player::delete_query_datasource(SQL_SERVER_DS)
        .r#where(PlayerFieldValue::id(&120), Comp::Gt)
        .and(PlayerFieldValue::id(&130), Comp::Lt)
        .query()
        .await
        .expect("Error connecting with the database when we are going to delete data! :)");

    assert!(Player::select_query_datasource(SQL_SERVER_DS)
        .r#where(PlayerFieldValue::id(&122), Comp::Eq)
        .query()
        .await
        .unwrap()
        .is_empty());
}

/// Tests for the generated SQL query after use the
/// WHERE clause
#[canyon_sql::macros::canyon_tokio_test]
fn test_where_clause() {
    let mut l = League::select_query();
    l.r#where(LeagueFieldValue::name(&"LEC"), Comp::Eq);

    assert_eq!(l.read_sql(), "SELECT * FROM league WHERE name = $1")
}

/// Tests for the generated SQL query after use the
/// AND clause
#[canyon_sql::macros::canyon_tokio_test]
fn test_and_clause() {
    let mut l = League::select_query();
    l.r#where(LeagueFieldValue::name(&"LEC"), Comp::Eq)
        .and(LeagueFieldValue::id(&10), Comp::LtEq);

    assert_eq!(
        l.read_sql().trim(),
        "SELECT * FROM league WHERE name = $1 AND id <= $2"
    )
}

/// Tests for the generated SQL query after use the
/// AND clause
#[canyon_sql::macros::canyon_tokio_test]
fn test_and_clause_with_in_constraint() {
    let mut l = League::select_query();
    l.r#where(LeagueFieldValue::name(&"LEC"), Comp::Eq)
        .and_values_in(LeagueField::id, &[1, 7, 10]);

    assert_eq!(
        l.read_sql().trim(),
        "SELECT * FROM league WHERE name = $1 AND id IN ($1, $2, $3)"
    )
}

/// Tests for the generated SQL query after use the
/// AND clause
#[canyon_sql::macros::canyon_tokio_test]
fn test_or_clause() {
    let mut l = League::select_query();
    l.r#where(LeagueFieldValue::name(&"LEC"), Comp::Eq)
        .or(LeagueFieldValue::id(&10), Comp::LtEq);

    assert_eq!(
        l.read_sql().trim(),
        "SELECT * FROM league WHERE name = $1 OR id <= $2"
    )
}

/// Tests for the generated SQL query after use the
/// AND clause
#[canyon_sql::macros::canyon_tokio_test]
fn test_or_clause_with_in_constraint() {
    let mut l = League::select_query();
    l.r#where(LeagueFieldValue::name(&"LEC"), Comp::Eq)
        .or_values_in(LeagueField::id, &[1, 7, 10]);

    assert_eq!(
        l.read_sql(),
        "SELECT * FROM league WHERE name = $1 OR id IN ($1, $2, $3) "
    )
}

/// Tests for the generated SQL query after use the
/// AND clause
#[canyon_sql::macros::canyon_tokio_test]
fn test_order_by_clause() {
    let mut l = League::select_query();
    l.r#where(LeagueFieldValue::name(&"LEC"), Comp::Eq)
        .order_by(LeagueField::id, false);

    assert_eq!(
        l.read_sql(),
        "SELECT * FROM league WHERE name = $1 ORDER BY id"
    )
}
