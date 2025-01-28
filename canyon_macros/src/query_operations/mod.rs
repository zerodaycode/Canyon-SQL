use proc_macro2::TokenStream;
use quote::quote;
use crate::query_operations::delete::generate_delete_tokens;
use crate::query_operations::foreign_key::generate_find_by_fk_ops;
use crate::query_operations::insert::generate_insert_tokens;
use crate::query_operations::read::generate_read_operations_tokens;
use crate::query_operations::update::generate_update_tokens;
use crate::utils::macro_tokens::MacroTokens;

pub mod delete;
pub mod foreign_key;
pub mod insert;
pub mod read;
pub mod update;

mod doc_comments;
mod macro_template;
mod consts;


pub fn impl_crud_operations_trait_for_struct(
    macro_data: &MacroTokens<'_>,
    table_schema_data: String,
) -> proc_macro::TokenStream {
    let mut crud_ops_tokens = TokenStream::new();
    let ty = macro_data.ty;

    let read_operations_tokens = generate_read_operations_tokens(macro_data, &table_schema_data);
    let insert_tokens = generate_insert_tokens(macro_data, &table_schema_data);
    let update_tokens = generate_update_tokens(macro_data, &table_schema_data);
    let delete_tokens = generate_delete_tokens(macro_data, &table_schema_data);

    let crud_operations_tokens = quote! {
        #read_operations_tokens
        #insert_tokens
        #update_tokens
        #delete_tokens
    };

    crud_ops_tokens.extend(quote!{
        use canyon_sql::core::IntoResults;

        #[canyon_sql::macros::async_trait] // TODO: get rid of the async_trait
        impl canyon_sql::crud::CrudOperations<#ty> for #ty {
            #crud_operations_tokens
        }

        impl canyon_sql::core::Transaction<#ty> for #ty {}
    });

    let foreign_key_ops_tokens = generate_find_by_fk_ops(macro_data, &table_schema_data);
    crud_ops_tokens.extend(quote!{ #foreign_key_ops_tokens });

    crud_ops_tokens.into()
}