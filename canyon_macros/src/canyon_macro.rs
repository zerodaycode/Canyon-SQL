/// Provides helpers to build the #[canyon] procedural macro like attribute

use proc_macro2::TokenStream;
use syn::Block;
use quote::quote;

use canyon_observer::CANYON_REGISTER;

/// Creates a TokenScream that is used to load the data generated at compile-time
/// by the `CanyonManaged` macros again on the Canyon register but
pub fn _wire_data_on_canyon_register(canyon_manager_tokens: &mut Vec<TokenStream>) {
    let mut identifiers = String::new();

    unsafe {
        for element in &CANYON_REGISTER {
            identifiers.push_str(element.as_str());
            identifiers.push(',');
        }
    }

    let tokens = quote! {
        use canyon_sql::canyon_observer::{
            CANYON_REGISTER, 
            CREDENTIALS, 
            credentials::DatabaseCredentials
        };
        
        unsafe { CREDENTIALS = Some(DatabaseCredentials::new()); }
        unsafe { println!("CREDENTIALS MACRO IN: {:?}", CREDENTIALS); }
        unsafe { CANYON_REGISTER = #identifiers
            .split(',')
            .map(str::to_string)
            .collect();
            // TODO Delete (or just pick without it) the last elemement
            // from the new assignation
            // CANYON_REGISTER.pop_back();
        }
        unsafe { println!("Register status IN: {:?}", CANYON_REGISTER) };
    };

    canyon_manager_tokens.push(tokens);
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