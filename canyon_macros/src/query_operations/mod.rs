use crate::query_operations::delete::generate_delete_tokens;
use crate::query_operations::foreign_key::generate_find_by_fk_ops;
use crate::query_operations::insert::generate_insert_tokens;
use crate::query_operations::read::generate_read_operations_tokens;
use crate::query_operations::update::generate_update_tokens;
use crate::utils::macro_tokens::MacroTokens;
use proc_macro2::TokenStream;
use quote::quote;

pub mod delete;
pub mod foreign_key;
pub mod insert;
pub mod read;
pub mod update;

mod consts;
mod doc_comments;

pub fn impl_crud_operations_trait_for_struct(
    macro_data: &MacroTokens<'_>,
    table_schema_data: String,
) -> proc_macro::TokenStream {
    let mut crud_ops_tokens = TokenStream::new();

    let ty = macro_data.ty;
    let mapper_ty = macro_data
        .retrieve_mapping_target_type()
        .expect("Expected mapping macro data")
        .unwrap_or_else(|| ty.clone());

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

    crud_ops_tokens.extend(quote! {
        use canyon_sql::core::IntoResults;
        use canyon_sql::core::RowMapper;

        impl canyon_sql::crud::CrudOperations<#mapper_ty> for #ty {
            #crud_operations_tokens
        }

        impl canyon_sql::core::Transaction for #ty {}
    });

    // NOTE: this extends should be documented WHY is needed to be after the base impl of CrudOperations
    let foreign_key_ops_tokens = generate_find_by_fk_ops(macro_data, &table_schema_data);
    crud_ops_tokens.extend(quote! { #foreign_key_ops_tokens });

    crud_ops_tokens.into()
}
