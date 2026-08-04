mod entity;
mod method;
mod querybuilder;

use crate::{
    query_operations::update::{
        entity::generate_update_entity_tokens as update_entity_tokens,
        method::generate_update_method_tokens as update_method_tokens,
        querybuilder::generate_update_querybuilder_tokens,
    },
    utils::macro_tokens::MacroTokens,
};
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate_update_method_tokens(
    macro_tokens: &MacroTokens,
    table_schema_data: &str,
) -> syn::Result<TokenStream> {
    let update_tokens = update_method_tokens(macro_tokens, table_schema_data)?;
    let querybuilder_tokens = generate_update_querybuilder_tokens(table_schema_data);

    Ok(quote! {
        #update_tokens
        #querybuilder_tokens
    })
}

pub fn generate_update_entity_tokens(table_schema_data: &str) -> syn::Result<TokenStream> {
    update_entity_tokens(table_schema_data)
}

mod __err {
    use proc_macro2::TokenStream;
    use quote::quote;

    pub(crate) fn generate_no_pk_err() -> TokenStream {
        quote! {
            Err(
                std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    "The type has either zero fields or exactly one that is annotated with #[primary_key].\
                     That's makes it ineligibly to be used in the update_entity family of operations."
                ).into_inner().unwrap()
            )
        }
    }
}
