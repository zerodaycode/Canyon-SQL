use canyon_sql::{*, bounds::IntegralNumber};

/// Represents a @LeagueOfLegends official League from some
/// region
/// 
/// Leagues are like the parent relation for [`Tournament`], where a
/// [`League`] can have multiple tournaments pointing one only league
/// 
/// Ex: LEC Spring Split + LEC Summer Split of 202X year
#[derive(Debug, Clone, CanyonCrud, CanyonMapper, ForeignKeyable)]
#[canyon_macros::canyon_entity]
pub struct League {
    pub id: i32,
    pub ext_id: i64,
    pub slug: String,
    pub name: String,
    pub region: String,
    pub image_url: String
}