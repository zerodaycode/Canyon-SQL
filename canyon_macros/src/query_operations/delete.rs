use proc_macro2::TokenStream;
use quote::quote;

use crate::utils::helpers::*;
use crate::utils::macro_tokens::MacroTokens;

/// Generates the TokenStream for the __delete() CRUD operation
pub fn generate_delete_tokens(macro_data: &MacroTokens) -> TokenStream {
    let (vis, ty) = (macro_data.vis, macro_data.ty);
    let table_name = database_table_name_from_struct(ty);

    let fields = macro_data.get_struct_fields();
    let pk = macro_data.get_primary_key_annotation()
        .unwrap_or_default();
    let pk_field = fields.iter().find( |f| 
        *f.to_string() == pk
    ).expect("Failed to obtain the value of the primary key for the delete operation");


    quote! {
        /// Deletes from a database entity the row that matches
        /// the current instance of a T type
        #vis async fn delete(&self) {
            <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__delete(
                #table_name,
                #pk,
                self.id,
                ""
            ).await
            .ok()
            .expect(
                format!(
                    "Delete operation failed for {:?}", 
                    &self
                ).as_str()
            );
        }

        /// Deletes from a database entity the row that matches
        /// the current instance of a T type with the specified datasource
        #vis async fn delete_datasource<'a>(&self, datasource_name: &'a str) {
            <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__delete(
                #table_name,
                #pk,
                self.id,
                datasource_name
            ).await
            .ok()
            .expect(
                format!(
                    "Delete operation failed for {:?}", 
                    &self
                ).as_str()
            );
        }
    }
}

/// Generates the TokenStream for the __delete() CRUD operation
/// returning a result, indicating a posible failure querying the database
pub fn generate_delete_result_tokens(macro_data: &MacroTokens) -> TokenStream {
    let (vis, ty) = (macro_data.vis, macro_data.ty);
    let table_name = database_table_name_from_struct(ty);

    let fields = macro_data.get_struct_fields();
    let pk = macro_data.get_primary_key_annotation()
        .unwrap_or_default();
    let pk_field = fields.iter().find( |f| 
        *f.to_string() == pk
    ).expect("Failed to obtain the value of the primary key for the delete operation");

    quote! {
        /// Deletes from a database entity the row that matches
        /// the current instance of a T type, returning a result
        /// indicating a posible failure querying the database.
        #vis async fn delete_result(&self) -> Result<(), Box<(dyn std::error::Error + Send + Sync + 'static)>> {
            let result = <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__delete(
                #table_name,
                #pk,
                self.#pk_field,
                ""
            ).await;

            if let Err(error) = result {
                Err(error)
            } else { Ok(()) }
        }

        /// Deletes from a database entity the row that matches
        /// the current instance of a T type, returning a result
        /// indicating a posible failure querying the database with the specified datasource.
        #vis async fn delete_result_datasource<'a>(&self, datasource_name: &'a str) -> 
            Result<(), Box<(dyn std::error::Error + Send + Sync + 'static)>>
        {
            let result = <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__delete(
                #table_name,
                #pk,
                self.#pk_field,
                datasource_name
            ).await;

            if let Err(error) = result {
                Err(error)
            } else { Ok(()) }
        }
    }
}

/// Generates the TokenStream for the __delete() CRUD operation as a 
/// [`query_elements::query_builder::QueryBuilder<'a, #ty>`]
pub fn generate_delete_query_tokens(macro_data: &MacroTokens) -> TokenStream {
    let (vis, ty) = (macro_data.vis, macro_data.ty);
    let table_name = database_table_name_from_struct(ty);

    quote! {
        /// Deletes a record on a table for the target database that matches the value
        /// of the primary key of the instance
        #vis fn delete_query<'a>(&self) -> query_elements::query_builder::QueryBuilder<'a, #ty> {
            <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__delete_query(
                #table_name, ""
            )
        }

        /// Deletes a record on a table for the target database with the specified
        /// values generated with the [`Querybuilder`] and with the
        #vis fn delete_query_datasource<'a>(&self, datasource_name: &'a str) -> 
            query_elements::query_builder::QueryBuilder<'a, #ty> 
        {
            <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__delete_query(
                #table_name, datasource_name
            )
        }
    }
}