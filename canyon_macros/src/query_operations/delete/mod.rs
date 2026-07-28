use crate::{
    query_operations::delete::{
        entity::generate_delete_entity_tokens, method::generate_delete_method_tokens,
        querybuilder::generate_delete_querybuilder_tokens,
    },
    utils::macro_tokens::MacroTokens,
};
use proc_macro2::TokenStream;
use quote::quote;

mod entity;
mod method;
mod querybuilder;

pub fn generate_delete_tokens(macro_data: &MacroTokens, table_schema_data: &str) -> TokenStream {
    let delete_method_ops = generate_delete_method_tokens(macro_data, table_schema_data);
    let delete_entity_ops = generate_delete_entity_tokens(table_schema_data);
    let delete_querybuilder_tokens = generate_delete_querybuilder_tokens(table_schema_data);

    quote! {
        #delete_method_ops
        #delete_entity_ops
        #delete_querybuilder_tokens
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
                     That's makes it ineligibly to be used in the DELETE family of operations."
                ).into_inner().unwrap()
            )
        }
    }
}
