//! Provides helpers to build the `#[canyon_macros::canyon]` procedural like attribute macro
#![cfg(feature = "migrations")]

use canyon_core::connection::get_canyon_tokio_runtime;
use canyon_migrations::migrations::handler::Migrations;
use canyon_migrations::{CM_QUERIES_TO_EXECUTE, QUERIES_TO_EXECUTE};
use proc_macro2::TokenStream;
use quote::quote;

pub fn main_with_queries() -> TokenStream {
    // TODO: migrations on main instead of main_with_queries
    get_canyon_tokio_runtime().block_on(async {
        canyon_core::connection::Canyon::init()
            .await
            .expect("Error initializing the connections POOL");
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
fn wire_queries_to_execute(canyon_manager_tokens: &mut Vec<TokenStream>) {
    let data_to_wire = if let Some(mutex) = QUERIES_TO_EXECUTE.get() {
        let queries = mutex.lock().expect("QUERIES_TO_EXECUTE poisoned");
        queries
            .iter()
            .map(|(key, value)| {
                quote! { hm.insert(#key, vec![#(#value),*]); }
            })
            .collect::<Vec<_>>()
    } else {
        vec![]
    };

    let cm_data_to_wire = if let Some(mutex) = CM_QUERIES_TO_EXECUTE.get() {
        let cm_queries = mutex.lock().expect("CM_QUERIES_TO_EXECUTE poisoned");
        cm_queries
            .iter()
            .map(|(key, value)| {
                quote! { cm_hm.insert(#key, vec![#(#value),*]); }
            })
            .collect::<Vec<_>>()
    } else {
        vec![]
    };

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

    canyon_manager_tokens.push(tokens);
}
