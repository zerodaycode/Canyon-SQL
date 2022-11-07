use canyon_sql::*;

#[derive(Debug, Clone, CanyonCrud, CanyonMapper, PartialEq)]
#[canyon_macros::canyon_entity]
pub struct Player {
    #[primary_key]
    id: i32,
    ext_id: i64,
    first_name: String,
    last_name: String,
    summoner_name: String,
    image_url: Option<String>,
    role: String
}