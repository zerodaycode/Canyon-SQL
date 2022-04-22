use canyon_sql::*;

/// TODO Some Rust documentation here
/// TODO Also, explain how `#[canyon_macros::canyon_entity]`
/// it's able to manage the whole operations for you
#[derive(Debug, Clone, CanyonCrud, CanyonMapper, ForeignKeyable)]
#[canyon_macros::canyon_entity]
pub struct Leagues {
    pub id: i32,
    pub ext_id: i64,
    pub slug: String,
    pub name: String,
    pub region: String,
    pub image_url: String
}