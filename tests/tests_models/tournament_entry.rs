use canyon_sql::macros::*;

/// Child fixture for foreign-key integration tests.
///
/// Its physical table is intentionally different from the default derived
/// name, and its relationship targets a non-primary-key parent column through
/// a fully-qualified Rust path.
#[derive(Debug, Clone, Fields, Crud, CanyonMapper, Eq, PartialEq)]
#[canyon_entity(table_name = "tournament_entries")]
pub struct TournamentEntry {
    #[primary_key]
    id: i32,
    #[foreign_key(
        references = crate::tests_models::tournament_details::TournamentDetails::external_id
    )]
    details_id: i64,
    #[foreign_key(
        references = crate::tests_models::tournament_organizer::TournamentOrganizer::external_id
    )]
    organizer_id: i64,
    label: String,
}
