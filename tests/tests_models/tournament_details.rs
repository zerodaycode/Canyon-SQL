use canyon_sql::macros::*;

/// Parent fixture for foreign-key integration tests.
///
/// Deliberately relies on Canyon's default table-name mapping so the Rust type
/// `TournamentDetails` must resolve to the physical table `tournament_details`.
#[derive(Debug, Clone, Fields, Crud, CanyonMapper, Eq, PartialEq)]
#[canyon_entity]
pub struct TournamentDetails {
    #[primary_key]
    id: i32,
    external_id: i64,
    description: String,
}
