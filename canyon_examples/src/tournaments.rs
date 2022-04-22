use canyon_sql::*;
// use chrono::NaiveDate;

/// TODO Some Rust documentation here
#[derive(Debug, Clone, CanyonCrud, CanyonMapper)]
#[canyon_macros::canyon_entity]
pub struct Tournaments {
    pub id: i32,
    pub ext_id: i64,
    pub slug: String,
    // pub start_date: NaiveDate,
    // pub end_date: NaiveDate,
    #[foreign_key(column = "id")]
    pub league: i32
}