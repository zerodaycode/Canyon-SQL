use canyon_sql::*;
// use chrono::NaiveDate;

use super::leagues::League;

/// TODO Some Rust documentation here
#[derive(Debug, Clone, CanyonCrud, CanyonMapper)]
#[canyon_macros::canyon_entity]
pub struct Tournament {
    pub id: i32,
    pub ext_id: i64,
    pub slug: String,
    // TODO Fix PostgreSQL relation with dates
    // pub start_date: NaiveDate,
    // pub end_date: NaiveDate,
    // TODO Make the table annotation accept a valid Rust identifier as well
    #[foreign_key(table = "league", column = "id")]
    pub league: i32
}