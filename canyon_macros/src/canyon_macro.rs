/// Provides helpers to build the #[canyon] procedural like attribute macro

use proc_macro2::TokenStream;
use quote::quote;

use canyon_observer::QUERIES_TO_EXECUTE;


/// Creates a TokenScream that is used to load the data generated at compile-time
/// by the `CanyonManaged` macros again on the queries register
pub fn wire_queries_to_execute(canyon_manager_tokens: &mut Vec<TokenStream>) {
    let mut queries = String::new();

    for query in QUERIES_TO_EXECUTE.lock().unwrap().iter() {
        queries.push_str(&(query.to_owned() + "->"));
    }
    
    let tokens = quote! {
        use canyon_sql::canyon_observer::{
            QUERIES_TO_EXECUTE,
            postgresql::migrations::DatabaseSyncOperations
        };


        *QUERIES_TO_EXECUTE.lock().unwrap() = #queries
            .split("->")
            .map(str::to_string)
            .collect::<Vec<String>>();
        
        
        if QUERIES_TO_EXECUTE.lock().unwrap().len() > 1 {
            // > 1 because there's an [""] entry
            for element in QUERIES_TO_EXECUTE.lock().unwrap().iter() {
                println!("\t{}", element)
            }
        }

        DatabaseSyncOperations::from_query_register().await;
    };
    
    canyon_manager_tokens.push(tokens)    
}
