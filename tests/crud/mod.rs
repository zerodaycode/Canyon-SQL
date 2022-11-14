use canyon_sql::{crud::CrudOperations, runtime::tokio};

use crate::tests_models::league::League;

pub mod delete_operations;
pub mod foreign_key_operations;
pub mod insert_operations;
pub mod querybuilder_operations;
pub mod select_operations;
pub mod update_operations;

use crate::constants::PSQL_DS;
use crate::constants::SQL_SERVER_DS;

/// In order to initialize data on `SqlServer`. we must manually insert it
/// when the docker starts. SqlServer official docker from Microsoft does
/// not allow you to run `.sql` files against the database (not at least, without)
/// using a workaround. So, we are going to query the `SqlServer` to check if already
/// has some data (other processes, persistance or multi-threading envs), af if not,
/// we are going to retrieve the inserted data on the `postgreSQL` at start-up and
/// inserting into the `SqlServer` instance.
/// 
/// This will be marked as `#[ignore]`, so we can force to run first the marked as
/// ignored, check the data available, perform the necessary init operations and
/// then *cargo test <args...>* the real integration tests
#[tokio::test]
#[ignore]
async fn initialize_sql_server_docker_instance() {

    let leagues_sql = League::find_all_datasource(SQL_SERVER_DS).await.unwrap();
    let leagues = League::find_all_datasource(PSQL_DS).await.unwrap();
}