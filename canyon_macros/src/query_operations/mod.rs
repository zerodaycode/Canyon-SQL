use crate::{
    query_operations::{
        delete::{generate_delete_entity_tokens, generate_delete_method_tokens},
        insert::{generate_insert_entity_function_tokens, generate_insert_method_tokens},
        read::{foreign_key::generate_find_by_fk_ops, generate_read_operations_tokens},
        update::{generate_update_entity_tokens, generate_update_method_tokens},
    },
    utils::{
        helpers::compute_crud_ops_mapping_target_type_with_generics, macro_tokens::MacroTokens,
    },
};
use proc_macro2::TokenStream;
use quote::quote;

pub mod delete;
pub mod insert;
pub mod read;
pub mod update;

mod consts;
mod doc_comments;

/// Generates every static CRUD implementation.
///
/// `CrudOperations` itself is provided by its blanket implementation once the
/// type implements `ReadOperations`, `InsertOperations`, `UpdateOperations`
/// and `DeleteOperations`.
pub fn impl_crud_operations_trait_for_struct(
    macro_data: &MacroTokens<'_>,
    table_schema_data: &str,
) -> syn::Result<TokenStream> {
    let read_operations = impl_read_operations_trait_for_struct(macro_data, table_schema_data)?;
    let insert_operations = impl_insert_operations_trait_for_struct(macro_data, table_schema_data)?;
    let update_operations = impl_update_operations_trait_for_struct(macro_data, table_schema_data)?;
    let delete_operations = impl_delete_operations_trait_for_struct(macro_data, table_schema_data)?;
    let transaction = impl_transaction_trait_for_struct(macro_data);

    Ok(quote! {
        #read_operations
        #insert_operations
        #update_operations
        #delete_operations
        #transaction
    })
}

/// Generates the static read implementation.
///
/// The mapping target only determines the type returned by read operations. It
/// does not switch the operation to the runtime entity API.
pub fn impl_read_operations_trait_for_struct(
    macro_data: &MacroTokens<'_>,
    table_schema_data: &str,
) -> syn::Result<TokenStream> {
    let ty = macro_data.ty;
    let (impl_generics, ty_generics, where_clause) = macro_data.generics.split_for_impl();
    let mapper_ty = compute_crud_ops_mapping_target_type_with_generics(
        ty,
        &ty_generics,
        macro_data.retrieve_mapping_target_type().as_ref(),
    );

    let methods = generate_read_operations_tokens(macro_data, table_schema_data)?;
    let foreign_key_operations = generate_find_by_fk_ops(macro_data, table_schema_data);

    Ok(quote! {
        impl #impl_generics
            canyon_sql::crud::ReadOperations<#mapper_ty> for #ty #ty_generics #where_clause {
            #methods
        }

        #foreign_key_operations
    })
}

/// Generates the static insert implementation.
pub fn impl_insert_operations_trait_for_struct(
    macro_data: &MacroTokens<'_>,
    table_schema_data: &str,
) -> syn::Result<TokenStream> {
    let ty = macro_data.ty;
    let (impl_generics, ty_generics, where_clause) = macro_data.generics.split_for_impl();

    let methods = generate_insert_method_tokens(macro_data, table_schema_data)?;

    Ok(quote! {
        impl #impl_generics canyon_sql::crud::InsertOperations for #ty #ty_generics #where_clause {
            #methods
        }
    })
}

/// Generates the static update implementation.
pub fn impl_update_operations_trait_for_struct(
    macro_data: &MacroTokens<'_>,
    table_schema_data: &str,
) -> syn::Result<TokenStream> {
    let ty = macro_data.ty;
    let (impl_generics, ty_generics, where_clause) = macro_data.generics.split_for_impl();

    let methods = generate_update_method_tokens(macro_data, table_schema_data)?;

    Ok(quote! {
        impl #impl_generics canyon_sql::crud::UpdateOperations for #ty #ty_generics #where_clause {
            #methods
        }
    })
}

/// Generates the static delete implementation.
pub fn impl_delete_operations_trait_for_struct(
    macro_data: &MacroTokens<'_>,
    table_schema_data: &str,
) -> syn::Result<TokenStream> {
    let ty = macro_data.ty;
    let (impl_generics, ty_generics, where_clause) = macro_data.generics.split_for_impl();

    let methods = generate_delete_method_tokens(macro_data, table_schema_data)?;

    Ok(quote! {
        impl #impl_generics canyon_sql::crud::DeleteOperations for #ty #ty_generics #where_clause {
            #methods
        }
    })
}

/// Generates the runtime entity CRUD implementation.
///
/// This contract is completely separate from `CrudOperations`: its methods
/// receive the entity to persist instead of operating on `self`.
pub fn impl_crud_entity_operations_trait_for_struct(
    macro_data: &MacroTokens<'_>,
    table_schema_data: &str,
) -> syn::Result<TokenStream> {
    let ty = macro_data.ty;
    let (impl_generics, ty_generics, where_clause) = macro_data.generics.split_for_impl();
    let insert_operations = generate_insert_entity_function_tokens(table_schema_data)?;
    let update_operations = generate_update_entity_tokens(table_schema_data)?;
    let delete_operations = generate_delete_entity_tokens(table_schema_data)?;

    Ok(quote! {
        impl #impl_generics canyon_sql::crud::EntityCrudOperations for #ty #ty_generics #where_clause {
            #insert_operations
            #update_operations
            #delete_operations
        }
    })
}

fn impl_transaction_trait_for_struct(macro_data: &MacroTokens<'_>) -> TokenStream {
    let ty = macro_data.ty;

    let (impl_generics, ty_generics, where_clause) = macro_data.generics.split_for_impl();

    quote! {
        impl #impl_generics canyon_sql::core::Transaction for #ty #ty_generics #where_clause {}
    }
}
