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
        .where_clause(TestFields::id(1), Comp::Eq)
        .query()
        .await;
    println!("Test elements QUERYBUILDER: {:?}", &all_test_as_queryb);

    let test_prueba = Test {id: 5, name: "Name".to_string(), field: "field".to_string(), just_an_i32: 1};

    let tests_foreign = 
        TestForeign::search_by_fk(&test_prueba).await;
        println!("TestForeign elements FK: {:?}", &tests_foreign);

}
