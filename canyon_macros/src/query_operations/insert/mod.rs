mod entity;
mod method;

use crate::{
    query_operations::insert::{
        entity::generate_insert_entity_function_tokens as insert_entity_function_tokens,
        method::generate_insert_method_tokens as insert_method_tokens,
    },
    utils::macro_tokens::MacroTokens,
};
use proc_macro2::TokenStream;

pub fn generate_insert_method_tokens(
    macro_tokens: &MacroTokens,
    table_schema_data: &str,
) -> syn::Result<TokenStream> {
    insert_method_tokens(macro_tokens, table_schema_data)
}

pub fn generate_insert_entity_function_tokens(table_schema_data: &str) -> syn::Result<TokenStream> {
    insert_entity_function_tokens(table_schema_data)
}

mod __shared {
    use proc_macro2::TokenStream;
    use quote::quote;

    pub(crate) fn no_fields_to_insert_err() -> TokenStream {
        quote! {
            Err(
                std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    "The type has either zero fields or exactly one that is annotated with #[primary_key].\
                     That's makes it ineligibly to be used in the INSERT family of operations."
                ).into_inner().unwrap()
            )
        }
    }
}
