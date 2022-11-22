use canyon_sql::macros::*;

#[derive(Debug, Clone, Fields, CanyonCrud, CanyonMapper, Eq, PartialEq)]
#[canyon_entity]
/// Data model that represents a database entity for Players.
///
/// For test the behaviour of Canyon with entities that no declares primary keys,
/// or that is configuration isn't autoincremental, we will use this class.
/// Note that this entity has a primary key declared in the database, but we will
/// omit this in Canyon, so for us, is like if the primary key wasn't setted up.
///
/// Remember that the entities that does not declares at least a field as `#[primary_key]`
/// does not have all the CRUD operations available, only the ones that doesn't
/// requires of a primary key.
pub struct Player {
    // #[primary_key]  We will omit this to use it as a mock of entities that doesn't declares primary key
    id: i32,
    ext_id: i64,
    first_name: String,
    last_name: String,
    summoner_name: String,
    image_url: Option<String>,
    role: String,
}
