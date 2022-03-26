/// Provides helpers to build the #[canyon] procedural like attribute macro

use proc_macro2::TokenStream;
use syn::Block;
use quote::quote;

use canyon_observer::QUERIES_TO_EXECUTE;


/// Creates a TokenScream that is used to load the data generated at compile-time
/// by the `CanyonManaged` macros again on the queries register
pub fn wire_queries_to_execute(canyon_manager_tokens: &mut Vec<TokenStream>) {
    let mut queries = String::new();

    unsafe {
        for query in &QUERIES_TO_EXECUTE {
            queries.push_str(&("[".to_owned() + query + "]"));
        }
    }

    let tokens = quote! {
        use canyon_sql::canyon_observer::QUERIES_TO_EXECUTE;
        unsafe { QUERIES_TO_EXECUTE = #queries
            .split(',')
            .map(str::to_string)
            .collect();
        }

        unsafe { println!("Queries to execute : {:?}", &QUERIES_TO_EXECUTE) };
    };
    
    canyon_manager_tokens.push(tokens)    
}

/// Generates the TokenStream that has the code written by the user
/// in the `fn main()`
pub fn _user_body_builder(func_body: Box<Block>, macro_tokens: &mut Vec<TokenStream>) {
    // Gets a Vec<Stmt> with all the staments in the body of the fn
    let function_statements = func_body.stmts;
    
    for stmt in function_statements {
        let quote = quote! {#stmt};
        let quoterino: TokenStream = quote
            .to_string()
            .parse()
            .unwrap();

        macro_tokens.push(quoterino)
    }
}

