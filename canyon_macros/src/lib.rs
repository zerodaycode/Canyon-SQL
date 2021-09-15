extern crate proc_macro;
// extern crate quote;

// #[macro_export] macro_rules! quote
use proc_macro2::Ident;

use quote::quote;
use syn::{
    DeriveInput, Fields, Visibility
};

/// Allows the implementors to auto-derive de `crud-operations` trait, which defines the methods
/// that will perform the database communication and that will query against the db.
#[proc_macro_derive(CanyonCRUD)]
pub fn crud_operations(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_crud_operations_trait_for_struct(&ast)
}

fn impl_crud_operations_trait_for_struct(ast: &syn::DeriveInput) -> proc_macro::TokenStream {
    let ty = &ast.ident;
    let tokens = quote! {
        #[async_trait]
        impl canyon_sql::crud::CrudOperations<#ty> for #ty { }
    };
    tokens.into()
}


#[proc_macro_derive(CanyonMapper)]
pub fn implement_row_mapper_for_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let (vis, ty, generics) = (&ast.vis, &ast.ident, &ast.generics);

    let fields = filter_fields(
        match ast.data {
            syn::Data::Struct(ref s) => &s.fields,
            _ => panic!("Field names can only be derived for structs"),
        }
    );

    let names_const_fields_str = fields.iter().map(|(_vis, ident)| {
        let ident_name = ident.to_string();
        quote! {  
            #ident_name
        }
    });
    
    let field_names_for_row_mapper = fields.iter().map(|(_vis, ident)| {
        let ident_name = ident.to_string();
        quote! {  
            #ident: row.try_get(#ident_name)
                .expect(format!("Failed to retrieve the {} field", #ident_name).as_ref())
        }
    });

    let (impl_generics, ty_generics, where_clause) = 
        generics.split_for_impl();

    let tokens = quote! {
        use canyon_sql::{self, crud::CrudOperations, mapper::RowMapper};
        use canyon_sql::tokio_postgres::Row;

        impl #impl_generics #ty #ty_generics
            #where_clause
        {

            #vis fn get_field_names() -> Vec<String> {
                let mut vec = Vec::new();

                let field_names = stringify!( 
                        #(#names_const_fields_str),*
                    ).split(",")
                    .collect::<Vec<_>>()
                    .into_iter()
                    .for_each( |field_name| 
                        vec.push(
                            field_name
                            .replace('"', "")
                            .replace(' ', "")
                            .to_string()
                        )
                    );
                vec
            }

        }

        impl RowMapper<Self> for #ty {
            fn deserialize(row: &Row) -> Self {
                Self {
                    #(#field_names_for_row_mapper),*
                }
            }
        }
    };

    tokens.into()
}

fn filter_fields(fields: &Fields) -> Vec<(Visibility, Ident)> {
    fields
        .iter()
        .map(|field| 
            (field.vis.clone(), field.ident.as_ref().unwrap().clone()) 
        )
        .collect::<Vec<_>>()
}