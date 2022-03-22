/// Provides helpers to build the #[canyon] procedural like attribute macro

use proc_macro2::TokenStream;
use syn::Block;
use quote::quote;

use canyon_observer::CANYON_REGISTER;


/// Creates a TokenScream that is used to load the data generated at compile-time
/// by the `CanyonManaged` macros again on the Canyon register but
pub fn _wire_data_on_canyon_register(canyon_manager_tokens: &mut Vec<TokenStream>) {
    let mut entities_as_string = String::new();

    unsafe {
        // println!("a :{:?}",canyon_manager_tokens);
        for element in &CANYON_REGISTER {
            let pattern_entity = format!(
                "[{}]",element
            );

            entities_as_string.push_str(pattern_entity.as_str());
        }
    }

    let tokens = quote! {
        // TODO wire the credentials from the observer, not from CRUD
        // unsafe { CREDENTIALS = Some(DatabaseCredentials::new()); }
        // unsafe { println!("CREDENTIALS MACRO IN: {:?}", CREDENTIALS); }
        unsafe { CANYON_REGISTER = #entities_as_string
            .split(',')
            .map(str::to_string)
            .collect();
            // TODO Delete (or just pick without it) the last elemement
            // from the new assignation
            // CANYON_REGISTER.pop_back();
        }

        unsafe { println!("Register status IN: {:?}", CANYON_REGISTER) };
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

/// Handler to activate the Canyon Manager, and wire the tokens into the main fn()
pub fn call_canyon_manager(canyon_manager_tokens: &mut Vec<TokenStream>) {
    //*   HANDLER EVENTS */
    let canyon_manager_actions = quote! {
        
        CanyonHandler::new().await;

        CanyonHandler::<'_>::compare_tables_register_db().await;
    };

    canyon_manager_tokens.push(canyon_manager_actions);
}