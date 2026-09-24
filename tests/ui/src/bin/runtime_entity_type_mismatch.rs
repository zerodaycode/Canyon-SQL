use canyon_sql::{
    crud::EntityInsert as _,
    macros::{CanyonMapper, EntityInsert, canyon_entity},
};

#[derive(CanyonMapper)]
#[canyon_entity(table_name = "users")]
struct User {
    #[primary_key]
    id: i64,
}

#[derive(CanyonMapper)]
#[canyon_entity(table_name = "teams")]
struct Team {
    #[primary_key]
    id: i64,
}

#[derive(EntityInsert)]
#[canyon_crud(maps_to = User)]
#[canyon_entity(table_name = "users")]
struct UserRepository {
    marker: (),
}

fn main() {
    let mut team = Team { id: 1 };
    let _ = UserRepository::insert_entity(&mut team);
}
