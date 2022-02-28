extern crate proc_macro;

mod managed;

use proc_macro::TokenStream as CompilerTokenStream;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{
    DeriveInput, Fields, Visibility
};

#[proc_macro_attribute]
pub fn canyon_manager(meta: CompilerTokenStream, input: CompilerTokenStream) -> CompilerTokenStream {
    input.into()
}

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
        impl canyon_sql::crud::Transaction<#ty> for #ty { }
    };
    tokens.into()
}


#[proc_macro_derive(CanyonMapper)]
pub fn implement_row_mapper_for_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let (vis, ty, generics) = (&ast.vis, &ast.ident, &ast.generics);
    let table_name: String = database_table_name_from_struct(ty);

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
        use canyon_sql::{
            self, crud::CrudOperations, mapper::RowMapper,
            async_trait::*,
        };
        use canyon_sql::tokio_postgres::Row;

        impl #impl_generics #ty #ty_generics
            #where_clause
        {
            // Find all
            #vis async fn find_all() -> Vec<#ty> {
                #ty::__find_all(#table_name, &[])
                    .await
                    .as_response::<#ty>()
            }

            // Find by ID
            #vis async fn find_by_id(id: i32) -> #ty {
                #ty::__find_by_id(#table_name, id)
                    .await
                    .as_response::<#ty>()[0].clone()
            }

            fn get_field_names() -> Vec<String> {
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


/// Parses a syn::Identifier to get a snake case database name from the type identifier
fn database_table_name_from_struct(ty: &Ident) -> String {

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