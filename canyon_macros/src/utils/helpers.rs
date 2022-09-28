// use std::collections::HashMap;

// use proc_macro::TokenStream as TokenStream1;
use proc_macro2::Ident;
// use syn::{NestedMeta, Lit};

// use quote::quote;

// // #[derive(Debug)]
// /// Utilery struct for wrapping the content and result of parsing the attributes on the `canyon` macro
// pub struct MacroAttributesParser<'a> {
//     pub attributes: HashMap<&'a str, &'a dyn ToString>,
//     pub error: Option<TokenStream1>
// }

// /// Parses the [`syn::NestedMeta::Meta`] or [`syn::NestedMeta::Lit`] attached to the `canyon` macro
// pub fn parse_macro_attributes<'a>(_meta: &'a Vec<NestedMeta>, macro_name: &'a str) -> MacroAttributesParser<'a> {
//     let mut res = MacroAttributesParser { 
//         attributes: HashMap::new(), 
//         error: None 
//     };

//     for nested_meta in _meta {
//         match nested_meta {
//             syn::NestedMeta::Meta(m) => determine_allowed_attributes(m, &mut res, None),
//             syn::NestedMeta::Lit(lit) => match lit {
//                 syn::Lit::Str(ref l) => {
//                     match macro_name {
//                         "canyon" => res.error = Some(report_literals_not_allowed(&l.value(), lit)),
//                         "canyon_entity" => res.attributes.get_mut("table_name")
//                     }
//                 },
//                 syn::Lit::ByteStr(ref l) => res.error = Some(report_literals_not_allowed(&String::from_utf8_lossy(&l.value()), &lit)),
//                 syn::Lit::Byte(ref l) => res.error = Some(report_literals_not_allowed(&l.value().to_string(), lit)),
//                 syn::Lit::Char(ref l) => res.error = Some(report_literals_not_allowed(&l.value().to_string(), lit)),
//                 syn::Lit::Int(ref l) => res.error = Some(report_literals_not_allowed(&l.to_string(), lit)),
//                 syn::Lit::Float(ref l) => res.error = Some(report_literals_not_allowed(&l.to_string(), lit)),
//                 syn::Lit::Bool(ref l) => res.error = Some(report_literals_not_allowed(&l.value().to_string(), lit)) ,
//                 syn::Lit::Verbatim(ref l) => res.error = Some(report_literals_not_allowed(&l.to_string(), lit))
//             }
//         }
//     };

//     res
// }


// /// Determines whenever a [`syn::NestedMeta::Meta`] it's classified as a valid argument of some macro attribute
// fn determine_allowed_attributes(meta: &syn::Meta, cma: &mut MacroAttributesParser, val: Option<Lit>) {
//     const ALLOWED_ATTRS: [&'static str; 1] = ["enable_migrations"];
    
//     let attr_ident = meta.path().get_ident().unwrap();
//     let attr_ident_str = attr_ident.to_string();
    
//     if attr_ident_str.as_str() == "enable_migrations" {
//         cma.attributes.insert("enable_migrations", &true);
//     } else if attr_ident_str.as_str() == "table_name" {
//         cma.attributes.insert("table_name", &true);
//     } else {
//         let error = syn::Error::new_spanned(
//             Ident::new(&attr_ident_str, attr_ident.span().into()), 
//         format!(
//                 "No `{attr_ident_str}` arguments allowed in the `Canyon` macro attributes.\n\
//                 Allowed ones are: {:?}", ALLOWED_ATTRS
//             )
//         ).into_compile_error();
//         cma.error = Some(
//            quote! {
//                 #error
//                 fn main() {}
//            }.into()
//         )
//     }
// }


// /// Creates a custom error for report not allowed literals on the attribute
// /// args of the `canyon` proc macro
// fn report_literals_not_allowed(ident: &str, s: &Lit) -> TokenStream1 {
//     let error = syn::Error::new_spanned(Ident::new(ident, s.span().into()), 
//         "No literals allowed in the `Canyon` macro"
//     ).into_compile_error();
    
//     quote! {
//         #error
//         fn main() {}
//    }.into()
// }


/// Parses a syn::Identifier to get a snake case database name from the type identifier
/// TODO: #[macro(table_name = 'user_defined_db_table_name)]' 
pub fn database_table_name_from_struct(ty: &Ident) -> String {

    let struct_name: String = String::from(ty.to_string());
    let mut table_name: String = String::new();
    
    let mut index = 0;
    for char in struct_name.chars() {
        if index < 1 {
            table_name.push(char.to_ascii_lowercase());
            index += 1;
        } else {
            match char {
                n if n.is_ascii_uppercase() => {
                    table_name.push('_');
                    table_name.push(n.to_ascii_lowercase()); 
                }
                _ => table_name.push(char)
            }
        }   
    }

    table_name
}

/// Parses a syn::Identifier to get a snake case database name from the type identifier
/// TODO: #[macro(table_name = 'user_defined_db_table_name)]' 
pub fn database_table_name_from_entity_name(ty: &str) -> String {

    let struct_name: String = String::from(ty.to_string());
    let mut table_name: String = String::new();
    
    let mut index = 0;
    for char in struct_name.chars() {
        if index < 1 {
            table_name.push(char.to_ascii_lowercase());
            index += 1;
        } else {
            match char {
                n if n.is_ascii_uppercase() => {
                    table_name.push('_');
                    table_name.push(n.to_ascii_lowercase()); 
                }
                _ => table_name.push(char)
            }
        }   
    }

    table_name
}

/// Parses the content of an &str to get the related identifier of a type
pub fn database_table_name_to_struct_ident(name: &str) -> Ident {
    let mut struct_name: String = String::new();
    
    let mut first_iteration = true;
    let mut previous_was_underscore = false;

    for char in name.chars() {
        if first_iteration {
            struct_name.push(char.to_ascii_uppercase());
            first_iteration = false;
        } else {
            match char {
                n if n == '_' => {
                    previous_was_underscore = true;
                },
                char if char.is_ascii_lowercase() => {
                    if previous_was_underscore {
                        struct_name.push(char.to_ascii_lowercase())
                    } else { struct_name.push(char) }
                },
                _ => panic!("Detected wrong format or broken convention for database table names")
            }
        }   
    }

    Ident::new(
        &struct_name,
        proc_macro2::Span::call_site()
    )
}
