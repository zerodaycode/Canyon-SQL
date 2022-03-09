use proc_macro2::TokenStream;
use quote::quote;

use crate::query_operations::utils::helpers::*;
use crate::query_operations::utils::macro_tokens::MacroTokens;

/// Generates the TokenStream for the __delete() CRUD operation
pub fn generate_delete_tokens(macro_data: &MacroTokens) -> TokenStream {
    
    // Destructure macro_tokens into raw data
    let (vis,ty) = (macro_data.vis, macro_data.ty);

    // Gets the name of the table in the database that maps the annotated Struct
    let table_name = database_table_name_from_struct(ty);

    // Retrives the fields of the Struct
    let fields = macro_data.get_struct_fields();

    // Gets a reference to the id of the instance
    let delete_id = fields.iter().map( |ident| {
        quote! { &self.id }
    });
    
    quote! {
        #vis async fn delete(&self) -> () {
            <#ty as CrudOperations<#ty>>::__delete(
                #table_name, 
                &[ #delete_id ]
            ).await;
        }
    }
}