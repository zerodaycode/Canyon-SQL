//! Provides helpers to build the `#[canyon_macros::canyon]` procedural like attribute macro

use proc_macro2::TokenStream;
use quote::quote;
use canyon_connection::CANYON_TOKIO_RUNTIME;
use canyon_migrations::{CM_QUERIES_TO_EXECUTE, QUERIES_TO_EXECUTE};
use canyon_migrations::migrations::handler::Migrations;

#[cfg(feature = "migrations")]
pub fn main_with_queries() -> TokenStream {
    CANYON_TOKIO_RUNTIME.block_on(async {
        canyon_connection::init_connections_cache().await;
        Migrations::migrate().await;
    });

    // The queries to execute at runtime in the managed state
    let mut queries_tokens: Vec<TokenStream> = Vec::new();
    wire_queries_to_execute(&mut queries_tokens);
    quote! {
        {
            #(#queries_tokens)*
        }
    }
}

/// Creates a TokenScream that is used to load the data generated at compile-time
/// by the `CanyonManaged` macros again on the queries register
#[cfg(feature = "migrations")]
fn wire_queries_to_execute(canyon_manager_tokens: &mut Vec<TokenStream>) {
    let cm_data = CM_QUERIES_TO_EXECUTE.lock().unwrap();
    let data = QUERIES_TO_EXECUTE.lock().unwrap();

    let cm_data_to_wire = cm_data.iter().map(|(key, value)| {
        quote! { cm_hm.insert(#key, vec![#(#value),*]); }
    });
    let data_to_wire = data.iter().map(|(key, value)| {
        quote! { hm.insert(#key, vec![#(#value),*]); }
    });

    let tokens = quote! {
        use std::collections::HashMap;
        use canyon_sql::migrations::processor::MigrationsProcessor;

        let mut cm_hm: HashMap<&str, Vec<&str>> = HashMap::new();
        let mut hm: HashMap<&str, Vec<&str>> = HashMap::new();

        #(#cm_data_to_wire)*;
        #(#data_to_wire)*;

        MigrationsProcessor::from_query_register(&cm_hm).await;
        MigrationsProcessor::from_query_register(&hm).await;
    };

    canyon_manager_tokens.push(tokens)
}
