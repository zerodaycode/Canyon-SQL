use crate::utils::helpers::filter_fields;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn foreignkeyable_impl_tokens(ast: DeriveInput) -> TokenStream {
    let ty = ast.ident;

    // Recovers the identifiers of the structs members
    let fields = filter_fields(match ast.data {
        syn::Data::Struct(ref s) => &s.fields,
        _ => {
            return syn::Error::new(ty.span(), "ForeignKeyable only works with Structs")
                .to_compile_error();
        }
    });

    let field_idents = fields.iter().map(|(_vis, ident)| {
        let i = ident.to_string();
        quote! {
            #i => Some(&self.#ident as &dyn canyon_sql::query::QueryParameter)
        }
    });
    let field_idents_cloned = field_idents.clone();

    quote! {
        /// Implementation of the trait `ForeignKeyable` for the type
        /// calling this derive proc macro
        impl canyon_sql::query::bounds::ForeignKeyable<Self> for #ty {
            fn get_fk_column(&self, column: &str) -> Option<&dyn canyon_sql::query::QueryParameter> {
                match column {
                    #(#field_idents),*,
                    _ => None
                }
            }
        }
        /// Implementation of the trait `ForeignKeyable` for a reference of this type
        /// calling this derive proc macro
        impl canyon_sql::query::bounds::ForeignKeyable<&Self> for &#ty {
            fn get_fk_column<'a>(&self, column: &'a str) -> Option<&dyn canyon_sql::query::QueryParameter> {
                match column {
                    #(#field_idents_cloned),*,
                    _ => None
                }
            }
        }
    }
}
