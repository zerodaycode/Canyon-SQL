use crate::tests_models::league::League;
use canyon_sql::{date_time::NaiveDate, macros::*};

#[derive(Debug, Clone, Fields, CanyonCrud, CanyonMapper, Eq, PartialEq)]
#[canyon_entity]
pub struct Tournament {
    #[primary_key]
    id: i32,
    ext_id: i64,
    slug: String,
    start_date: NaiveDate,
    end_date: NaiveDate,
    // TODO Error on CanyonCRUD macro, bad return type
    // #[foreign_key(table = "league", column = "id")]
    league: i32,
}
