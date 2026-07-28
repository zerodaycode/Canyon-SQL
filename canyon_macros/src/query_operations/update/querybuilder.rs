use proc_macro2::TokenStream;
use quote::quote;

/// Generates the TokenStream for the __update() CRUD operation
/// being the query generated with the [`QueryBuilder`]
pub(crate) fn generate_update_querybuilder_tokens(table_schema_data: &str) -> TokenStream {
    quote! {
        /// Generates a [`canyon_sql::query::querybuilder::UpdateQueryBuilder`]
        /// that allows you to customize the query by adding parameters and constrains dynamically.
        ///
        /// It performs an `UPDATE table_name`, where `table_name` it's the name of your
        /// entity but converted to the corresponding database convention,
        /// unless concrete values are set on the available parameters of the
        /// `canyon_macro(table_name = "table_name", schema = "schema")`
        fn update_query<'a>() -> canyon_sql::query::querybuilder::UpdateQueryBuilder<'a> {
            canyon_sql::query::querybuilder::UpdateQueryBuilder::new(#table_schema_data)
        }

        /// Generates a [`canyon_sql::query::querybuilder::UpdateQueryBuilder`]
        /// that allows you to customize the query by adding parameters and constrains dynamically.
        ///
        /// It performs an `UPDATE table_name`, where `table_name` it's the name of your
        /// entity but converted to the corresponding database convention,
        /// unless concrete values are set on the available parameters of the
        /// `canyon_macro(table_name = "table_name", schema = "schema")`
        ///
        /// The query it's made against the database with the configured datasource
        /// described in the configuration file, and selected with the input parameter
        fn update_query_with<'a>(database_type: canyon_sql::connection::DatabaseType) ->
            canyon_sql::query::querybuilder::UpdateQueryBuilder<'a> {
            canyon_sql::query::querybuilder::UpdateQueryBuilder::new_for(#table_schema_data, database_type)
        }
    }
}
