use canyon_sql::*;

#[derive(Debug, Clone, CanyonCRUD, CanyonMapper, ForeignKeyable)]
#[canyon_macros::canyon_entity]
pub struct Leagues {
    pub id: i32,
    pub ext_id: i64,
    pub slug: String,
    pub name: String,
    pub region: String,
    pub image_url: String
}