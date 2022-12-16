use proc_macro2::TokenStream;
use quote::quote;

use crate::utils::macro_tokens::MacroTokens;

/// Generates the TokenStream for the __delete() CRUD operation
/// returning a result, indicating a posible failure querying the database
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
            quote! { &self.#pk_field as &dyn canyon_sql::crud::bounds::QueryParameters<'_> };

        quote! {
            /// Deletes from a database entity the row that matches
            /// the current instance of a T type, returning a result
            /// indicating a posible failure querying the database.
            async fn delete(&self) -> Result<(), Box<(dyn std::error::Error + Send + Sync + 'static)>> {
                let stmt = format!("DELETE FROM {} WHERE {:?} = $1", #table_schema_data, #primary_key);

                let result = <#ty as canyon_sql::crud::Transaction<#ty>>::query(
                    stmt,
                    &[#pk_field_value],
                    ""
                ).await;

                if let Err(error) = result {
                    Err(error)
                } else { Ok(()) }
            }

            /// Deletes from a database entity the row that matches
            /// the current instance of a T type, returning a result
            /// indicating a posible failure querying the database with the specified datasource.
            async fn delete_datasource<'a>(&self, datasource_name: &'a str)
                -> Result<(), Box<(dyn std::error::Error + Send + Sync + 'static)>>
            {
                let stmt = format!("DELETE FROM {} WHERE {:?} = $1", #table_schema_data, #primary_key);

                let result = <#ty as canyon_sql::crud::Transaction<#ty>>::query(
                    stmt,
                    &[#pk_field_value],
                    datasource_name
                ).await;

                if let Err(error) = result {
                    Err(error)
                } else { Ok(()) }
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
        // /// Deletes a record on a table for the target database that matches the value
        // /// of the primary key of the instance
        // fn delete_query<'a>() -> canyon_sql::query::QueryBuilder<'a, #ty> {
        //     canyon_sql::query::Query::generate(format!("DELETE FROM {}", #table_schema_data), "")
        // }

        // /// Deletes a record on a table for the target database with the specified
        // /// values generated with the [`Querybuilder`] and with the
        // fn delete_query_datasource(datasource_name: &str) -> canyon_sql::query::QueryBuilder<'_, #ty> {
        //     canyon_sql::query::Query::generate(format!("DELETE FROM {}", #table_schema_data), datasource_name)
        // }
    }
}
