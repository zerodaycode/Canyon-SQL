use proc_macro2::TokenStream;
use quote::quote;
use proc_macro2::Ident;
use quote::__private::Span;
use syn::{Field, Type};

pub(crate) struct PrimaryKeyAttribute<'a> {
    pub ident: &'a Ident,
    pub ty: &'a Type,
    pub name: String,
}

// impl<'a> Default for &'a PrimaryKeyAttribute<'a> {
//     fn default() -> Self {
//         Self {
//             ident: &Ident::new_raw("Non existent", Span::call_site()),
//             ty: &Type::Verbatim(quote!{}.into()),
//             name: "".to_string(),
//         }
//     }
// }

impl<'a> PrimaryKeyAttribute<'a> {
    pub(crate) fn get_ident_as_token_stream(&self) -> TokenStream {
        let ident = self.ident;
        quote! { #ident }
    }
    pub(crate) fn get_type_as_token_stream(&self) -> TokenStream {
        let ty = self.ty;
        quote! { #ty }
    }
}
impl<'a> From<&'a Field> for PrimaryKeyAttribute<'a> {
    fn from(value: &'a Field) -> Self {
        Self { ident: value.ident.as_ref().unwrap(), ty: &value.ty, name: value.ident.as_ref().unwrap().to_string() }
    }
}