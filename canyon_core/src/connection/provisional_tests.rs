

// TODO: && NOTE: tests defined below should be integration tests, unfortunately, since they require a new connection to be made
// Or just to split them further, and just unit test the url string generation from the actual connection instantion
// #[cfg(test)]
// mod connection_tests {
//     use tokio;
//     use super::connection_helpers::*;
//     use crate::{db_connector::DatabaseConnection, datasources::{Auth, DatasourceConfig, DatasourceProperties, PostgresAuth}};

//     #[tokio::test]
//     #[cfg(feature = "postgres")]
//     async fn test_create_postgres_connection() {
//         use crate::datasources::PostgresAuth;

//         let config = DatasourceConfig {
//             name: "PostgresDs".to_string(),
//             auth: Auth::Postgres(PostgresAuth::Basic {
//                 username: "test_user".to_string(),
//                 password: "test_password".to_string(),
//             }),
//             properties: DatasourceProperties {
//                 host: "localhost".to_string(),
//                 port: Some(5432),
//                 db_name: "test_db".to_string(),
//                 migrations: None
//             },
//         };

//         let result = create_postgres_connection(&config).await;
//         assert!(result.is_ok());
//     }

//     #[tokio::test]
//     #[cfg(feature = "mssql")]
//     async fn test_create_sqlserver_connection() {
//         use crate::datasources::SqlServerAuth;

//         let config = DatasourceConfig {
//             name: "SqlServerDs".to_string(),
//             auth: Auth::SqlServer(SqlServerAuth::Basic {
//                 username: "test_user".to_string(),
//                 password: "test_password".to_string(),
//             }),
//             properties: DatasourceProperties {
//                 host: "localhost".to_string(),
//                 port: Some(1433),
//                 db_name: "test_db".to_string(),
//                 migrations: None
//             },
//         };

//         let result = create_sqlserver_connection(&config).await;
//         assert!(result.is_ok());
//     }

//     #[tokio::test]
//     #[cfg(feature = "mysql")]
//     async fn test_create_mysql_connection() {
//         use crate::datasources::MySQLAuth;

//         let config = DatasourceConfig {
//             name: "MySQLDs".to_string(),
//             auth: Auth::MySQL(MySQLAuth::Basic {
//                 username: "test_user".to_string(),
//                 password: "test_password".to_string(),
//             }),
//             properties: DatasourceProperties {
//                 host: "localhost".to_string(),
//                 port: Some(3306),
//                 db_name: "test_db".to_string(),
//                 migrations: None,
//             },
//         };

//         let result = create_mysql_connection(&config).await;
//         assert!(result.is_ok());
//     }

//     #[tokio::test]
//     async fn test_database_connection_new() {
//         #[cfg(feature = "postgres")]
//         {
//             use crate::datasources::PostgresAuth;

//             let config = DatasourceConfig {
//                 name: "PostgresDs".to_string(),
//                 auth: Auth::Postgres(PostgresAuth::Basic {
//                     username: "test_user".to_string(),
//                     password: "test_password".to_string(),
//                 }),
//                 properties: DatasourceProperties {
//                     host: "localhost".to_string(),
//                     port: Some(5432),
//                     db_name: "test_db".to_string(),
//                     migrations: None
//                 },
//             };

//             let result = DatabaseConnection::new(&config).await;
//             assert!(result.is_ok());
//         }

//         // #[cfg(feature = "mssql")]
//         // {
//         //     let config = DatasourceConfig {
//         //         db_type: DatabaseType::SqlServer,
//         //         auth: Auth::SqlServer(SqlServerAuth::Basic {
//         //             username: "test_user".to_string(),
//         //             password: "test_password".to_string(),
//         //         }),
//         //         properties: crate::datasources::Properties {
//         //             host: "localhost".to_string(),
//         //             port: Some(1433),
//         //             db_name: "test_db".to_string(),
//         //         },
//         //     };

//         //     let result = DatabaseConnection::new(&config).await;
//         //     assert!(result.is_ok());
//         // }

//         // #[cfg(feature = "mysql")]
//         // {
//         //     let config = DatasourceConfig {
//         //         db_type: DatabaseType::MySQL,
//         //         auth: Auth::MySQL(MySQLAuth::Basic {
//         //             username: "test_user".to_string(),
//         //             password: "test_password".to_string(),
//         //         }),
//         //         properties: crate::datasources::Properties {
//         //             host: "localhost".to_string(),
//         //             port: Some(3306),
//         //             db_name: "test_db".to_string(),
//         //         },
//         //     };

//         //     let result = DatabaseConnection::new(&config).await;
//         //     assert!(result.is_ok());
//         // }
//     }
// }

