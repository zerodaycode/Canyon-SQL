use proc_macro2::{Ident, TokenStream, Span};
use syn::{
    Visibility, 
    Generics, 
    DeriveInput, 
    Fields, 
    Type, 
    Attribute, 
    Token, 
    punctuated::Punctuated, 
    MetaNameValue
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

    /// If the `canyon_entity` macro has valid attributes attached, and those attrs are the
    /// user's desired `table_name` and/or the `schema_name`, this method returns its 
    /// correct form to be wired as the table name that the CRUD methods requires for generate
    /// the queries
    pub fn get_desired_table_name(&self) -> Result<String, TokenStream> {
        let mut table_name: Option<String> = None;
        let mut schema: Option<String> = None;

        for attr in self.attrs {
            let name_values: Result<Punctuated<MetaNameValue, Token![,]>, syn::Error> = 
                attr.parse_args_with(Punctuated::parse_terminated);
            
            if let Err(_) = name_values {
                return Ok(self.ty.to_string());
            }

            for nv in name_values.ok().expect("Failure parsing canyon_entity macro attributes") {
                let ident = nv.path.get_ident();
                if let Some(i) = ident {
                    let identifier = i.to_string();
                    match &nv.lit {
                        syn::Lit::Str(s) => {
                            if identifier == "table_name" {
                                table_name = Some(s.value())
                            } else if identifier == "schema" {
                                schema = Some(s.value())
                            } else {
                                return Err(
                                    syn::Error::new_spanned(
                                        Ident::new(&identifier, i.span().into()), 
                                        "Only string literals are valid values for the attribute arguments"
                                        ).into_compile_error()
                                );
                            }
                        },
                        _ => return Err(
                            syn::Error::new_spanned(
                                Ident::new(&identifier, i.span().into()), 
                                "Only string literals are valid values for the attribute arguments"
                                ).into_compile_error()
                        ),
                    }
                } else {
                    return Err(
                        syn::Error::new(
                            Span::call_site(), 
                            "Only string literals are valid values for the attribute arguments"
                        ).into_compile_error()
                    );
                }
            }
        }

        let mut final_table_name = String::new();
            if schema.is_some() { 
                final_table_name.push_str(
                    format!("{}.", schema.unwrap()).as_str()
                ) 
            }
            if table_name.is_some() {
                final_table_name.push_str(table_name.unwrap().as_str())
            } else {
                final_table_name.push_str(self.ty.to_string().as_str())
            }
            
    
        Ok(final_table_name)
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
                    ).find ( |field_name| 
                        field_name == "primary_key"
                    ) == Some("primary_key".to_string())
            );

        f.map( |v| v.ident.clone().unwrap().to_string())
    }
}