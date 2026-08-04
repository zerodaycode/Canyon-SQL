use crate::{
    MacroResult,
    utils::function_parser::FunctionParser,
};
use proc_macro::TokenStream;
use quote::quote;

pub(crate) fn generate_canyon_tokio_test_tokens(
    input: TokenStream,
) -> MacroResult {
    let function = syn::parse::<FunctionParser>(input)?;

    let visibility = function.vis;
    let signature = function.sig;
    let body = function.block.stmts;
    let attributes = function.attrs;

    Ok(quote! {
        #[test]
        #(#attributes)*
        #visibility #signature {
            canyon_sql::runtime::get_canyon_tokio_runtime()
                .handle()
                .block_on(async {
                    canyon_sql::core::Canyon::init()
                        .await
                        .expect("error initializing Canyon's connection pools");

                    async {
                        {
                            #(#body)*
                        }

                        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                    }
                    .await
                    .expect("error executing the `canyon_tokio_test` body");
                })
        }
    })
}
