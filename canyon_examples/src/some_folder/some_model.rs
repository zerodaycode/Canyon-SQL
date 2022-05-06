use canyon_sql::*;

#[canyon_macros::canyon_entity]
pub struct SomeModel {
    pub some_field: String
}