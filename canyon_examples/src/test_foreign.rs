use canyon_sql::*;

#[derive(Debug, Clone, CanyonCRUD, CanyonMapper)]
#[canyon_macros::canyon_entity]
pub struct TestForeign {
    pub id: i32,
    pub other_value: String
}