use proc_macro2::TokenStream;
use quote::quote;

/// Generates the TokenStream for the __delete() CRUD operation as a
/// [`query_elements::query_builder::QueryBuilder<'a, #ty>`]
pub(crate) fn generate_delete_querybuilder_tokens(table_schema_data: &str) -> TokenStream {
    quote! {
        /// Generates a [`canyon_sql::query::querybuilder::DeleteQueryBuilder`]
        /// that allows you to customize the query by adding parameters and constrains dynamically.
        ///
        /// It performs an `DELETE FROM table_name`, where `table_name` it's the name of your
        /// entity but converted to the corresponding database convention,
        /// unless concrete values are set on the available parameters of the
        /// `canyon_macro(table_name = "table_name", schema = "schema")`
        fn delete_query<'canyon, 'err>() ->
            canyon_sql::query::querybuilder::DeleteQueryBuilder<'canyon> where 'canyon: 'err {
            canyon_sql::query::querybuilder::DeleteQueryBuilder::new(#table_schema_data)
        }

        /// Generates a [`canyon_sql::query::querybuilder::DeleteQueryBuilder`]
        /// that allows you to customize the query by adding parameters and constrains dynamically.
        ///
        /// It performs an `DELETE FROM table_name`, where `table_name` it's the name of your
        /// entity but converted to the corresponding database convention,
        /// unless concrete values are set on the available parameters of the
        /// `canyon_macro(table_name = "table_name", schema = "schema")`
        ///
        /// The query it's made against the database with the configured datasource
        /// described in the configuration file, selected with the input parameter
        fn delete_query_with<'canyon, 'err>(database_type: canyon_sql::connection::DatabaseType)
        -> canyon_sql::query::querybuilder::DeleteQueryBuilder<'canyon> where 'canyon: 'err {
            canyon_sql::query::querybuilder::DeleteQueryBuilder::new_for(#table_schema_data, database_type)
        }
    }
}
