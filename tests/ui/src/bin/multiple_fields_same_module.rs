use canyon_sql::macros::{canyon_entity, CanyonMapper, Crud, Fields};

#[derive(Debug, Fields, Crud, CanyonMapper)]
#[canyon_entity(table_name = "teams")]
struct Team {
    #[primary_key]
    id: i64,
    name: String,
}

#[derive(Debug, Fields, Crud, CanyonMapper)]
#[canyon_entity(table_name = "players")]
struct Player {
    #[primary_key]
    id: i64,
    #[foreign_key(references = Team::id)]
    team_id: i64,
}

fn main() {
    assert_eq!(TeamTable::DbName.to_string(), "teams");
    assert_eq!(TeamTable::DbName.table_name(), "teams");
    assert_eq!(TeamField::name.to_string(), "name");
    assert_eq!(TeamField::name.as_str(), "name");
    assert_eq!(PlayerTable::DbName.to_string(), "players");
    assert_eq!(PlayerField::team_id.to_string(), "team_id");

    let value = PlayerFieldValue::team_id(7);
    let column = canyon_sql::query::bounds::FieldValueIdentifier::column(&value);
    assert_eq!(column.name(), "team_id");
}
