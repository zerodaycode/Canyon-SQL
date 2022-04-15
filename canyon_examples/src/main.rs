use canyon_sql::*;

pub mod test;
pub mod test_foreign;
use test::{Test, TestFields};

use crate::test_foreign::TestForeign;

#[canyon]
fn main() {
    let all_test_elements: Vec<Test> = Test::find_all().await;
    println!("Test elements: {:?}", &all_test_elements);

    let all_test_as_queryb = Test::find_all_query()
        .where_clause(TestFields::id(1), Comp::Lt)
        .query()
        .await;
    println!("Test elements QUERYBUILDER: {:?}", &all_test_as_queryb);

    let tests_foreign: Vec<TestForeign> = TestForeign::search_by_fk(&all_test_elements[0]).await;
}
