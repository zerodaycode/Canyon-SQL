use canyon_sql::macros::{CanyonMapper, Read, canyon_entity};

#[derive(CanyonMapper)]
#[canyon_entity(table_name = "teams")]
struct Team {
    id: i64,
    external_id: i64,
}

#[derive(CanyonMapper, Read)]
#[canyon_entity(table_name = "players")]
struct Player {
    id: i64,
    #[foreign_key(references = crate::Team::external_id)]
    team_id: i64,
}

fn relationship_methods_are_inherent(player: &Player, team: &Team) {
    let _ = player.find_team();
    let _ = player.find_team_with("main");
    let _ = Player::find_all_by_team(team);
    let _ = Player::find_all_by_team_with(team, "main");
}

fn main() {}
