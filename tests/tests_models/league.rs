use canyon_sql::macros::*;

#[derive(Debug, Fields, CanyonCrud, CanyonMapper, ForeignKeyable, Eq, PartialEq)]
// #[canyon_entity(table_name = "league", schema = "public")]
#[canyon_entity(table_name = "league")]
pub struct League {
    #[primary_key]
    id: i32,
    ext_id: i64,
    slug: String,
    name: String,
    region: String,
    image_url: String,
}
