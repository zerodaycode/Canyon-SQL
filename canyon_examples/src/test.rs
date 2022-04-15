use canyon_sql::*;

#[derive(Debug, Clone, CanyonCRUD, CanyonMapper, ForeignKeyable)]
#[canyon_macros::canyon_entity]
pub struct Test {
    pub id: i32,
    pub field: String,
    pub name: String,
    #[foreign_key(table = "testforeign", column = "id")]
    pub just_an_i32: i32
}