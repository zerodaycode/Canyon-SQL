use proc_macro2::Ident;
use syn::{Visibility, Generics, DeriveInput, Fields, Type};

/// Provides a convenient way of store the data for the TokenStream 
/// received on a macro
pub struct MacroTokens<'a> {
    pub vis: &'a Visibility,
    pub ty: &'a Ident,
    pub generics: &'a Generics,
    pub fields: &'a Fields
}

impl<'a> MacroTokens<'a> {
    // Constructor
    pub fn new(ast: &'a DeriveInput) -> Self {
        Self {
            vis: &ast.vis,
            ty: &ast.ident,
            generics: &ast.generics,
            fields: match &ast.data {
                syn::Data::Struct(ref s) => &s.fields,
                _ => panic!("Field names can only be derived for structs"),
            }
        }
    }

    /// Gives a Vec ot tuples that contains the visibilty, the name and
    /// the type of every field on a Struct
    pub fn _fields_with_visibility_and_types(&self) -> Vec<(Visibility, Ident, Type)> {
        self.fields
            .iter()
            .map( |field| 
                (
                    field.vis.clone(), 
                    field.ident.as_ref().unwrap().clone(),
                    field.ty.clone()
                ) 
        )
        .collect::<Vec<_>>()
    }


    /// Gives a Vec ot tuples that contains the name and
    /// the type of every field on a Struct
    pub fn _fields_with_types(&self) -> Vec<(Ident, Type)> {
        self.fields
            .iter()
            .map( |field|  
                (
                    field.ident.as_ref().unwrap().clone(),
                    field.ty.clone()
                ) 
            )
        .collect::<Vec<_>>()
    }


    /// Gives a Vec with the fields of a Strut
    pub fn get_struct_fields(&self) -> Vec<Ident> {
        self.fields
            .iter()
            .map( |field|  
                field.ident.as_ref().unwrap().clone(),
            )
        .collect::<Vec<_>>()
    }

}


pub trait MacroDataParser {
    // TODO Implement the common methods that acts over the data
    // that it's typically needed when build a macro 
}