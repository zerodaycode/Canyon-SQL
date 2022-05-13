/// Provides helpers to build the #[canyon] procedural like attribute macro

use proc_macro2::TokenStream;
use quote::quote;

use canyon_observer::QUERIES_TO_EXECUTE;


/// Creates a TokenScream that is used to load the data generated at compile-time
/// by the `CanyonManaged` macros again on the queries register
pub fn wire_queries_to_execute(canyon_manager_tokens: &mut Vec<TokenStream>) {
    let mut queries = String::new();

    unsafe {
        for query in &QUERIES_TO_EXECUTE {
            queries.push_str(&(query.to_owned() + "->"));
        }
    }
    
    let tokens = quote! {
        use canyon_sql::canyon_observer::{
            QUERIES_TO_EXECUTE, 
            postgresql::migrations::DatabaseSyncOperations
        };

        unsafe { QUERIES_TO_EXECUTE = #queries
            .split("->")
            .map(str::to_string)
            .collect();
        }
        
        if unsafe { QUERIES_TO_EXECUTE.len() > 1 } {
            // > 1 beacuase there's an [""] entry
            println!("Queries to execute -> {}:", unsafe { QUERIES_TO_EXECUTE.len() });
            for element in unsafe { &QUERIES_TO_EXECUTE } {
                println!("\t{}", element)
            }
        }

        DatabaseSyncOperations::from_query_register().await;
    };
    
    canyon_manager_tokens.push(tokens)    
}
