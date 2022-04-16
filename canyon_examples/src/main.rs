use canyon_sql::*;
use chrono::NaiveDate;
pub mod leagues;
pub mod tournaments;

use leagues::*;
use tournaments::*;

#[canyon]
fn main() {
    let all_leagues: Vec<Leagues> = Leagues::find_all().await;
    println!("Leagues elements: {:?}", &all_leagues);

    let all_leagues_as_querybuilder = Leagues::find_all_query()
        .where_clause(LeaguesFields::id(1), Comp::Eq)
        .query()
        .await;
    println!("Leagues elements QUERYBUILDER: {:?}", &all_leagues_as_querybuilder);

    let tournament_test = Tournaments {
            id: 1,
            ext_id: 1, 
            slug: "slug".to_string(),
            // start_date: NaiveDate::from_ymd(2015, 3, 14), 
            // end_date: NaiveDate::from_ymd(2015, 3, 14),
            league: 1
    };

    let league_test = Leagues {
        id: 1,
        ext_id: 1,
        slug: "slug".to_string(),
        name: "name".to_string(),
        region: "region".to_string(),
        image_url: "image_url".to_string(),
    };

    let tests_foreign = Tournaments::search_by_fk(&league_test).await;
    println!("TestForeign elements FK: {:?}", &tests_foreign);

}