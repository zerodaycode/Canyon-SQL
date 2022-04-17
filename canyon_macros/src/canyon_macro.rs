/// Provides helpers to build the #[canyon] procedural like attribute macro

use proc_macro2::{TokenStream, Span};
use syn::{Block, spanned::Spanned};
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
            QUERIES_TO_EXECUTE, handler::DatabaseSyncOperations
        };

        unsafe { QUERIES_TO_EXECUTE = #queries
            .split("->")
            .map(str::to_string)
            .collect();
        }
        
        unsafe { println!("Queries to execute : {:?}", &QUERIES_TO_EXECUTE) };

        DatabaseSyncOperations::from_query_register().await;
    };
    
    canyon_manager_tokens.push(tokens)    
}

/// Generates the TokenStream that has the code written by the user
/// in the `fn main()`
pub fn _user_body_builder(func: syn::ItemFn, macro_tokens: &mut Vec<TokenStream>) -> Result<(), TokenStream> {
    // Gets a Vec<Stmt> with all the staments in the body of the fn
    let function_statements = func.clone().block.stmts;
    println!("\nOn func statements");
    
    for stmt in function_statements {
        println!("\nStatement: {:?}", quote! {#stmt});
        let quote = quote! {#stmt};
        // let quote_span = quote.span();
        let quoterino  = quote
        .to_string()
        .parse();
        
        println!("\nQuote span: {:?}", &quoterino.is_err());
        if quoterino.is_err() {
            println!("\n\n\nDetected error\n\n\n");
            return Err(
                quote::quote_spanned! {
                    stmt.span() =>
                    compile_error!("expected bool");
                }
            )
        } else {
            macro_tokens.push(quoterino.unwrap());
        }
    }
    println!("\n\n\nMain does not contains errors\n\n\n");
    Ok(())
}

