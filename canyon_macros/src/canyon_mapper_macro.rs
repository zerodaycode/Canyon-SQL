use std::iter::Map;
use std::slice::Iter;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{DeriveInput, Type, Visibility};

use crate::utils::helpers::{fields_with_types};

pub fn canyon_mapper_impl_tokens(ast: DeriveInput) -> TokenStream {
    let ty = &ast.ident;
    let mut impl_methods = TokenStream::new();
    
    // Recovers the identifiers of the structs members
    let fields = fields_with_types(match ast.data {
        syn::Data::Struct(ref s) => &s.fields,
        _ => {
            return syn::Error::new(ast.ident.span(), "CanyonMapper only works with Structs")
                .to_compile_error()
                .into()
        }
    });

    #[cfg(feature = "postgres")]
    let pg_implementation = create_postgres_fields_mapping(&fields);
    #[cfg(feature = "postgres")]
    impl_methods.extend(quote! {
        fn deserialize_postgresql(row: &canyon_sql::db_clients::tokio_postgres::Row) -> #ty {
            Self {
                #(#pg_implementation),*
            }
        }
    });
    
    #[cfg(feature = "mssql")]
    let sqlserver_implementation = create_sqlserver_fields_mapping(&fields);
    #[cfg(feature = "mssql")]
    impl_methods.extend(quote! {
        fn deserialize_sqlserver(row: &canyon_sql::db_clients::tiberius::Row) -> #ty {
            Self {
                #(#sqlserver_implementation),*
            }
        }
    });
    
    #[cfg(feature = "mysql")]
    let mysql_implementation = create_mysql_fields_mapping(&fields);
    #[cfg(feature = "mysql")]
    impl_methods.extend(quote! {
        fn deserialize_mysql(row: &canyon_sql::db_clients::mysql_async::Row) -> #ty {
            Self {
                #(#mysql_implementation),*
            }
        }
    });

    quote! {
        impl canyon_sql::core::RowMapper<Self> for #ty {
            #impl_methods
        }
    }
}

#[cfg(feature = "postgres")]
fn create_postgres_fields_mapping(fields: &Vec<(Visibility, Ident, Type)>) -> Map<Iter<'_, (Visibility, Ident, Type)>, fn(&'_ (Visibility, Ident, Type)) -> TokenStream> {
    fields.iter().map(|(_vis, ident, _ty)| {
        let ident_name = ident.to_string();
        quote! {
            #ident: row.try_get(#ident_name) // TODO: can we wrap RowMapper in a Result and propagate errors with ?\?
                .expect(format!("Failed to retrieve the {} field", #ident_name).as_ref())
        }
    })
}

#[cfg(feature = "mysql")]
fn create_mysql_fields_mapping(fields: &Vec<(Visibility, Ident, Type)>) -> Map<Iter<'_, (Visibility, Ident, Type)>, fn(&'_ (Visibility, Ident, Type)) -> TokenStream> {
    fields.iter().map(|(_vis, ident, _ty)| {
        let ident_name = ident.to_string();
        quote! {
            #ident: row.get(#ident_name)
                .expect(format!("Failed to retrieve the {} field", #ident_name).as_ref())
        }
    })
}

#[cfg(feature = "mssql")]
fn create_sqlserver_fields_mapping(fields: &Vec<(Visibility, Ident, Type)>) -> Map<Iter<'_, (Visibility, Ident, Type)>, fn(&'_ (Visibility, Ident, Type)) -> TokenStream> {
    fields.iter().map(|(_vis, ident, ty)| {
        let ident_name = ident.to_string();

        if get_field_type_as_string(ty) == "String" {
            quote! {
                #ident: row.get::<&str, &str>(#ident_name)
                    .expect(format!("Failed to retrieve the `{}` field", #ident_name).as_ref())
                    .to_string()
            }
        } else if get_field_type_as_string(ty).replace(' ', "") == "Option < i64 >" {
            quote! {
                #ident: row.get::<i64, &str>(#ident_name)
            }
        } else if get_field_type_as_string(ty).replace(' ', "") == "Option<i32>" {
            quote! {
                #ident: row.get::<i32, &str>(#ident_name)
            }
        } else if get_field_type_as_string(ty).replace(' ', "") == "Option<i16>" {
            quote! {
                #ident: row.get::<i16, &str>(#ident_name)
            }
        } else if get_field_type_as_string(ty).replace(' ', "") == "Option<f32>" {
            quote! {
                #ident: row.get::<f32, &str>(#ident_name)
            }
        } else if get_field_type_as_string(ty).replace(' ', "") == "Option<f64>" {
            quote! {
                #ident: row.get::<f64, &str>(#ident_name)
            }
        } else if get_field_type_as_string(ty).replace(' ', "") == "Option<String>" {
            quote! {
                #ident: row.get::<&str, &str>(#ident_name)
                    .map( |x| x.to_owned() )
            }
        } else if get_field_type_as_string(ty) == "NaiveDate" {
            quote! {
                #ident: row.get::<canyon_sql::date_time::NaiveDate, &str>(#ident_name)
                    .expect(format!("Failed to retrieve the `{}` field", #ident_name).as_ref())
            }
        } else if get_field_type_as_string(ty).replace(' ', "") == "Option<NaiveDate>" {
            quote! {
                #ident: row.get::<canyon_sql::date_time::NaiveDate, &str>(#ident_name)
            }
        } else if get_field_type_as_string(ty) == "NaiveTime" {
            quote! {
                #ident: row.get::<canyon_sql::date_time::NaiveTime, &str>(#ident_name)
                    .expect(format!("Failed to retrieve the `{}` field", #ident_name).as_ref())
            }
        } else if get_field_type_as_string(ty).replace(' ', "") == "Option<NaiveTime>" {
            quote! {
                #ident: row.get::<canyon_sql::date_time::NaiveTime, &str>(#ident_name)
            }
        } else if get_field_type_as_string(ty) == "NaiveDateTime" {
            quote! {
                #ident: row.get::<canyon_sql::date_time::NaiveDateTime, &str>(#ident_name)
                    .expect(format!("Failed to retrieve the `{}` field", #ident_name).as_ref())
            }
        } else if get_field_type_as_string(ty).replace(' ', "") == "Option<NaiveDateTime>" {
            quote! {
                #ident: row.get::<canyon_sql::date_time::NaiveDateTime, &str>(#ident_name)
            }
        } else if get_field_type_as_string(ty) == "DateTime" {
            quote! {
                #ident: row.get::<canyon_sql::date_time::DateTime, &str>(#ident_name)
                    .expect(format!("Failed to retrieve the `{}` field", #ident_name).as_ref())
            }
        } else if get_field_type_as_string(ty).replace(' ', "") == "Option<DateTime>" {
            quote! {
                #ident: row.get::<canyon_sql::date_time::DateTime, &str>(#ident_name)
            }
        } else {
            quote! {
                #ident: row.get::<#ty, &str>(#ident_name)
                    .expect(format!("Failed to retrieve the `{}` field", #ident_name).as_ref())
            }
        }
    })
}

#[cfg(feature = "mssql")]
use quote::ToTokens;
#[cfg(feature = "mssql")]
fn get_field_type_as_string(typ: &Type) -> String {
    match typ {
        Type::Array(type_) => type_.to_token_stream().to_string(),
        Type::BareFn(type_) => type_.to_token_stream().to_string(),
        Type::Group(type_) => type_.to_token_stream().to_string(),
        Type::ImplTrait(type_) => type_.to_token_stream().to_string(),
        Type::Infer(type_) => type_.to_token_stream().to_string(),
        Type::Macro(type_) => type_.to_token_stream().to_string(),
        Type::Never(type_) => type_.to_token_stream().to_string(),
        Type::Paren(type_) => type_.to_token_stream().to_string(),
        Type::Path(type_) => type_.to_token_stream().to_string(),
        Type::Ptr(type_) => type_.to_token_stream().to_string(),
        Type::Reference(type_) => type_.to_token_stream().to_string(),
        Type::Slice(type_) => type_.to_token_stream().to_string(),
        Type::TraitObject(type_) => type_.to_token_stream().to_string(),
        Type::Tuple(type_) => type_.to_token_stream().to_string(),
        Type::Verbatim(type_) => type_.to_token_stream().to_string(),
        _ => "".to_owned(),
    }
}