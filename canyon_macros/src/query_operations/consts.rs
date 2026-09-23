#![allow(dead_code)]

use std::cell::RefCell;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, Type};

pub const UNAVAILABLE_CRUD_OP_ON_INSTANCE: &str = "Operation is unavailable. T doesn't contain a #[primary_key]\
    annotation. You must construct the query with the QueryBuilder type\
    (<op_type>_query method for the Crud implementors";

pub(crate) fn generate_no_pk_error() -> TokenStream {
    quote! {
        return Err(canyon_sql::error::QueryBuilderError::MissingPrimaryKey.into());
    }
}

pub(crate) fn generate_default_db_conn_tokens() -> TokenStream {
    quote! {
        let default_db_conn = canyon_sql::core::Canyon::instance()?
            .get_default_connection()?;
        default_db_conn
    }
}

pub(crate) fn generate_default_db_conn_and_type_tokens() -> TokenStream {
    quote! {
        let default_db_conn = canyon_sql::core::Canyon::instance()?
            .get_default_connection()?;
        let db_type = default_db_conn.get_database_type()?;
    }
}

thread_local! {
    pub static USER_MOCK_TY: RefCell<Ident> = RefCell::new(Ident::new("User", Span::call_site()));
    pub static USER_MOCK_MAPPER_TY: RefCell<Ident> = RefCell::new(Ident::new("User", Span::call_site()));
    pub static VOID_RET_TY: RefCell<TokenStream> = RefCell::new({
        let ret_ty: Type = syn::parse_str("()").expect("Failed to parse unit type");
        quote! { #ret_ty }
    });
    pub static PK_MOCK_FIELD_VALUE: RefCell<TokenStream> = RefCell::new({
        quote! { 1 }
    });
}

pub const RAW_RET_TY: &str = "Vec < User >";
pub const RES_RET_TY: &str = "canyon_sql :: CanyonResult < Vec < User > >";
pub const RES_VOID_RET_TY: &str = "canyon_sql :: CanyonResult < () >";
pub const RES_RET_TY_LT: &str = "canyon_sql :: CanyonResult < Vec < User > >";
pub const RES_VOID_RET_TY_LT: &str = "canyon_sql :: CanyonResult < () >";
pub const OPT_RET_TY_LT: &str = "canyon_sql :: CanyonResult < Option < User > >";
pub const I64_RET_TY: &str = "canyon_sql :: CanyonResult < i64 >";
pub const I64_RET_TY_LT: &str = "canyon_sql :: CanyonResult < i64 >";

pub const MAPS_TO: &str = "into_results :: < User > ()";
pub const LT_CONSTRAINT: &str = "< 'a ";
pub const INPUT_PARAM: &str = "input : I";
pub const VALUE_PARAM: &str = "& 'a dyn canyon_sql :: core :: QueryParameter < 'a >";

pub const WITH_WHERE_BOUNDS: &str = "where I : canyon_sql :: core :: DbConnection + Send + 'a ";

pub const FIND_BY_PK_ERR_NO_PK: &str = "You can't use the 'find_by_pk' associated function on a \
                        CanyonEntity that does not have a #[primary_key] annotation. \
                        If you need to perform an specific search, use the Querybuilder instead.";
