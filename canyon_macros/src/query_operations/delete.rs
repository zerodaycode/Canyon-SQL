use proc_macro2::TokenStream;
use quote::quote;

use crate::utils::helpers::*;
use crate::utils::macro_tokens::MacroTokens;

/// Generates the TokenStream for the __delete() CRUD operation
pub fn generate_delete_tokens(macro_data: &MacroTokens) -> TokenStream {
    
    // Destructure macro_tokens into raw data
    let (vis, ty) = (macro_data.vis, macro_data.ty);

    // Gets the name of the table in the database that maps the annotated Struct
    let table_name = database_table_name_from_struct(ty);

    quote! {
        /// Deletes from a database entity the row that matches
        /// the current instance of a T type
        #vis async fn delete(&self) -> () {
            <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__delete(
                #table_name, 
                self.id
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
    
    // Destructure macro_tokens into raw data
    let (vis, ty) = (macro_data.vis, macro_data.ty);

    // Gets the name of the table in the database that maps the annotated Struct
    let table_name = database_table_name_from_struct(ty);

    quote! {
        /// Deletes from a database entity the row that matches
        /// the current instance of a T type, returning a result
        /// indicating a posible failure querying the database.
        #vis async fn delete_result(&self) -> Result<(), canyon_sql::tokio_postgres::Error> {
            let result = <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__delete(
                #table_name, 
                self.id
            ).await;

            if let Err(error) = result {
                Err(error)
            } else { Ok(()) }
        }
    }
}

/// Generates the TokenStream for the __delete() CRUD operation as a 
/// [`query_elements::query_builder::QueryBuilder<'static, #ty>`]
pub fn generate_delete_query_tokens(macro_data: &MacroTokens) -> TokenStream {
    
    // Destructure macro_tokens into raw data
    let (vis, ty) = (macro_data.vis, macro_data.ty);

    // Gets the name of the table in the database that maps the annotated Struct
    let table_name = database_table_name_from_struct(ty);

    quote! {
        #vis fn delete_query(&self) -> query_elements::query_builder::QueryBuilder<'static, #ty> {
            <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__delete_query(
                #table_name
            )
        }
    }
}