//! Provides helpers to build the #[canyon] procedural like attribute macro

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Ident, TokenStream};

use quote::quote;

use canyon_observer::QUERIES_TO_EXECUTE;
use syn::{Lit, NestedMeta};

#[derive(Debug)]
/// Utilery struct for wrapping the content and result of parsing the attributes on the `canyon` macro
pub struct CanyonMacroAttributes {
    pub allowed_migrations: bool,
    pub error: Option<TokenStream1>
}

/// Parses the [`syn::NestedMeta::Meta`] or [`syn::NestedMeta::Lit`] attached to the `canyon` macro
pub fn parse_canyon_macro_attributes(_meta: &Vec<NestedMeta>) -> CanyonMacroAttributes {
    let mut res = CanyonMacroAttributes { 
        allowed_migrations: false, 
        error: None 
    };

    for nested_meta in _meta {
        match nested_meta {
            syn::NestedMeta::Meta(m) => determine_allowed_attributes(m, &mut res),
            syn::NestedMeta::Lit(lit) => match lit {
                syn::Lit::Str(ref l) => res.error = Some(report_literals_not_allowed(&l.value(), &lit)),
                syn::Lit::ByteStr(ref l) => res.error = Some(report_literals_not_allowed(&String::from_utf8_lossy(&l.value()), &lit)),
                syn::Lit::Byte(ref l) => res.error = Some(report_literals_not_allowed(&l.value().to_string(), &lit)),
                syn::Lit::Char(ref l) => res.error = Some(report_literals_not_allowed(&l.value().to_string(), &lit)),
                syn::Lit::Int(ref l) => res.error = Some(report_literals_not_allowed(&l.to_string(), &lit)),
                syn::Lit::Float(ref l) => res.error = Some(report_literals_not_allowed(&l.to_string(), &lit)),
                syn::Lit::Bool(ref l) => res.error = Some(report_literals_not_allowed(&l.value().to_string(), &lit)) ,
                syn::Lit::Verbatim(ref l) => res.error = Some(report_literals_not_allowed(&l.to_string(), &lit))
            }
        }
    };

    res
}


/// Determines whenever a [`syn::NestedMeta::Meta`] it's classified as a valid argument of the `canyon` macro
fn determine_allowed_attributes(meta: &syn::Meta, cma: &mut CanyonMacroAttributes) {
    const ALLOWED_ATTRS: [&'static str; 1] = ["enable_migrations"];
    
    let attr_ident = meta.path().get_ident().unwrap();
    let attr_ident_str = attr_ident.to_string();
    
    if attr_ident_str.as_str() == "enable_migrations" {
        cma.allowed_migrations = true;
    } else {
        let error = syn::Error::new_spanned(
            Ident::new(&attr_ident_str, attr_ident.span().into()), 
        format!(
                "No `{attr_ident_str}` arguments allowed in the `Canyon` macro attributes.\n\
                Allowed ones are: {:?}", ALLOWED_ATTRS
            )
        ).into_compile_error();
        cma.error = Some(
           quote! {
                #error
                fn main() {}
           }.into()
        )
    }
}

/// Creates a custom error for report not allowed literals on the attribute
/// args of the `canyon` proc macro
fn report_literals_not_allowed(ident: &str, s: &Lit) -> TokenStream1 {
    let error = syn::Error::new_spanned(Ident::new(ident, s.span().into()), 
        "No literals allowed in the `Canyon` macro"
    ).into_compile_error();
    
    quote! {
        #error
        fn main() {}
   }.into()
}


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
