use std::cell::RefCell;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, Type};

thread_local! {
    pub static USER_MOCK_TY: RefCell<Ident> = RefCell::new(Ident::new("User", Span::call_site()));
    pub static VOID_RET_TY: RefCell<TokenStream> = RefCell::new({
        let ret_ty: Type = syn::parse_str("()").expect("Failed to parse unit type");
        quote! { #ret_ty }
    });
    pub static PK_MOCK_FIELD_VALUE: RefCell<TokenStream> = RefCell::new({
        quote! { 1 }
    });
}

pub const RAW_RET_TY: &str = "Vec < User >";
pub const RES_RET_TY: &str =
    "Result < Vec < User > , Box < (dyn std :: error :: Error + Send + Sync) > >";
pub const RES_VOID_RET_TY: &str =
    "Result < () , Box < (dyn std :: error :: Error + Send + Sync) > >";
pub const RES_RET_TY_LT: &str =
    "Result < Vec < User > , Box < (dyn std :: error :: Error + Send + Sync + 'a) > >";
pub const RES_VOID_RET_TY_LT: &str =
    "Result < () , Box < (dyn std :: error :: Error + Send + Sync + 'a) > >";
pub const OPT_RET_TY_LT: &str =
    "Result < Option < User > , Box < (dyn std :: error :: Error + Send + Sync + 'a) > >";

pub const MAPS_TO: &str = "into_results :: < User > ()";
pub const LT_CONSTRAINT: &str = "< 'a ";
pub const INPUT_PARAM: &str = "input : I";

pub const WITH_WHERE_BOUNDS: &str =
    "where I : canyon_sql :: core :: DbConnection + Send + 'a ";