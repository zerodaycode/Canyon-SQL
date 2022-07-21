use canyon_sql::*;

pub mod league;
pub mod tournament;

use chrono::NaiveDate;
use league::*;
use tournament::*;

// A type alias for the error returned in the _result operations
type QueryError = canyon_sql::tokio_postgres::Error;


/// The `#[canyon]` macro represents the entry point of a Canyon program.
/// 
/// When this annotation it's present, Canyon it's able to take care about everything
/// for you related to mantain the database that you provide in the `secrets.toml` file,
/// being the most obvious and important the migrations control.
#[canyon]
fn main() {

    /*  
        On the first run, you may desire to uncomment the method call below,
        to be able to populate some data into the schema.
        Remember that all operation with CanyonCrud must be awaited,
        due to it's inherent async nature
    */
    // _wire_data_on_schema().await;

    /*
        The most basic usage pattern.
        Finds all elements on a type T, if the type its annotated with the
        #[derive(Debug, Clone, CanyonCrud, CanyonMapper)] derive macro

        This automatically returns a collection (Vector) of elements found
        after query the database, automatically desearializating the returning
        rows into elements of type T

        For short, the next operations are unwraped for results and options
        without error handling.

        A section in the official documentation it's dedicated to demostrate
        how, when required, failable operations must be error handled.
    */
    let _all_leagues: Vec<League> = League::find_all().await;
    println!("Leagues elements: {:?}", &_all_leagues);

    // The find_by_id(Number) operation. Returns an optional, 'cause this operation
    // it could be easily a failure (not found the record by the provided PRIMARY KEY)
    // let _find_by_id: Option<League> = League::find_by_id(1).await;
    // println!("Find by ID: {:?}", &_find_by_id);

    // // Same operation but with the result variants
    let _all_leagues_res: Result<Vec<League>, QueryError> = League::find_all_result().await;
    println!("Leagues elements on the result variant: {:?}", &_all_leagues_res.ok().unwrap());

    let _find_by_id: Result<Option<League>, QueryError> = League::find_by_id_result(1).await;
    println!("Find by ID as a result: {:?}", &_find_by_id.ok().unwrap()); // Still has the Option<League>

    // A simple example insertating data and handling the result returned
    // _insert_result_example().await;

    /*
        Canyon also has a powerful querybuilder.
        Every associated function or method provided through the macro implementations
        that returns a QueryBuilder type can be used as a raw builder to construct
        the query that Canyon will use to retrive data from the database.

        One really important thing to note it's that any struct annotated with the
        `[#canyon_entity]` annotation will automatically generates two enumerations
        for the curren type, following the convention: 
        
        Type identifier + Field, holding variants to identify every
        field that the type has. You can recover the field name by writting:
        `Type::variant.field_name_as_str()`

        Type identifier + FieldValue, holding variants to identify every
        field that the type has and let the user attach them data of the same
        data type that the field is bounded.
        You can recover the passed in value when created by writting:
        `Type::variant(some_value).value()` that will gives you access to the
        some value inside the variant.

        So for a -> 
            pub struct League { /* fields */ }
        an enum with the fields as variants its generated ->
            pub enum LeagueField { /* variants */ }
        an enum with the fields as variants its generated ->
            pub enum LeagueFieldValue { /* variants(data_type) */ }

        So you must bring into scope `use::/* path to my type .rs file */::TypeFieldValue`
        or simply `use::/* path to my type .rs file */::*` with a wildcard import.
        
        The querybuilder methods usually accept one of the variants of the enum to make a filter
        for the SQL clause, and a variant of the Canyon's `Comp` enum type, which indicates
        how the comparation element on the filter clauses will be 
    */
    let _all_leagues_as_querybuilder: Vec<League> = League::find_all_query()
        .where_clause(
            LeagueFieldValue::id(1), // This will create a filter -> `WHERE type.id = 1`
            Comp::Eq // where the `=` symbol it's given by this variant
        )
        .query()
        .await
        .ok()
        .unwrap();
    println!("Leagues elements QUERYBUILDER: {:?}", &_all_leagues_as_querybuilder);

    // Uncomment to see the example of find by data through a FK relation
    // _search_data_by_fk_example().await;

    // Example of make a multi insert
    _multi_insert_example().await;

    // Example of update some columns at the same time for a concrete table
    _update_columns_associated_fn().await;
}

/// Example of usage of the `.insert()` Crud operation. Also, allows you
/// to wire some data on the database to be able to retrieve and play with data 
/// 
/// Notice how the `fn` must be `async`, due to Canyon's usage of **tokio**
/// as it's runtime
/// 
/// One big important note on Canyon insert. Canyon automatically manages
/// the ID field (commonly the primary key of any table) for you.
/// This means that if you keep calling this method, Canyon will keep inserting
/// records on the database, not with the id on the instance, only with the 
/// autogenerated one. 
/// 
/// This may change on a nearly time. 'cause it's direct implications on the
/// data integrity, but for now keep an eye on this.
/// 
/// An example of multiples inserts ignoring the provided `id` could end on a
/// situation like this:
/// 
/// ```
/// ... League { id: 43, ext_id: 1, slug: "LEC", name: "League Europe Champions", region: "EU West", image_url: "https://lec.eu" }, 
/// League { id: 44, ext_id: 2, slug: "LCK", name: "League Champions Korea", region: "South Korea", image_url: "https://korean_lck.kr" }, 
/// League { id: 45, ext_id: 1, slug: "LEC", name: "League Europe Champions", region: "EU West", image_url: "https://lec.eu" }, 
/// League { id: 46, ext_id: 2, slug: "LCK", name: "League Champions Korea", region: "South Korea", image_url: "https://korean_lck.kr" } ...
/// ``` 
async fn _wire_data_on_schema() {
    // Data for the examples
    let mut lec: League = League {
        id: Default::default(),
        ext_id: 1,
        slug: "LEC".to_string(),
        name: "League Europe Champions".to_string(),
        region: "EU West".to_string(),
        image_url: "https://lec.eu".to_string(),
    };

    let mut lck: League = League {
        id: Default::default(),
        ext_id: 2,
        slug: "LCK".to_string(),
        name: "League Champions Korea".to_string(),
        region: "South Korea".to_string(),
        image_url: "https://korean_lck.kr".to_string(),
    };

    let mut lpl: League = League {
        id: Default::default(),
        ext_id: 3,
        slug: "LPL".to_string(),
        name: "League PRO China".to_string(),
        region: "China".to_string(),
        image_url: "https://chinese_lpl.ch".to_string(),
    };

    // Now, the insert operations in Canyon is designed as a method over
    // the object, so the data of the instance is automatically parsed
    // into it's correct types and formats and inserted into the table
    lec.insert().await;
    lck.insert().await;
    lpl.insert().await;
}

/// Example of usage for a search given an entity related throught the 
/// `ForeignKey` annotation
/// 
/// Every struct that contains a `ForeignKey` annotation will have automatically
/// implemented a method to find data by an entity that it's related
/// through a foreign key relation.
/// 
/// So, in the example, the struct `Tournament` has a `ForeignKey` annotation
/// in it's `league` field, which holds a value relating the data on the `id` column
/// on the table `League`, so Canyon will generate an associated function following the convenction
/// `Type::search_by__name_of_the_related_table` 
/// 
/// NOTE: You can annotate any field entity with a ForeignKey[args] annotation, but
/// if the related entity does not exists, Canyon will not throw an error, just
/// until when a query it's made you'll notice that something is wrong.
/// 
/// NOTE: When you need to make an insert into a table that contains a Foreign Key
/// relation, you must retrieve first the value of the Foreign Key field
/// by performing a SELECT operation on the database. Otherwise, you'll find
/// a data integrity error
async fn _search_data_by_fk_example() {
    // So, to recover the lpl related data in DB, and workaround the AUTOGENERATED ID, 
    // we can make a query by some other field and get the ID
    let some_lpl: Vec<League> = League::find_all_query()
        .where_clause(
            LeagueFieldValue::slug("LPL".to_string()),  // This will create a filter -> `WHERE type.slug = "LPL"`
            Comp::Eq  // where the `=` symbol it's given by this variant
        )
        .query()
        .await
        .ok()
        .unwrap();
    println!("LPL QUERYBUILDER: {:?}", &some_lpl);
        

    let tournament_itce = Tournament {
        id: 1,
        ext_id: 4126494859789,
        slug: "Slugaso".to_string(),
        start_date: NaiveDate::from_ymd(2022, 5, 07),
        end_date: NaiveDate::from_ymd(2023, 5, 10),
        league: some_lpl
            .get(0)  // Returns an Option<&T>
            .cloned()
            .unwrap()
            .id,  // The Foreign Key, pointing to the table 'League' and the 'id' column
    };
    // tournament_itce.insert().await.ok().unwrap();

    // You can search the 'League' that it's the parent of 'Tournament'
    let related_tournaments_league_method: Option<League> = 
        tournament_itce.search_league().await;
    println!(
        "The related League queried through a method of tournament: {:?}", 
        &related_tournaments_league_method
    );

    // Also, the common usage w'd be operating on data retrieve from the database, `but find_by_id`
    // returns an Option<T>, so an Option destructurement should be necessary
    let tournament: Option<Tournament> = Tournament::find_by_id(1).await;
    println!("Tournament: {:?}", &tournament);

    if let Some(trnmt) = tournament {
        let result: Option<League> = trnmt.search_league().await;
        println!("The related League as method if tournament is some: {:?}", &result);
    } else { println!("`tournament` variable contains a None value") }
    
    
    // The alternative as an associated function, passing as argument a type <K: ForeignKeyable> 
    // Data for the examples. Obviously will also work passing the above `tournament` variable as argument
    let lec: League = League {
        id: 4,
        ext_id: 1,
        slug: "LEC".to_string(),
        name: "League Europe Champions".to_string(),
        region: "EU West".to_string(),
        image_url: "https://lec.eu".to_string(),
    };

    /*  
        Finds the League (if exists) that it's the parent side of the FK relation
        for the Tournament entity.
        This does exactly the same as the `.search_league()` method, but implemented
        as an associated function. 
        There's no point on use this way, this it's just provided in the example
        because was codified before the method implementation over a T type, and
        still exists in the Canyon's code. 
        It seems a little more readable to call the method .search_parentname()
        over a T type rather than make the search with the associated method, 
        passing a reference to the instance, but it's up to the developer
        to decide which one likes more.
    */
    let related_tournaments_league: Option<League> = Tournament::belongs_to(&lec).await;
    println!(
        "The related League queried as through an associated function: {:?}", 
        &related_tournaments_league
    );

    // Finds all the tournaments that it's pointing to a concrete `League` record
    // This is usually known as the reverse side of a foreign key, but being a
    // many-to-one relation on this side
    let tournaments_belongs_to_league: Vec<Tournament> = 
        Tournament::search_by__league(&lec).await.ok().unwrap();
    println!("Tournament belongs to a league: {:?}", &tournaments_belongs_to_league);
}

/// Simple example on how to insert data into the database with a _result
/// based method, which returns `()` or Error depending on how the action
/// went when the query was performed
async fn _insert_result_example() {
    // A simple example on how to insert new data into the database
    // On the .insert() method, you always must have a &mut reference
    // to the data that you want to insert, because the .insert() query
    // will update the `Default::default()` value assigned in the
    // initialization
    let mut lec: League = League {
        id: Default::default(),
        ext_id: 1,
        slug: "AAA LEC".to_string(),
        name: "AAA League Europe Champions".to_string(),
        region: "AAAAA EU West".to_string(),
        image_url: "https://lec.eu".to_string(),
    };
    
    println!("LEC before: {:?}", &lec);

    let ins_result = lec.insert_result().await;
    
    // Now, we can handle the result returned, because it can contains a
    // critical error that may leads your program to panic
    if let Ok(_) = ins_result {
        println!("LEC after: {:?}", &lec);
    } else {
        eprintln!("{:?}", ins_result.err())
    }
}

/// Demonstration on how to perform an insert of multiple items on a table
async fn _multi_insert_example() {
    let mut new_league = League {
        id: Default::default(),
        ext_id: 392489032,
        slug: "League10".to_owned(),
        name: "League10also".to_owned(),
        region: "Turkey".to_owned(),
        image_url: "https://www.sdklafjsd.com".to_owned()
    };
    let mut new_league2 = League {
        id: Default::default(),
        ext_id: 392489032,
        slug: "League11".to_owned(),
        name: "League11also".to_owned(),
        region: "LDASKJF".to_owned(),
        image_url: "https://www.sdklafjsd.com".to_owned()
    };
    let mut new_league3 = League {
        id: Default::default(),
        ext_id: 9687392489032,
        slug: "League3".to_owned(),
        name: "3League".to_owned(),
        region: "EU".to_owned(),
        image_url: "https://www.lag.com".to_owned()
    };

    League::insert_multiple(
        &mut [&mut new_league, &mut new_league2, &mut new_league3]
    ).await
    .ok();
}


/// Example on how to update one or more columns with the associated function
/// Type::update_query()
/// 
/// In this particular one, we update multiple columns on a table
/// It will update the columns slug and image_url with
/// the provided values to all the entries on the League table which ID
/// is greater than 3
async fn _update_columns_associated_fn() {
    
    League::update_query()
        .set_clause(
            &[
                (LeagueField::slug, "Updated slug"),
                (LeagueField::image_url, "https://random_updated_url.up")
            ]
        ).where_clause(
            LeagueFieldValue::id(3), Comp::Gt
        ).query()
        .await
        .ok();
}