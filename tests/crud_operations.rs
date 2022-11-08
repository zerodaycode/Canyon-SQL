use std::error::Error;

///! Integration tests for the heart of a Canyon-SQL application, the CRUD operations.
///
///! This tests will tests mostly the whole source code of Canyon, due to its integration nature
///
/// Guide-style: Almost every operation in Canyon is `Result` wrapped (without the) unckecked
/// variants of the `find_all` implementations. We will go to directly `.unwrap()` the results
/// because, if there's something wrong in the code reported by the tests, we want to *panic*
/// and abort the execution.
///
/// TODO We must use, for example, the datasource versions of every CRUD method to test
/// agains *sql server* databases, and use the default datasource for test against *postgresql*
mod constants;
mod crud;
mod tests_models;
