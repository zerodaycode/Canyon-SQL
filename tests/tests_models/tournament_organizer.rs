use canyon_sql::macros::*;

/// Parent fixture whose physical table name cannot be inferred from its Rust
/// type. Relationship lookups must obtain it from `EntityRuntimeInfo`.
#[derive(Debug, Clone, Fields, Crud, CanyonMapper, Eq, PartialEq)]
#[canyon_entity(table_name = "tournament_organizers")]
pub struct TournamentOrganizer {
    #[primary_key]
    id: i32,
    external_id: i64,
    name: String,
}
