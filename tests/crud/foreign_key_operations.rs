/// Integration tests for the CRUD operations available in `Canyon` that
/// generates and executes *SELECT* statements based on a entity
/// annotated with the `#[foreign_key(... args)]` annotation looking
/// for the related data with some entity `U` that acts as is parent, where `U`
/// impls `ForeignKeyable` (isn't required, but it won't unlock the
/// reverse search features parent -> child, only the child -> parent ones).
///
/// Names of the foreign key methods are autogenerated for the direct and
/// reverse side of the implementations.
/// For more info: TODO -> Link to the docs of the foreign key chapter
use canyon_sql::crud::CrudOperations;

#[cfg(feature = "mssql")]
use crate::constants::MYSQL_DS;
#[cfg(feature = "mssql")]
use crate::constants::SQL_SERVER_DS;

use crate::tests_models::league::*;
use crate::tests_models::tournament::*;

/// Given an entity `T` which has some field declaring a foreign key relation
/// with some another entity `U`, for example, performs a search to find
/// what is the parent type `U` of `T`
#[cfg(feature = "postgres")]
#[canyon_sql::macros::canyon_tokio_test]
fn test_crud_search_by_foreign_key() {
    let some_tournament: Tournament = Tournament::find_by_pk(&1)
        .await
        .expect("Result variant of the query is err")
        .expect("No result found for the given parameter");

    // We can get the parent entity for the retrieved child instance
    let parent_entity: Option<League> = some_tournament
        .search_league()
        .await
        .expect("Result variant of the query is err");

    if let Some(league) = parent_entity {
        assert_eq!(some_tournament.league, league.id)
    } else {
        assert_eq!(parent_entity, None)
    }
}

/// Same as the search by foreign key, but with the specified datasource
#[cfg(feature = "mssql")]
#[canyon_sql::macros::canyon_tokio_test]
fn test_crud_search_by_foreign_key_datasource_mssql() {
    let some_tournament: Tournament = Tournament::find_by_pk_datasource(&10, SQL_SERVER_DS)
        .await
        .expect("Result variant of the query is err")
        .expect("No result found for the given parameter");

    // We can get the parent entity for the retrieved child instance
    let parent_entity: Option<League> = some_tournament
        .search_league_datasource(SQL_SERVER_DS)
        .await
        .expect("Result variant of the query is err");

    // These are tests, and we could unwrap the result contained in the option, because
    // it always should exist that search for the data inserted when the docker starts.
    // But, just for change the style a little bit and offer more options about how to
    // handle things done with Canyon
    if let Some(league) = parent_entity {
        assert_eq!(some_tournament.league, league.id)
    } else {
        assert_eq!(parent_entity, None)
    }
}

/// Same as the search by foreign key, but with the specified datasource
#[cfg(feature = "mysql")]
#[canyon_sql::macros::canyon_tokio_test]
fn test_crud_search_by_foreign_key_datasource_mysql() {
    let some_tournament: Tournament = Tournament::find_by_pk_datasource(&10, MYSQL_DS)
        .await
        .expect("Result variant of the query is err")
        .expect("No result found for the given parameter");

    // We can get the parent entity for the retrieved child instance
    let parent_entity: Option<League> = some_tournament
        .search_league_datasource(MYSQL_DS)
        .await
        .expect("Result variant of the query is err");

    // These are tests, and we could unwrap the result contained in the option, because
    // it always should exist that search for the data inserted when the docker starts.
    // But, just for change the style a little bit and offer more options about how to
    // handle things done with Canyon
    if let Some(league) = parent_entity {
        assert_eq!(some_tournament.league, league.id)
    } else {
        assert_eq!(parent_entity, None)
    }
}

/// Given an entity `U` that is know as the "parent" side of the relation with another
/// entity `T`, for example, we can ask to the parent for the childrens that belongs
/// to `U`.
///
/// For this to work, `U`, the parent, must have derived the `ForeignKeyable` proc macro
#[cfg(feature = "postgres")]
#[canyon_sql::macros::canyon_tokio_test]
fn test_crud_search_reverse_side_foreign_key() {
    let some_league: League = League::find_by_pk(&1)
        .await
        .expect("Result variant of the query is err")
        .expect("No result found for the given parameter");

    // Computes how many tournaments are pointing to the retrieved league
    let child_tournaments: Vec<Tournament> = Tournament::search_league_childrens(&some_league)
        .await
        .expect("Result variant of the query is err");

    assert!(!child_tournaments.is_empty());
    child_tournaments
        .iter()
        .for_each(|t| assert_eq!(t.league, some_league.id));
}

/// Same as the search by the reverse side of a foreign key relation
/// but with the specified datasource
#[cfg(feature = "mssql")]
#[canyon_sql::macros::canyon_tokio_test]
fn test_crud_search_reverse_side_foreign_key_datasource_mssql() {
    let some_league: League = League::find_by_pk_datasource(&1, SQL_SERVER_DS)
        .await
        .expect("Result variant of the query is err")
        .expect("No result found for the given parameter");

    // Computes how many tournaments are pointing to the retrieved league
    let child_tournaments: Vec<Tournament> =
        Tournament::search_league_childrens_datasource(&some_league, SQL_SERVER_DS)
            .await
            .expect("Result variant of the query is err");

    assert!(!child_tournaments.is_empty());
    child_tournaments
        .iter()
        .for_each(|t| assert_eq!(t.league, some_league.id));
}

/// Same as the search by the reverse side of a foreign key relation
/// but with the specified datasource
#[cfg(feature = "mysql")]
#[canyon_sql::macros::canyon_tokio_test]
fn test_crud_search_reverse_side_foreign_key_datasource_mysql() {
    let some_league: League = League::find_by_pk_datasource(&1, MYSQL_DS)
        .await
        .expect("Result variant of the query is err")
        .expect("No result found for the given parameter");

    // Computes how many tournaments are pointing to the retrieved league
    let child_tournaments: Vec<Tournament> =
        Tournament::search_league_childrens_datasource(&some_league, MYSQL_DS)
            .await
            .expect("Result variant of the query is err");

    assert!(!child_tournaments.is_empty());
    child_tournaments
        .iter()
        .for_each(|t| assert_eq!(t.league, some_league.id));
}
