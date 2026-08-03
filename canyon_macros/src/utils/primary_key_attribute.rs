use proc_macro2::Ident;
use quote::ToTokens;
use std::fmt::{Display, Formatter};
use syn::{Field, Type};

pub(crate) struct PrimaryKeyAttribute<'a> {
    pub ident: &'a Ident,
    pub ty: &'a Type,
    pub name: String,
}

impl<'a> Display for &'a PrimaryKeyAttribute<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let _ = f.write_fmt(format_args!(
            "ident:{},ty:{},name:{}",
            self.ident,
            self.ty.to_token_stream(),
            self.name
        ));
        Ok(())
    }
}

/// Ad-hoc creation for the process of parsing a primary key attribute along with its index position
/// on the struct
impl<'a> From<&'a Field> for PrimaryKeyAttribute<'a> {
    fn from(field: &'a Field) -> Self {
        Self {
            ident: field.ident.as_ref().unwrap(),
            ty: &field.ty,
            name: field.ident.as_ref().unwrap().to_string(),
        }
    }
}
