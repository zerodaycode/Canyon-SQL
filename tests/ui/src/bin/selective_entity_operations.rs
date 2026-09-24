use canyon_sql::{
    crud::{EntityDelete as _, EntityInsert as _, EntityUpdate as _},
    macros::{CanyonMapper, EntityDelete, EntityInsert, EntityUpdate, canyon_entity},
};

#[derive(CanyonMapper)]
#[canyon_entity(table_name = "users")]
struct User {
    #[primary_key]
    id: i64,
    name: String,
}

#[derive(EntityInsert)]
#[canyon_crud(maps_to = User)]
#[canyon_entity(table_name = "users")]
struct InsertUser {
    marker: (),
}

#[derive(EntityUpdate)]
#[canyon_crud(maps_to = User)]
#[canyon_entity(table_name = "users")]
struct UpdateUser {
    marker: (),
}

#[derive(EntityDelete)]
#[canyon_crud(maps_to = User)]
#[canyon_entity(table_name = "users")]
struct DeleteUser {
    marker: (),
}

fn operations_are_independent(user: &mut User) {
    let _ = InsertUser::insert_entity(user);
    let _ = UpdateUser::update_entity(user);
    let _ = DeleteUser::delete_entity(user);
}

fn main() {}
