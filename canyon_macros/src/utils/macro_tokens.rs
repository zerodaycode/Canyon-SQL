use std::convert::TryFrom;
use std::fmt::Write;

use crate::utils::canyon_crud_attribute::CanyonCrudAttribute;
use canyon_entities::field_annotation::EntityFieldAnnotation;
use proc_macro2::{Ident, Span};
use syn::{Attribute, DeriveInput, Field, Fields, Generics, Type, Visibility};
use crate::utils::helpers;

/// Provides a convenient way of store the data for the TokenStream
/// received on a macro
#[allow(dead_code)]
pub struct MacroTokens<'a> {
    pub vis: &'a Visibility,
    pub ty: &'a Ident,
    pub generics: &'a Generics,
    pub attrs: &'a Vec<Attribute>,
    pub fields: &'a Fields,
    // -------- the new fields that must help to avoid recalculations every time that the user compiles
    pub(crate) canyon_crud_attribute: Option<CanyonCrudAttribute>,
}

impl<'a> MacroTokens<'a> {
    pub fn new(ast: &'a DeriveInput) -> Result<Self, syn::Error> {
        // TODO: impl syn::parse instead
        if let syn::Data::Struct(ref s) = ast.data {
            let attrs = &ast.attrs;
            let mut canyon_crud_attribute = None;
            for attr in attrs {
                if attr.path.is_ident("canyon_crud") {
                    canyon_crud_attribute = Some(attr.parse_args::<CanyonCrudAttribute>()?);
                }
            }

            Ok(Self {
                vis: &ast.vis,
                ty: &ast.ident,
                generics: &ast.generics,
                attrs: &ast.attrs,
                fields: &s.fields,
                canyon_crud_attribute,
            })
        } else {
            Err(syn::Error::new(
                Span::call_site(),
                "CanyonCrud may only be implemented for structs",
            ))
        }
    }

    pub fn retrieve_mapping_target_type(&self) -> &Option<Ident> {
        if let Some(canyon_crud_attribute) = &self.canyon_crud_attribute {
            &canyon_crud_attribute.maps_to
        } else {
            &None
        }
    }

    pub fn fields(&self) -> Vec<(Visibility, Ident, Type)> {
        self.fields
            .iter()
            .map(|field| {
                (
                    field.vis.clone(),
                    field.ident.clone().unwrap(),
                    field.clone().ty,
                )
            })
            .collect::<Vec<_>>()
    }

    /// Gives a Vec of tuples that contains the name and
    /// the type of every field on a Struct
    pub fn fields_with_types(&self) -> Vec<(&Ident, &Type)> {
        self.fields
            .iter()
            .map(|field| (field.ident.as_ref().unwrap(), &field.ty))
            .collect::<Vec<_>>()
    }

    /// Gives a Vec of Ident with the fields of a Struct
    pub fn get_struct_fields(&self) -> Vec<Ident> {
        self.fields
            .iter()
            .map(|field| field.ident.as_ref().unwrap().clone())
            .collect::<Vec<_>>()
    }

    /// Returns a Vec populated with the fields of the struct
    ///
    /// If the type contains a `#[primary_key]` annotation (and), returns the
    /// name of the columns without the fields that maps against the column designed as
    /// primary key (if its present and its autoincremental attribute is set to true)
    /// (autoincremental = true) or its without the autoincremental attribute, which leads
    /// to the same behaviour.
    ///
    /// Returns every field if there's no PK, or if it's present but autoincremental = false
    pub fn get_columns_pk_parsed(&self) -> Vec<&Field> {
        self.fields
            .iter()
            .filter(|field| {
                if !field.attrs.is_empty() {
                    field.attrs.iter().any(|attr| {
                        let a = attr.path.segments[0].clone().ident;
                        let b = attr.tokens.to_string();
                        !(a == "primary_key" || b.contains("false"))
                    })
                } else {
                    true
                }
            })
            .collect::<Vec<_>>()
    }

    /// Returns a collection with all the [`syn::Ident`] for all the type members, skipping (if present)
    /// the field which is annotated with #[primary_key]
    pub fn get_fields_idents_pk_parsed(&self) -> Vec<&Ident> {
        self.get_columns_pk_parsed()
            .iter()
            .map(|field| field.ident.as_ref().unwrap())
            .collect::<Vec<_>>()
    }

    /// Returns a Vec populated with the name of the fields of the struct
    /// already quote scaped for avoid the upper case column name mangling.
    ///
    /// If the type contains a `#[primary_key]` annotation (and), returns the
    /// name of the columns without the fields that maps against the column designed as
    /// primary key (if its present and its autoincremental attribute is set to true)
    /// (autoincremental = true) or its without the autoincremental attribute, which leads
    /// to the same behaviour.
    ///
    /// Returns every field if there's no PK, or if it's present but autoincremental = false
    pub fn get_column_names_pk_parsed(&self) -> Vec<String> {
        self.get_columns_pk_parsed()
            .iter()
            .map(|c| format!("\"{}\"", c.ident.as_ref().unwrap()))
            .collect::<Vec<String>>()
    }

    /// Retrieves the fields of the Struct as continuous String, comma separated
    pub fn _get_struct_fields_as_strings(&self) -> String {
        let column_names: String = self
            .get_struct_fields()
            .iter()
            .map(|ident| ident.to_owned().to_string())
            .map(|column| column.to_owned() + ", ")
            .collect();

        let mut column_names_as_chars = column_names.chars();
        column_names_as_chars.next_back();
        column_names_as_chars.next_back();

        column_names_as_chars.as_str().to_owned()
    }

    /// Retrieves the value of the index of an annotated field with #[primary_key]
    pub fn _get_pk_index(&self) -> Option<usize> {
        let mut pk_index = None;
        for (idx, field) in self.fields.iter().enumerate() {
            for attr in &field.attrs {
                if attr.path.segments.first()      
                    .map(|segment| segment.ident == "primary_key")?
                {
                    pk_index = Some(idx);
                }
            }
        }
        pk_index
    }

    /// Utility for find the primary key attribute (if exists) and the
    /// column name (field) which belongs
    pub fn get_primary_key_annotation(&self) -> Option<String> {
        let f = self.fields.iter().find(|field| {
            helpers::field_has_target_attribute(field, "primary_key")
        });

        f.map(|v| v.ident.clone().unwrap().to_string())
    }

    /// Utility for find the `foreign_key` attributes (if exists)
    pub fn get_fk_annotations(&self) -> Vec<(&Ident, EntityFieldAnnotation)> {
        let mut foreign_key_annotations = Vec::new();

        self.fields.iter().for_each(|field| {
            let attrs = field
                .attrs
                .iter()
                .filter(|attr| attr.path.segments[0].clone().ident == "foreign_key");
            attrs.for_each(|attr| {
                let fk_parse = EntityFieldAnnotation::try_from(&attr);
                if let Ok(fk_annotation) = fk_parse {
                    foreign_key_annotations.push((field.ident.as_ref().unwrap(), fk_annotation))
                }
            });
        });

        foreign_key_annotations
    }

    /// Boolean that returns true if the type contains a `#[primary_key]`
    /// annotation. False otherwise.
    pub fn type_has_primary_key(&self) -> bool {
        self.fields.iter().any(|field| {
            helpers::field_has_target_attribute(field, "primary_key")
        })
    }

    /// Returns a String ready to be inserted on the VALUES Sql clause
    /// representing generic query parameters ($x).
    ///
    /// Already returns the correct number of placeholders, skipping one
    /// entry in the type contains a `#[primary_key]`
    pub fn placeholders_generator(&self) -> String {
        let range_upper_bound = if self.type_has_primary_key() {
            self.fields.len()
        } else {
            self.fields.len() + 1
        };

        helpers::placeholders_generator(range_upper_bound)
    }
}
