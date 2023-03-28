use proc_macro2::TokenStream;
use quote::quote;

use crate::utils::macro_tokens::MacroTokens;

/// Generates the TokenStream for the __delete() CRUD operation
/// returning a result, indicating a possible failure querying the database
pub fn generate_delete_tokens(macro_data: &MacroTokens, table_schema_data: &String) -> TokenStream {
    let ty = macro_data.ty;

    let fields = macro_data.get_struct_fields();
    let pk = macro_data.get_primary_key_annotation();

    if let Some(primary_key) = pk {
        let pk_field = fields
            .iter()
            .find(|f| *f.to_string() == primary_key)
            .expect(
                "Something really bad happened finding the Ident for the pk field on the delete",
            );
        let pk_field_value =
            quote! { &self.#pk_field as &dyn canyon_sql::crud::bounds::QueryParameter<'_> };

        quote! {
            /// Deletes from a database entity the row that matches
            /// the current instance of a T type, returning a result
            /// indicating a possible failure querying the database.
            async fn delete(&self) -> Result<(), Box<(dyn std::error::Error + Send + Sync + 'static)>> {
                <#ty as canyon_sql::crud::Transaction<#ty>>::query(
                    format!("DELETE FROM {} WHERE {:?} = $1", #table_schema_data, #primary_key),
                    &[#pk_field_value],
                    ""
                ).await?;

                Ok(())
            }

            /// Deletes from a database entity the row that matches
            /// the current instance of a T type, returning a result
            /// indicating a possible failure querying the database with the specified datasource.
            async fn delete_datasource<'a>(&self, datasource_name: &'a str)
                -> Result<(), Box<(dyn std::error::Error + Send + Sync + 'static)>>
            {
                <#ty as canyon_sql::crud::Transaction<#ty>>::query(
                    format!("DELETE FROM {} WHERE {:?} = $1", #table_schema_data, #primary_key),
                    &[#pk_field_value],
                    datasource_name
                ).await?;

                Ok(())
            }
        }
    } else {
        // Delete operation over an instance isn't available without declaring a primary key.
        // The delete querybuilder variant must be used for the case when there's no pk declared
        quote! {
            async fn delete(&self)
                -> Result<(), Box<dyn std::error::Error + Sync + std::marker::Send>>
            {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    "You can't use the 'delete' method on a \
                    CanyonEntity that does not have a #[primary_key] annotation. \
                    If you need to perform an specific search, use the Querybuilder instead."
                ).into_inner().unwrap())
            }

            async fn delete_datasource<'a>(&self, datasource_name: &'a str)
                -> Result<(), Box<dyn std::error::Error + Sync + std::marker::Send>>
            {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    "You can't use the 'delete_datasource' method on a \
                    CanyonEntity that does not have a #[primary_key] annotation. \
                    If you need to perform an specific search, use the Querybuilder instead."
                ).into_inner().unwrap())
            }
        }
    }
}

/// Generates the TokenStream for the __delete() CRUD operation as a
/// [`query_elements::query_builder::QueryBuilder<'a, #ty>`]
pub fn generate_delete_query_tokens(
    macro_data: &MacroTokens,
    table_schema_data: &String,
) -> TokenStream {
    let ty = macro_data.ty;

    quote! {
        /// Generates a [`canyon_sql::query::DeleteQueryBuilder`]
        /// that allows you to customize the query by adding parameters and constrains dynamically.
        ///
        /// It performs an `DELETE FROM table_name`, where `table_name` it's the name of your
        /// entity but converted to the corresponding database convention,
        /// unless concrete values are set on the available parameters of the
        /// `canyon_macro(table_name = "table_name", schema = "schema")`
        fn delete_query<'a>() -> canyon_sql::query::DeleteQueryBuilder<'a, #ty> {
            canyon_sql::query::DeleteQueryBuilder::new(#table_schema_data, "")
        }

        /// Generates a [`canyon_sql::query::DeleteQueryBuilder`]
        /// that allows you to customize the query by adding parameters and constrains dynamically.
        ///
        /// It performs an `DELETE FROM table_name`, where `table_name` it's the name of your
        /// entity but converted to the corresponding database convention,
        /// unless concrete values are set on the available parameters of the
        /// `canyon_macro(table_name = "table_name", schema = "schema")`
        ///
        /// The query it's made against the database with the configured datasource
        /// described in the configuration file, and selected with the [`&str`]
        /// passed as parameter.
        fn delete_query_datasource<'a>(datasource_name: &'a str) -> canyon_sql::query::DeleteQueryBuilder<'a, #ty> {
            canyon_sql::query::DeleteQueryBuilder::new(#table_schema_data, datasource_name)
        }
    }
}
