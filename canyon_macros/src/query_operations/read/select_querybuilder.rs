use proc_macro2::TokenStream;
use quote::quote;

pub fn generate_select_querybuilder_tokens(table_schema_data: &str) -> TokenStream {
    quote! {
        fn select_query<'a>() -> Result<canyon_sql::query::querybuilder::SelectQueryBuilder<'a>, Box<dyn std::error::Error + Send + Sync + 'a>> {
            let default_db_type = canyon_sql::core::Canyon::instance()?.get_default_db_type()?;
            Ok(canyon_sql::query::querybuilder::SelectQueryBuilder::new(#table_schema_data, default_db_type))
        }

        fn select_query_with<'a>(database_type: canyon_sql::connection::DatabaseType)
            -> Result<canyon_sql::query::querybuilder::SelectQueryBuilder<'a>, Box<dyn std::error::Error + Send + Sync + 'a>> {
            Ok(canyon_sql::query::querybuilder::SelectQueryBuilder::new(#table_schema_data, database_type))
        }
    }
}
