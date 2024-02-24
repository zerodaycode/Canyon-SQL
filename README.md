<div align="center">
<h1>CANYON-SQL</h1>
  <p align="center">
    <h3><strong>A full written in `Rust` ORM for multiple databases</strong></h3>
    <h4>`Canyon-SQL` is a high level abstraction for working with multiple databases concurrently. Is build on top of the `async` language features
to provide a high speed, high performant library to handling data access for consumers.</h4>
    <br />
    <br />
</div>
<div align="center">

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)

![crates.io](https://img.shields.io/crates/v/canyon_sql?style=for-the-badge)

[![Continuous Integration](https://github.com/zerodaycode/Canyon-SQL/actions/workflows/continuous-integration.yml/badge.svg)](https://github.com/zerodaycode/Canyon-SQL/actions/workflows/continuous-integration.yml)
[![Code Quality](https://github.com/zerodaycode/Canyon-SQL/actions/workflows/code-quality.yml/badge.svg)](https://github.com/zerodaycode/Canyon-SQL/actions/workflows/code-quality.yml)
[![Code Coverage Measure](https://zerodaycode.github.io/Canyon-SQL/badges/flat.svg)](https://zerodaycode.github.io/Canyon-SQL)
[![Code Coverage Status](https://github.com/zerodaycode/Canyon-SQL/actions/workflows/code-coverage.yml/badge.svg)](https://github.com/zerodaycode/Canyon-SQL/actions/workflows/code-coverage.yml)
</div>

## Early stage disclaimer

The library it's still on an `early stage` state.
Any contrib via `fork` + `PR` it's really appreciated. Currently we are involved in a really active development on the project.

## :memo: Full documentation resources

There is a `work-in-progress` web page, build with `mdBook` containing the official documentation.
Here is where you will find all the technical documentation for `Canyon-SQL`.
You can read it [by clicking this link](https://zerodaycode.github.io/canyon-book/)

## :pushpin: Most important features

- **Async** by default. Almost every functionality provided is ready to be consumed concurrently.
- Use of multiple datasources. You can query multiple databases at the same time, even different ones! This means that you will be able to query concurrently a `PostgreSQL` database and a `SqlServer` one in the same project.
- Is macro based. With a few annotations and a configuration file, you are ready to write your data access.
- Allows **migrations**. `Canyon-SQL` comes with a *god-mode* that will manage every table on your database for you. You can modify in `Canyon` code your tables internally, altering columns, setting up constraints... Also, in the future, we have plans to allow you to manipulate the whole server, like creating databases, altering configurations... everything, but in a programmatically approach with `Canyon`!

## Supported databases

`Canyon-SQL` currently has support for work with the following databases:

- PostgreSQL (via `tokio-postgres` crate)
- SqlServer (via `tiberius` crate)

Every crate listed above is an `async` based crate, in line with the guidelines of the `Canyon-SQL` design.

There are plans to include more databases engines.

## Better by example

Let's take a look to see how the `Canyon` code looks like!

### The classical SELECT * FROM {table_name}

```rust
let find_all_result: Result<Vec<League>, Box<dyn Error + Send + Sync>> =  League::find_all().await;

// Connection doesn't return an error
assert!(find_all_result.is_ok());
// We retrieved elements from the League table
assert!(!find_all_result.unwrap().is_empty());
```

### :mag_right: Performing a search over the primary key column

```rust
let find_by_pk_result: Result<Option<League>, Box<dyn Error + Send + Sync>> = League::find_by_pk(&1).await;

assert!(find_by_pk_result.as_ref().unwrap().is_some());

let some_league = find_by_pk_result.unwrap().unwrap();
assert_eq!(some_league.id, 1);
assert_eq!(some_league.ext_id, 100695891328981122_i64);
assert_eq!(some_league.slug, "european-masters");
assert_eq!(some_league.name, "European Masters");
assert_eq!(some_league.region, "EUROPE");
assert_eq!(
    some_league.image_url,
    "http://static.lolesports.com/leagues/EM_Bug_Outline1.png"
);
```

Note the leading reference on the `find_by_pk(...)` parameter. This associated function receives an `&dyn QueryParameter<'_>` as argument, not a value.

### :wrench: Building more complex queries

To exemplify the capabilities of `Canyon`, we will use `SelectQueryBuilder<T>`, which implements the `QueryBuilder<T>` trait
to build a more complex where, filtering data and joining tables.

```rust
let mut select_with_joins = LeagueTournament::select_query();
    select_with_joins
        .inner_join("tournament", "league.id", "tournament.league_id")
        .left_join("team", "tournament.id", "player.tournament_id")
        .r#where(LeagueFieldValue::id(&7), Comp::Gt)
        .and(LeagueFieldValue::name(&"KOREA"), Comp::Eq)
        .and_values_in(LeagueField::name, &["LCK", "STRANGER THINGS"]);
    // NOTE: We don't have in the docker the generated relationships
    // with the joins, so for now, we are just going to check that the
    // generated SQL by the SelectQueryBuilder<T> is the spected
    assert_eq!(
        select_with_joins.read_sql(),
        "SELECT * FROM league INNER JOIN tournament ON league.id = tournament.league_id LEFT JOIN team ON tournament.id = player.tournament_id WHERE id > $1 AND name = $2  AND name IN ($2, $3) "
    )
```

> [!NOTE]
>
> For now, when you use joins, you will need to create a new model with the columns in both tables (in case that you desire the data in such columns), but just follows the usual process with the CanyonMapper.
It will try to retrieve the data for every field declared. If you don't declare a field that is in the open clause, in this case (*), that field won't be retrieved. No problem. But if you have fields that aren't mapable with some column in the database, the program will panic.

## More examples

If you want to see more examples, you can take a look into the `tests` folder, at the root of this repository. Every available database operation is tested there, so you can use it to find the usage of the described operations in the documentation mentioned above.

## :octocat: Contributing to CANYON-SQL

First of all, thanks for taking in consideration helping us with the project.
You can take a look to our [templated guide](./CONTRIBUTING.md).

But, to summarize:

- Take a look at the already opened issues, to verify if it already exists or if someone is already taking care about solving it. Even though, you can enter to participate and explain your point of view, or even help to accomplish the task.
- Make a fork of `Canyon-SQL`
- If you opened an issue, create a branch from the base branch of the repo (that's the default), and point it to your fork.
- After completing your changes, open a `PR` to the default branch. Fill the template provided in the best way possible.
- Wait for the approval. In most of cases, a test over the feature will be required before approving your changes.

## :question: What about the tests?

Typically in `Canyon`, isolated unit tests are written as doc-tests, and the integration ones are under the folder `./tests`

If you want to run the tests (because this is the first thing that you want to do after fork the repo), before moving forward, there are a couple of things that have to be considered.

- You will need Docker installed in the target machine.
- If you have Docker, and `Canyon-SQL` cloned of forked, you can run our docker-compose file `(docker/docker-compose.yml)`, which will initialize a `PostgreSQL` database and will put content on it to make the tests able to work.
- Finally, some tests run against `MSSQL`. We didn't found a nice way of inserting data directly when the Docker wakes up, but instead, we run a very special test located at `tests/crud/mod.rs`, that is named `initialize_sql_server_docker_instance`. When you run this one, initial data will be inserted into the tables that are created when this test run.
(If you know a better way of doing this, please, open an issue to let us know, and improve this process!)
