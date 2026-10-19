//! Helpers shared by the integration tests.

/// One test per enabled backend, each calling `$body(datasource)`; `$body` is an
/// `async fn(&str) -> CanyonResult<()>` path, relative to the generated module.
macro_rules! per_backend {
    ($module:ident, $body:path) => {
        mod $module {
            #[cfg(feature = "postgres")]
            #[canyon_sql::macros::canyon_tokio_test]
            fn postgres() {
                $body(crate::constants::PSQL_DS).await?;
            }

            #[cfg(feature = "mssql")]
            #[canyon_sql::macros::canyon_tokio_test]
            fn mssql() {
                $body(crate::constants::SQL_SERVER_DS).await?;
            }

            #[cfg(feature = "mysql")]
            #[canyon_sql::macros::canyon_tokio_test]
            fn mysql() {
                $body(crate::constants::MYSQL_DS).await?;
            }
        }
    };
}
// So modules declared before `support` reach it as `crate::support::per_backend!`
#[allow(unused_imports)]
pub(crate) use per_backend;

pub mod test_db;

#[cfg(test)]
mod test_db_tests;
