use canyon_sql::macros::{CanyonMapper, Read, canyon_entity};

#[derive(CanyonMapper, Read)]
#[canyon_entity(table_name = "players")]
struct Player {
    id: i64,
    #[foreign_key(references = "Team::id")]
    team_id: i64,
}

fn main() {}
