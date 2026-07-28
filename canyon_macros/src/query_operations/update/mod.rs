use crate::{
    query_operations::update::{
        entity::generate_update_entity_tokens, method::generate_update_method_tokens,
        querybuilder::generate_update_querybuilder_tokens,
    },
    utils::macro_tokens::MacroTokens,
};
use proc_macro2::TokenStream;
use quote::quote;

mod entity;
mod method;
mod querybuilder;

pub fn generate_update_tokens(macro_data: &MacroTokens, table_schema_data: &str) -> TokenStream {
    let update_method_ops = generate_update_method_tokens(macro_data, table_schema_data);
    let update_entity_ops = generate_update_entity_tokens(table_schema_data);
    let update_querybuilder_tokens = generate_update_querybuilder_tokens(table_schema_data);

    quote! {
        #update_method_ops
        #update_entity_ops
        #update_querybuilder_tokens
    }
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
