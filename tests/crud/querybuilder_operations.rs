#[cfg(feature = "mysql")]
use crate::constants::MYSQL_DS;
#[cfg(feature = "mssql")]
use crate::constants::SQL_SERVER_DS;
use canyon_sql::connection::DatabaseType;

/// Tests for the QueryBuilder available operations within Canyon.
///
/// QueryBuilder are the way of obtain more flexibility that with
/// the default generated queries, essentially for build the queries
/// with the SQL filters
///
use canyon_sql::query::operators::{
    LikeKind::{Full, Left, Right},
    Operator,
    Operator::*,
};

/// Tests for the QueryBuilder available operations within Canyon.
///
/// QueryBuilder are the way of obtain more flexibility that with
/// the default generated queries, essentially for build the queries
/// with the SQL filters
///
use canyon_sql::{
    crud::CrudOperations,
    query::querybuilder::{QueryBuilderOps, SelectQueryBuilderOps, UpdateQueryBuilderOps},
};

use crate::tests_models::league::*;
use crate::tests_models::player::*;
use crate::tests_models::tournament::*;

#[canyon_sql::macros::canyon_tokio_test]
#[cfg(feature = "postgres")]
fn test_generated_sql_by_the_select_querybuilder() {
    let fv = LeagueFieldValue::name("KOREA".to_string());
    let select_with_joins = League::select_query()
        .unwrap()
        .inner_join(
            TournamentTable::DbName,
            LeagueField::id,
            TournamentField::league,
        )
        .left_join(PlayerTable::DbName, TournamentField::id, PlayerField::id)
        .where_value(&LeagueFieldValue::id(7), Operator::Gt)
        .and(&fv, Operator::Eq)
        .and_values_in(LeagueField::name, &["LCK", "STRANGER THINGS"]);

    assert_eq!(
        select_with_joins.unwrap().sql().unwrap(),
        "SELECT * FROM league INNER JOIN tournament ON \"league\".\"id\" = \"tournament\".\"league\" LEFT JOIN player ON \"tournament\".\"id\" = \"player\".\"id\" WHERE \"id\" > $1 AND \"name\" = $2 AND \"name\" IN ($2, $3);"
    )
}

// #[cfg(feature = "postgres")]
// #[canyon_sql::macros::canyon_tokio_test]
// fn test_crud_find_with_querybuilder() {
//     // Find all the leagues with ID less or equals that 7
//     // and where it's region column value is equals to 'Korea'
//     let fv = LeagueFieldValue::region("KOREA".to_string());
//     let filtered_leagues_result: Result<Vec<League>, _> = League::select_query()
//         .unwrap()
//         .where_value(&LeagueFieldValue::id(50), Operator::LtEq)
//         .and(&fv, Operator::Eq)
//         .build()
//         .unwrap()
//         .launch_default()
//         .await;
//
//     let filtered_leagues: Vec<League> = filtered_leagues_result.unwrap();
//     assert!(!filtered_leagues.is_empty());
//
//     let league_idx_0 = filtered_leagues.first().unwrap();
//     assert_eq!(league_idx_0.id, 34);
//     assert_eq!(league_idx_0.region, "KOREA");
// }

/// Builds a new SQL statement for retrieves entities of the `T` type, filtered
/// with the parameters that modifies the base SQL to SELECT * FROM <entity>
#[cfg(feature = "postgres")]
#[canyon_sql::macros::canyon_tokio_test]
fn test_crud_find_with_querybuilder_and_fulllike() {
    // Find all the leagues with "LC" in their name
    let binding = LeagueFieldValue::name("LEC".to_string());
    let filtered_leagues_result = League::select_query()
        .unwrap()
        .where_value(&binding, Like(Full));

    assert_eq!(
        filtered_leagues_result.build().unwrap().sql,
        "SELECT * FROM league WHERE name LIKE CONCAT('%', CAST($1 AS VARCHAR) ,'%')"
    )
}
//
// /// Builds a new SQL statement for retrieves entities of the `T` type, filtered
// /// with the parameters that modifies the base SQL to SELECT * FROM <entity>
// #[cfg(feature = "mssql")]
// #[canyon_sql::macros::canyon_tokio_test]
// fn test_crud_find_with_querybuilder_and_fulllike_with_mssql() {
//     // Find all the leagues with "LC" in their name
//     let fv = LeagueFieldValue::name("LEC".to_string());
//     let filtered_leagues_result = League::select_query_with(DatabaseType::SqlServer)
//         .unwrap()
//         .where_value(&fv, Like(Full));
//
//     assert_eq!(
//         filtered_leagues_result.read_sql(),
//         "SELECT * FROM league WHERE name LIKE CONCAT('%', CAST($1 AS VARCHAR) ,'%')"
//     )
// }
//
// /// Builds a new SQL statement for retrieves entities of the `T` type, filtered
// /// with the parameters that modifies the base SQL to SELECT * FROM <entity>
// #[cfg(feature = "mysql")]
// #[canyon_sql::macros::canyon_tokio_test]
// fn test_crud_find_with_querybuilder_and_fulllike_with_mysql() {
//     // Find all the leagues with "LC" in their name
//     let fv = LeagueFieldValue::name("LEC".to_string());
//     let filtered_leagues_result = League::select_query_with(DatabaseType::MySQL)
//         .unwrap()
//         .where_value(&fv, Like(Full));
//
//     assert_eq!(
//         filtered_leagues_result.read_sql(),
//         "SELECT * FROM league WHERE name LIKE CONCAT('%', CAST($1 AS CHAR) ,'%')"
//     )
// }
//
// /// Builds a new SQL statement for retrieves entities of the `T` type, filtered
// /// with the parameters that modifies the base SQL to SELECT * FROM <entity>
// #[cfg(feature = "postgres")]
// #[canyon_sql::macros::canyon_tokio_test]
// fn test_crud_find_with_querybuilder_and_leftlike() {
//     // Find all the leagues whose name ends with "CK"
//     let fv = LeagueFieldValue::name("CK".to_string());
//     let filtered_leagues_result = League::select_query().unwrap().where_value(&fv, Like(Left));
//
//     assert_eq!(
//         filtered_leagues_result.read_sql(),
//         "SELECT * FROM league WHERE name LIKE CONCAT('%', CAST($1 AS VARCHAR))"
//     )
// }
//
// /// Builds a new SQL statement for retrieves entities of the `T` type, filtered
// /// with the parameters that modifies the base SQL to SELECT * FROM <entity>
// #[cfg(feature = "mssql")]
// #[canyon_sql::macros::canyon_tokio_test]
// fn test_crud_find_with_querybuilder_and_leftlike_with_mssql() {
//     // Find all the leagues whose name ends with "CK"
//     let fv = LeagueFieldValue::name("CK".to_string());
//     let filtered_leagues_result = League::select_query_with(DatabaseType::SqlServer)
//         .unwrap()
//         .where_value(&fv, Like(Left));
//
//     assert_eq!(
//         filtered_leagues_result.read_sql(),
//         "SELECT * FROM league WHERE name LIKE CONCAT('%', CAST($1 AS VARCHAR))"
//     )
// }
//
// /// Builds a new SQL statement for retrieves entities of the `T` type, filtered
// /// with the parameters that modifies the base SQL to SELECT * FROM <entity>
// #[cfg(feature = "mysql")]
// #[canyon_sql::macros::canyon_tokio_test]
// fn test_crud_find_with_querybuilder_and_leftlike_with_mysql() {
//     // Find all the leagues whose name ends with "CK"
//     let fv = LeagueFieldValue::name("CK".to_string());
//     let filtered_leagues_result = League::select_query_with(DatabaseType::MySQL)
//         .unwrap()
//         .where_value(&fv, Like(Left));
//
//     assert_eq!(
//         filtered_leagues_result.read_sql(),
//         "SELECT * FROM league WHERE name LIKE CONCAT('%', CAST($1 AS CHAR))"
//     )
// }
//
// /// Builds a new SQL statement for retrieves entities of the `T` type, filtered
// /// with the parameters that modifies the base SQL to SELECT * FROM <entity>
// #[cfg(feature = "postgres")]
// #[canyon_sql::macros::canyon_tokio_test]
// fn test_crud_find_with_querybuilder_and_rightlike() {
//     // Find all the leagues whose name starts with "LC"
//     let fv = LeagueFieldValue::name("LEC".to_string());
//     let filtered_leagues_result = League::select_query().unwrap().where_value(&fv, Like(Right));
//
//     assert_eq!(
//         filtered_leagues_result.read_sql(),
//         "SELECT * FROM league WHERE name LIKE CONCAT(CAST($1 AS VARCHAR) ,'%')"
//     )
// }
//
// /// Builds a new SQL statement for retrieves entities of the `T` type, filtered
// /// with the parameters that modifies the base SQL to SELECT * FROM <entity>
// #[cfg(feature = "mssql")]
// #[canyon_sql::macros::canyon_tokio_test]
// fn test_crud_find_with_querybuilder_and_rightlike_with_mssql() {
//     // Find all the leagues whose name starts with "LC"
//     let fv = LeagueFieldValue::name("LEC".to_string());
//     let filtered_leagues_result = League::select_query_with(DatabaseType::SqlServer)
//         .unwrap()
//         .where_value(&fv, Like(Right));
//
//     assert_eq!(
//         filtered_leagues_result.read_sql(),
//         "SELECT * FROM league WHERE name LIKE CONCAT(CAST($1 AS VARCHAR) ,'%')"
//     )
// }
//
// /// Builds a new SQL statement for retrieves entities of the `T` type, filtered
// /// with the parameters that modifies the base SQL to SELECT * FROM <entity>
// #[cfg(feature = "mysql")]
// #[canyon_sql::macros::canyon_tokio_test]
// fn test_crud_find_with_querybuilder_and_rightlike_with_mysql() {
//     // Find all the leagues whose name starts with "LC"
//     let wh = LeagueFieldValue::name("LEC".to_string());
//     let filtered_leagues_result = League::select_query_with(DatabaseType::MySQL)
//         .unwrap()
//         .where_value(&wh, Like(Right));
//
//     assert_eq!(
//         filtered_leagues_result.read_sql(),
//         "SELECT * FROM league WHERE name LIKE CONCAT(CAST($1 AS CHAR) ,'%')"
//     )
// }
//
// /// Same than the above but with the specified datasource
// #[cfg(feature = "mssql")]
// #[canyon_sql::macros::canyon_tokio_test]
// fn test_crud_find_with_querybuilder_with_mssql() {
//     // Find all the players where its ID column value is greater than 50
//     let filtered_find_players = Player::select_query_with(DatabaseType::SqlServer)
//         .unwrap()
//         .where_value(&PlayerFieldValue::id(50), Comp::Gt)
//         .build()
//         .unwrap()
//         .launch_with::<&str, Player>(SQL_SERVER_DS)
//         .await;
//
//     assert!(!filtered_find_players.unwrap().is_empty());
// }
//
// /// Same than the above but with the specified datasource
// #[cfg(feature = "mysql")]
// #[canyon_sql::macros::canyon_tokio_test]
// fn test_crud_find_with_querybuilder_with_mysql() {
//     // Find all the players where its ID column value is greater than 50
//     let filtered_find_players = Player::select_query_with(DatabaseType::MySQL)
//         .unwrap()
//         .where_value(&PlayerFieldValue::id(50), Comp::Gt)
//         .build()
//         .unwrap()
//         .launch_with::<&str, Player>(MYSQL_DS)
//         .await;
//
//     assert!(!filtered_find_players.unwrap().is_empty());
// }
//
// /// Updates the values of the range on entries defined by the constraint parameters
// /// in the database entity
// #[cfg(feature = "postgres")]
// #[canyon_sql::macros::canyon_tokio_test]
// fn test_crud_update_with_querybuilder() {
//     // Find all the leagues with ID less or equals that 7
//     // and where it's region column value is equals to 'Korea'
//     let q = League::update_query()
//         .unwrap()
//         .set_values(&[
//             (LeagueField::slug, "Updated with the QueryBuilder"),
//             (LeagueField::name, "Random"),
//         ]).unwrap()
//         .where_value(&LeagueFieldValue::id(1), Comp::Gt)
//         .and(&LeagueFieldValue::id(8), Comp::Lt);
//
//     q.build()
//         .expect("Failed to update records with the querybuilder");
//
//     let found_updated_values = League::select_query()
//         .unwrap()
//         .where_value(&LeagueFieldValue::id(1), Comp::Gt)
//         .and(&LeagueFieldValue::id(7), Comp::Lt)
//         .build()
//         .unwrap()
//         .launch_default::<League>()
//         .await
//         .expect("Failed to retrieve database League entries with the querybuilder");
//
//     found_updated_values
//         .iter()
//         .for_each(|league| assert_eq!(league.slug, "Updated with the QueryBuilder"));
// }
//
// /// Same as above, but with the specified datasource
// #[cfg(feature = "mssql")]
// #[canyon_sql::macros::canyon_tokio_test]
// fn test_crud_update_with_querybuilder_with_mssql() {
//     // Find all the leagues with ID less or equals that 7
//     // and where it's region column value is equals to 'Korea'
//     let q = Player::update_query_with(DatabaseType::SqlServer).unwrap();
//     q.set_values(&[
//         (PlayerField::summoner_name, "Random updated player name"),
//         (PlayerField::first_name, "I am an updated first name"),
//     ]).unwrap()
//     .where_value(&PlayerFieldValue::id(1), Comp::Gt)
//     .and(&PlayerFieldValue::id(8), Comp::Lt)
//     .build()
//     .unwrap()
//     .launch_with::<&str, Player>(SQL_SERVER_DS)
//     .await
//     .expect("Failed to update records with the querybuilder");
//
//     let found_updated_values = Player::select_query_with(DatabaseType::SqlServer)
//         .unwrap()
//         .where_value(&PlayerFieldValue::id(1), Comp::Gt)
//         .and(&PlayerFieldValue::id(7), Comp::LtEq)
//         .build()
//         .unwrap()
//         .launch_with::<&str, Player>(SQL_SERVER_DS)
//         .await
//         .expect("Failed to retrieve database League entries with the querybuilder");
//
//     found_updated_values.iter().for_each(|player| {
//         assert_eq!(player.summoner_name, "Random updated player name");
//         assert_eq!(player.first_name, "I am an updated first name");
//     });
// }
//
// /// Same as above, but with the specified datasource
// #[cfg(feature = "mysql")]
// #[canyon_sql::macros::canyon_tokio_test]
// fn test_crud_update_with_querybuilder_with_mysql() {
//     // Find all the leagues with ID less or equals that 7
//     // and where it's region column value is equals to 'Korea'
//
//     let q = Player::update_query_with(DatabaseType::MySQL).unwrap();
//     q.set_values(&[
//         (PlayerField::summoner_name, "Random updated player name"),
//         (PlayerField::first_name, "I am an updated first name"),
//     ]).unwrap()
//     .where_value(&PlayerFieldValue::id(1), Comp::Gt)
//     .and(&PlayerFieldValue::id(8), Comp::Lt)
//     .build()
//     .unwrap()
//     .launch_with::<&str, Player>(MYSQL_DS)
//     .await
//     .expect("Failed to update records with the querybuilder");
//
//     let found_updated_values = Player::select_query_with(DatabaseType::MySQL)
//         .unwrap()
//         .where_value(&PlayerFieldValue::id(1), Comp::Gt)
//         .and(&PlayerFieldValue::id(7), Comp::LtEq)
//         .build()
//         .unwrap()
//         .launch_with::<&str, Player>(MYSQL_DS)
//         .await
//         .expect("Failed to retrieve database League entries with the querybuilder");
//
//     found_updated_values.iter().for_each(|player| {
//         assert_eq!(player.summoner_name, "Random updated player name");
//         assert_eq!(player.first_name, "I am an updated first name");
//     });
// }
//
// /// Deletes entries from the mapped entity `T` that are in the ranges filtered
// /// with the QueryBuilder
// ///
// /// Note if the database is persisted (not created and destroyed on every docker or
// /// GitHub Action wake up), it won't delete things that already have been deleted,
// /// but this isn't an error. They just don't exist.
// #[cfg(feature = "postgres")]
// #[canyon_sql::macros::canyon_tokio_test]
// fn test_crud_delete_with_querybuilder() {
//     Tournament::delete_query()
//         .unwrap()
//         .where_value(&TournamentFieldValue::id(14), Comp::Gt)
//         .and(&TournamentFieldValue::id(16), Comp::Lt)
//         .build()
//         .unwrap()
//         .launch_default::<Tournament>()
//         .await
//         .expect("Error connecting with the database on the delete operation");
//
//     assert_eq!(Tournament::find_by_pk(&15).await.unwrap(), None);
// }
//
// // #[cfg(feature = "postgres")]
// // #[canyon_sql::macros::canyon_tokio_test]
// // fn test_crud_delete_with_querybuilder_lt_creation() {
// //     let q = create_querybuilder_lt(10);
// //     assert_eq!(q.unwrap().read_sql(), "DELETE FROM tournament WHERE id = 10");
// // }
// //
// // #[cfg(feature = "postgres")]
// // fn create_querybuilder_lt<'a, 'b: 'a>(id: i32) -> DeleteQueryBuilder<'b> {
// //     Tournament::delete_query()
// //         .unwrap()
// //         .where_value(&TournamentFieldValue::id(id), Comp::Gt)
// // }
//
// /// Same as the above delete, but with the specified datasource
// #[cfg(feature = "mssql")]
// #[canyon_sql::macros::canyon_tokio_test]
// fn test_crud_delete_with_querybuilder_with_mssql() {
//     Player::delete_query_with(DatabaseType::SqlServer)
//         .unwrap()
//         .where_value(&PlayerFieldValue::id(120), Comp::Gt)
//         .and(&PlayerFieldValue::id(130), Comp::Lt)
//         .build()
//         .unwrap()
//         .launch_with::<&str, Player>(SQL_SERVER_DS)
//         .await
//         .expect("Error connecting with the database when we are going to delete data! :)");
//
//     assert!(
//         Player::select_query_with(DatabaseType::SqlServer)
//             .unwrap()
//             .where_value(&PlayerFieldValue::id(122), Comp::Eq)
//             .build()
//             .unwrap()
//             .launch_with::<&str, Player>(SQL_SERVER_DS)
//             .await
//             .unwrap()
//             .is_empty()
//     );
// }
//
// /// Same as the above delete, but with the specified datasource
// #[cfg(feature = "mysql")]
// #[canyon_sql::macros::canyon_tokio_test]
// fn test_crud_delete_with_querybuilder_with_mysql() {
//     Player::delete_query_with(DatabaseType::MySQL)
//         .unwrap()
//         .where_value(&PlayerFieldValue::id(120), Comp::Gt)
//         .and(&PlayerFieldValue::id(130), Comp::Lt)
//         .build()
//         .unwrap()
//         .launch_with::<&str, Player>(MYSQL_DS)
//         .await
//         .expect("Error connecting with the database when we are going to delete data! :)");
//
//     assert!(
//         Player::select_query_with(DatabaseType::MySQL)
//             .unwrap()
//             .where_value(&PlayerFieldValue::id(122), Comp::Eq)
//             .build()
//             .unwrap()
//             .launch_with::<&str, Player>(MYSQL_DS)
//             .await
//             .unwrap()
//             .is_empty()
//     );
// }
//
// /// Tests for the generated SQL query after use the
// /// WHERE clause
// #[canyon_sql::macros::canyon_tokio_test]
// fn test_where_clause() {
//     let wh = LeagueFieldValue::name("LEC".to_string());
//     let l = League::select_query().unwrap().where_value(&wh, Comp::Eq);
//
//     assert_eq!(l.read_sql(), "SELECT * FROM league WHERE name = $1")
// }
//
// /// Tests for the generated SQL query after use the
// /// AND clause
// #[canyon_sql::macros::canyon_tokio_test]
// fn test_and_clause() {
//     let wh = LeagueFieldValue::name("LEC".to_string());
//     let l = League::select_query()
//         .unwrap()
//         .where_value(&wh, Comp::Eq)
//         .and(&LeagueFieldValue::id(10), Comp::LtEq);
//
//     assert_eq!(
//         l.read_sql().trim(),
//         "SELECT * FROM league WHERE name = $1 AND id <= $2"
//     )
// }
//
// /// Tests for the generated SQL query after use the
// /// AND clause
// #[canyon_sql::macros::canyon_tokio_test]
// fn test_and_clause_with_in_constraint() {
//     let wh = LeagueFieldValue::name("LEC".to_string());
//     let l = League::select_query()
//         .unwrap()
//         .where_value(&wh, Comp::Eq)
//         .and_values_in(LeagueField::id, &[1, 7, 10]);
//
//     assert_eq!(
//         l.unwrap().read_sql().trim(),
//         "SELECT * FROM league WHERE name = $1 AND id IN ($1, $2, $3)"
//     )
// }
//
// /// Tests for the generated SQL query after use the
// /// AND clause
// #[canyon_sql::macros::canyon_tokio_test]
// fn test_or_clause() {
//     let wh = LeagueFieldValue::name("LEC".to_string());
//     let l = League::select_query()
//         .unwrap()
//         .where_value(&wh, Comp::Eq)
//         .or(&LeagueFieldValue::id(10), Comp::LtEq);
//
//     assert_eq!(
//         l.read_sql().trim(),
//         "SELECT * FROM league WHERE name = $1 OR id <= $2"
//     )
// }
//
// /// Tests for the generated SQL query after use the
// /// AND clause
// #[canyon_sql::macros::canyon_tokio_test]
// fn test_or_clause_with_in_constraint() {
//     let wh = LeagueFieldValue::name("LEC".to_string());
//     let l = League::select_query()
//         .unwrap()
//         .where_value(&wh, Comp::Eq)
//         .or_values_in(LeagueField::id, &[1, 7, 10]);
//
//     assert_eq!(
//         l.unwrap().read_sql(),
//         "SELECT * FROM league WHERE name = $1 OR id IN ($1, $2, $3)"
//     )
// }
//
// /// Tests for the generated SQL query after use the
// /// AND clause
// #[canyon_sql::macros::canyon_tokio_test]
// fn test_order_by_clause() {
//     let fv = LeagueFieldValue::name("LEC".to_string());
//     let l = League::select_query()
//         .unwrap()
//         .where_value(&fv, Comp::Eq)
//         .order_by(LeagueField::id, false);
//
//     assert_eq!(
//         l.read_sql(),
//         "SELECT * FROM league WHERE name = $1 ORDER BY id"
//     )
// }
