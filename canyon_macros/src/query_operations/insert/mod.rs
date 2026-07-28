mod entity;
mod method;

use crate::query_operations::insert::entity::generate_insert_entity_function_tokens;
use crate::query_operations::insert::method::generate_insert_method_tokens;
use crate::utils::macro_tokens::MacroTokens;
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate_insert_tokens(macro_data: &MacroTokens, table_schema_data: &str) -> TokenStream {
    let insert_method_ops = generate_insert_method_tokens(macro_data, table_schema_data);
    let insert_entity_ops = generate_insert_entity_function_tokens(macro_data, table_schema_data);

    quote! {
        #insert_method_ops
        #insert_entity_ops
    }
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
                     That's makes it ineligibly to be used in the insert_entity family of operations."
                ).into_inner().unwrap()
            )
        }
    }
}
