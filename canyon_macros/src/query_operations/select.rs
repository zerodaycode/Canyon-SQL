use proc_macro2::TokenStream;
use quote::quote;

use crate::utils::helpers::*;

use crate::utils::macro_tokens::MacroTokens;

/// Generates the TokenStream for build the __find_all() CRUD 
/// associated function
pub fn generate_find_all_tokens(macro_data: &MacroTokens) -> TokenStream {
    // Destructure macro_tokens into raw data
    let (vis,ty) = (macro_data.vis, macro_data.ty);

    let table_name = database_table_name_from_struct(ty);

    // quote! {
    //     #vis async fn find_all() -> Vec<#ty> {
    //         <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__find_all(
    //             #table_name, 
    //             &[]
    //         )
    //             .await
    //             .as_response::<#ty>()
    //     }
    // }
    quote! {
        #vis fn find_all() -> query::QueryBuilder<'static, #ty> {
            <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__find_all(
                #table_name
            )
        }
    }
}


/// Generates the TokenStream for build the __find_all() CRUD 
/// associated function
pub fn generate_find_by_id_tokens(macro_data: &MacroTokens) -> TokenStream {
    // Destructure macro_tokens into raw data
    let (vis,ty) = (macro_data.vis, macro_data.ty);

    let table_name = database_table_name_from_struct(ty);

    quote! {
        #vis async fn find_by_id(id: i32) -> #ty {
            <#ty as canyon_sql::canyon_crud::crud::CrudOperations<#ty>>::__find_by_id(#table_name, id)
                .await
                .as_response::<#ty>()[0].clone()
        }
    }
}