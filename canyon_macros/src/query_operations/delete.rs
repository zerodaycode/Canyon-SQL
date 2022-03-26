use proc_macro2::TokenStream;
use quote::quote;

use crate::utils::helpers::*;
use crate::utils::macro_tokens::MacroTokens;

/// Generates the TokenStream for the __delete() CRUD operation
pub fn generate_delete_tokens(macro_data: &MacroTokens) -> TokenStream {
    
    // Destructure macro_tokens into raw data
    let (vis,ty) = (macro_data.vis, macro_data.ty);

    // Gets the name of the table in the database that maps the annotated Struct
    let table_name = database_table_name_from_struct(ty);

    quote! {
        #vis async fn delete(&self) -> () {
            <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__delete(
                #table_name, 
                self.id
            ).await;
        }
    }
}