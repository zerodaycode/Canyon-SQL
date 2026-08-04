use crate::query_operations::read::count::generate_count_operations_tokens;
use crate::query_operations::read::find_all::generate_find_all_operations_tokens;
use crate::query_operations::read::find_by_primary_key::generate_find_by_pk_operations_tokens;
use crate::query_operations::read::select_querybuilder::generate_select_querybuilder_tokens;
use crate::utils::macro_tokens::MacroTokens;
use proc_macro2::TokenStream;
use quote::quote;

mod count;
mod find_all;
mod find_by_primary_key;
pub(crate) mod foreign_key;
mod select_querybuilder;

/// Facade function that acts as the unique API for export to the real macro implementation
/// of all the generated macros for the READ operations
pub(crate) fn generate_read_operations_tokens<'a>(
    macro_data: &MacroTokens<'_>,
    table_schema_data: &'a str,
) -> syn::Result<TokenStream> {
    let ty = macro_data.ty;
    let mapper_ty = macro_data
        .retrieve_mapping_target_type()
        .as_ref()
        .unwrap_or(ty);

    let find_all_tokens =
        generate_find_all_operations_tokens(mapper_ty, table_schema_data, macro_data);
    let count_tokens = generate_count_operations_tokens(table_schema_data);
    let find_by_pk_tokens = generate_find_by_pk_operations_tokens(macro_data, table_schema_data)?;
    let read_querybuilder_ops = generate_select_querybuilder_tokens(table_schema_data);

    Ok(quote! {
        #find_all_tokens
        #read_querybuilder_ops
        #count_tokens
        #find_by_pk_tokens
    })
}
