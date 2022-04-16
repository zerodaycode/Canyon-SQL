use canyon_sql::*;
use chrono::NaiveDate;


#[derive(Debug, Clone, CanyonCRUD, CanyonMapper, ForeignKeyable)]
#[canyon_macros::canyon_entity]
pub struct Tournaments {
    pub id: i32,
    pub ext_id: i64,
    pub slug: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    #[foreign_key(table = "leagues", column = "id")]
    pub league: i32
}