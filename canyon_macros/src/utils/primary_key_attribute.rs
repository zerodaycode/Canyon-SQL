use proc_macro2::Ident;
use quote::ToTokens;
use std::fmt::{Display, Formatter};
use syn::{Field, Type};

/// Strong type for the index numerical value of the actual position on where a `#[primary_key]`
/// annotation is declared on a struct field
#[derive(Copy, Clone)]
pub(crate) struct PrimaryKeyIndex(pub(crate) usize);
impl From<PrimaryKeyIndex> for usize {
    fn from(pk: PrimaryKeyIndex) -> usize {
        pk.0
    }
}

pub(crate) struct PrimaryKeyAttribute<'a> {
    pub ident: &'a Ident,
    pub ty: &'a Type,
    pub name: String,
    pub index: PrimaryKeyIndex,
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
impl<'a> From<(PrimaryKeyIndex, &'a Field)> for PrimaryKeyAttribute<'a> {
    fn from(value: (PrimaryKeyIndex, &'a Field)) -> Self {
        let field = value.1;
        Self {
            ident: field.ident.as_ref().unwrap(),
            ty: &field.ty,
            name: field.ident.as_ref().unwrap().to_string(),
            index: value.0,
        }
    }
}
