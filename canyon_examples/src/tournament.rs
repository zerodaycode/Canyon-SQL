use canyon_sql::*;
use chrono::NaiveDate;

use super::league::League;

/// TODO Some Rust documentation here
#[derive(Debug, Clone, CanyonCrud, CanyonMapper)]
#[canyon_macros::canyon_entity]
pub struct Tournament {
    pub id: i32,
    pub ext_id: i64,
    pub slug: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    
    #[foreign_key(table = "league", column = "id")]
    pub league: i32
}