use proc_macro2::Ident;
use syn::{
    Visibility, 
    Generics, 
    DeriveInput, 
    Fields, 
    Type, 
    Attribute
};

/// Provides a convenient way of store the data for the TokenStream 
/// received on a macro
pub struct MacroTokens<'a> {
    pub vis: &'a Visibility,
    pub ty: &'a Ident,
    pub generics: &'a Generics,
    pub attrs: &'a Vec<Attribute>,
    pub fields: &'a Fields
}

impl<'a> MacroTokens<'a> {
    pub fn new(ast: &'a DeriveInput) -> Self {
        Self {
            vis: &ast.vis,
            ty: &ast.ident,
            generics: &ast.generics,
            attrs: &ast.attrs,
            fields: match &ast.data {
                syn::Data::Struct(ref s) => &s.fields,
                _ => panic!("This derive macro can only be automatically derived for structs"),
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


    /// Gives a Vec of Ident with the fields of a Struct
    pub fn get_struct_fields(&self) -> Vec<Ident> {
        self.fields
            .iter()
            .map( |field|  
                field.ident.as_ref().unwrap().clone(),
            )
        .collect::<Vec<_>>()
    }

    /// Gives a Vec populated with the name of the fields of the struct 
    pub fn _get_struct_fields_as_collection_strings(&self) -> Vec<String> {
        self.get_struct_fields()
            .iter()
            .map( |ident| {
                ident.to_owned().to_string()
            }
        ).collect::<Vec<String>>()
    }

    /// Returns a Vec populated with the name of the fields of the struct
    /// already quote scaped for avoid the upper case column name mangling.
    /// 
    /// If the type contains a `#[primary_key]` annotation (and), returns the
    /// name of the columns without the fields that maps against the column designed as
    /// primary key (if its present and its autoincremental attribute is setted to true)
    /// (autoincremental = true) or its without the autoincremental attribute, which leads
    /// to the same behaviour.
    /// 
    /// Returns every field if there's no PK, or if it's present but autoincremental = false 
    pub fn get_column_names_pk_parsed(&self) -> Vec<String> {
        self.fields
            .iter()
            .filter( |field| {
                    if field.attrs.len() > 0 {
                        field.attrs.iter().any( |attr| 
                            {   
                                let a = attr.path.segments[0].clone().ident;
                                let b = attr.tokens.to_string();
                                if a.to_string() == "primary_key" || b.to_string().contains("false") {
                                    false
                                } else { true }
                            }
                        )
                    } else { true }
                }
            ).map( |c| 
                format!( "\"{}\"", c.ident.as_ref().unwrap().to_string() )
            ).collect::<Vec<String>>()
    }

    /// Retrieves the fields of the Struct as continuous String, comma separated
    pub fn get_struct_fields_as_strings(&self) -> String {
        let column_names: String = self.get_struct_fields()
            .iter()
            .map( |ident| {
                ident.to_owned().to_string()
            }).collect::<Vec<String>>()
                .iter()
                .map( |column| column.to_owned() + ", ")
            .collect::<String>();
        
        let mut column_names_as_chars = column_names.chars();
        column_names_as_chars.next_back();
        column_names_as_chars.next_back();
        
        column_names_as_chars.as_str().to_owned()
    }

    /// 
    pub fn get_pk_index(&self) -> Option<usize> {
        let mut pk_index = None;
        for (idx, field) in self.fields.iter().enumerate() {
            for attr in &field.attrs {
                if attr.path.segments[0].clone().ident.to_string() == "primary_key" {
                    pk_index = Some(idx);
                }
            }
        }
        pk_index
    }

    /// Utility for find the primary key attribute (if exists) and the
    /// column name (field) which belongs
    pub fn get_primary_key_annotation(&self) -> Option<String> {
        let f = self.fields
            .iter()
            .find( |field| 
                field.attrs.iter()
                    .map( |attr| 
                            attr.path.segments[0].clone().ident
                    ).map( |ident| 
                        ident.to_string()
                    ).find( |a| 
                            a == "primary_key"
                    ) == Some("primary_key".to_string())
            );

        f.map( |v| v.ident.clone().unwrap().to_string())
    }

    /// Boolean that returns true if the type contains a `#[primary_key]`
    /// annotation. False otherwise.
    pub fn type_has_primary_key(&self) -> bool {
        self.fields.iter()
            .any( |field| 
                field.attrs.iter()
                    .map( |attr| 
                        attr.path.segments[0].clone().ident
                    ).map( |ident| 
                        ident.to_string()
                    ).find ( |a| 
                        a == "primary_key"
                    ) == Some("primary_key".to_string())
            )
    }

    /// Returns an String ready to be inserted on the VALUES Sql clause
    /// representing generic query parameters ($x).
    /// 
    /// Already returns the correct number of placeholders, skipping one
    /// entry in the type contains a `#[primary_key]`
    pub fn placeholders_generator(&self) -> String {
        let mut placeholders = String::new();
        if self.type_has_primary_key() {
            for num in 1..self.fields.len() {
                if num < self.fields.len() - 1 {
                    placeholders.push_str(&("$".to_owned() + &(num).to_string() + ", "));
                } else {
                    placeholders.push_str(&("$".to_owned() + &(num).to_string()));
                }
            }
        } else {
            for num in 1..self.fields.len() + 1 {
                if num < self.fields.len() {
                    placeholders.push_str(&("$".to_owned() + &(num).to_string() + ", "));
                } else {
                    placeholders.push_str(&("$".to_owned() + &(num).to_string()));
                }
            }
        }

        placeholders
    }
}