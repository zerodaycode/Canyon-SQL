#[cfg(feature = "postgres")] use crate::constants::postgresql_type;
#[cfg(feature = "mssql")] use crate::constants::sqlserver_type;
use crate::constants::{regex_patterns, rust_type};

pub mod handler;
pub mod information_schema;
pub mod memory;
pub mod processor;
pub mod transforms;

use canyon_entities::register_types::CanyonRegisterEntityField;
use regex::Regex;

/// Return the postgres datatype and parameters to create a column for a given rust type
#[cfg(feature = "postgres")]
pub fn to_postgres_syntax(field: &CanyonRegisterEntityField) -> String {
    let rust_type_clean = field.field_type.replace(' ', "");

    match rust_type_clean.as_str() {
        rust_type::I8 | rust_type::U8 => {
            String::from(&format!("{} NOT NULL", postgresql_type::INTEGER))
        }
        rust_type::OPT_I8 | rust_type::OPT_U8 => String::from(postgresql_type::INTEGER),

        rust_type::I16 | rust_type::U16 => {
            String::from(&format!("{} NOT NULL", postgresql_type::INTEGER))
        }
        rust_type::OPT_I16 | rust_type::OPT_U16 => String::from(postgresql_type::INTEGER),

        rust_type::I32 | rust_type::U32 => {
            String::from(&format!("{} NOT NULL", postgresql_type::INTEGER))
        }
        rust_type::OPT_I32 | rust_type::OPT_U32 => String::from(postgresql_type::INTEGER),

        rust_type::I64 | rust_type::U64 => {
            String::from(&format!("{} NOT NULL", postgresql_type::BIGINT))
        }
        rust_type::OPT_I64 | rust_type::OPT_U64 => String::from(postgresql_type::BIGINT),

        rust_type::STRING => String::from(&format!("{} NOT NULL", postgresql_type::TEXT)),
        rust_type::OPT_STRING => String::from(postgresql_type::TEXT),

        rust_type::BOOL => String::from(&format!("{} NOT NULL", postgresql_type::BOOLEAN)),
        rust_type::OPT_BOOL => String::from(postgresql_type::BOOLEAN),

        rust_type::NAIVE_DATE => String::from(&format!("{} NOT NULL", postgresql_type::DATE)),
        rust_type::OPT_NAIVE_DATE => String::from(postgresql_type::DATE),

        rust_type::NAIVE_TIME => String::from(&format!("{} NOT NULL", postgresql_type::TIME)),
        rust_type::OPT_NAIVE_TIME => String::from(postgresql_type::TIME),

        rust_type::NAIVE_DATE_TIME => {
            String::from(&format!("{} NOT NULL", postgresql_type::DATETIME))
        }
        rust_type::OPT_NAIVE_DATE_TIME => String::from(postgresql_type::DATETIME),
        &_ => todo!("Not supported datatype for this migrations version"),
    }
}

/// Return the postgres datatype and parameters to create a column for a given rust type
/// for Microsoft SQL Server
#[cfg(feature = "mssql")]
pub fn to_sqlserver_syntax(field: &CanyonRegisterEntityField) -> String {
    let rust_type_clean = field.field_type.replace(' ', "");

    match rust_type_clean.as_str() {
        rust_type::I8 | rust_type::U8 => {
            String::from(&format!("{} NOT NULL", sqlserver_type::INT))
        }
        rust_type::OPT_I8 | rust_type::OPT_U8 => String::from(sqlserver_type::INT),

        rust_type::I16 | rust_type::U16 => {
            String::from(&format!("{} NOT NULL", sqlserver_type::INT))
        }
        rust_type::OPT_I16 | rust_type::OPT_U16 => String::from(sqlserver_type::INT),

        rust_type::I32 | rust_type::U32 => {
            String::from(&format!("{} NOT NULL", sqlserver_type::INT))
        }
        rust_type::OPT_I32 | rust_type::OPT_U32 => String::from(sqlserver_type::INT),

        rust_type::I64 | rust_type::U64 => {
            String::from(&format!("{} NOT NULL", sqlserver_type::BIGINT))
        }
        rust_type::OPT_I64 | rust_type::OPT_U64 => String::from(sqlserver_type::BIGINT),

        rust_type::STRING => {
            String::from(&format!("{} NOT NULL DEFAULT ''", sqlserver_type::NVARCHAR))
        }
        rust_type::OPT_STRING => String::from(sqlserver_type::NVARCHAR),

        rust_type::BOOL => String::from(&format!("{} NOT NULL", sqlserver_type::BIT)),
        rust_type::OPT_BOOL => String::from(sqlserver_type::BIT),

        rust_type::NAIVE_DATE => String::from(&format!("{} NOT NULL", sqlserver_type::DATE)),
        rust_type::OPT_NAIVE_DATE => String::from(sqlserver_type::DATE),

        rust_type::NAIVE_TIME => String::from(&format!("{} NOT NULL", sqlserver_type::TIME)),
        rust_type::OPT_NAIVE_TIME => String::from(sqlserver_type::TIME),

        rust_type::NAIVE_DATE_TIME => {
            String::from(&format!("{} NOT NULL", sqlserver_type::DATETIME))
        }
        rust_type::OPT_NAIVE_DATE_TIME => String::from(sqlserver_type::DATETIME),
        &_ => todo!("Not supported datatype for this migrations version"),
    }
}

#[cfg(feature = "postgres")]
pub fn to_postgres_alter_syntax(field: &CanyonRegisterEntityField) -> String {
    let mut rust_type_clean = field.field_type.replace(' ', "");
    let rs_type_is_optional = field.field_type.to_uppercase().starts_with("OPTION");

    if rs_type_is_optional {
        let type_regex = Regex::new(regex_patterns::EXTRACT_RUST_OPT_REGEX).unwrap();
        let capture_rust_type = type_regex.captures(rust_type_clean.as_str()).unwrap();
        rust_type_clean = capture_rust_type
            .name("rust_type")
            .unwrap()
            .as_str()
            .to_string();
    }

    match rust_type_clean.as_str() {
        rust_type::I8 | rust_type::U8 | rust_type::OPT_I8 | rust_type::OPT_U8 => {
            String::from(postgresql_type::INT_8)
        }
        rust_type::I16 | rust_type::U16 | rust_type::OPT_I16 | rust_type::OPT_U16 => {
            String::from(postgresql_type::SMALL_INT)
        }
        rust_type::I32 | rust_type::U32 | rust_type::OPT_I32 | rust_type::OPT_U32 => {
            String::from(postgresql_type::INTEGER)
        }
        rust_type::I64 | rust_type::U64 | rust_type::OPT_I64 | rust_type::OPT_U64 => {
            String::from(postgresql_type::BIGINT)
        }
        rust_type::STRING | rust_type::OPT_STRING => String::from(postgresql_type::TEXT),
        rust_type::BOOL | rust_type::OPT_BOOL => String::from(postgresql_type::BOOLEAN),
        rust_type::NAIVE_DATE | rust_type::OPT_NAIVE_DATE => {
            String::from(postgresql_type::DATE)
        }
        rust_type::NAIVE_TIME | rust_type::OPT_NAIVE_TIME => {
            String::from(postgresql_type::TIME)
        }
        rust_type::NAIVE_DATE_TIME | rust_type::OPT_NAIVE_DATE_TIME => {
            String::from(postgresql_type::DATETIME)
        }
        &_ => todo!("Not supported datatype for this migrations version"),
    }
}

#[cfg(feature = "mssql")]
pub fn to_sqlserver_alter_syntax(field: &CanyonRegisterEntityField) -> String {
    let mut rust_type_clean = field.field_type.replace(' ', "");
    let rs_type_is_optional = field.field_type.to_uppercase().starts_with("OPTION");

    if rs_type_is_optional {
        let type_regex = Regex::new(regex_patterns::EXTRACT_RUST_OPT_REGEX).unwrap();
        let capture_rust_type = type_regex.captures(rust_type_clean.as_str()).unwrap();
        rust_type_clean = capture_rust_type
            .name("rust_type")
            .unwrap()
            .as_str()
            .to_string();
    }

    match rust_type_clean.as_str() {
        rust_type::I8 | rust_type::U8 | rust_type::OPT_I8 | rust_type::OPT_U8 => {
            String::from(sqlserver_type::TINY_INT)
        }
        rust_type::I16 | rust_type::U16 | rust_type::OPT_I16 | rust_type::OPT_U16 => {
            String::from(sqlserver_type::SMALL_INT)
        }
        rust_type::I32 | rust_type::U32 | rust_type::OPT_I32 | rust_type::OPT_U32 => {
            String::from(sqlserver_type::INT)
        }
        rust_type::I64 | rust_type::U64 | rust_type::OPT_I64 | rust_type::OPT_U64 => {
            String::from(sqlserver_type::BIGINT)
        }
        rust_type::STRING | rust_type::OPT_STRING => String::from(sqlserver_type::NVARCHAR),
        rust_type::BOOL | rust_type::OPT_BOOL => String::from(sqlserver_type::BIT),
        rust_type::NAIVE_DATE | rust_type::OPT_NAIVE_DATE => String::from(sqlserver_type::DATE),
        rust_type::NAIVE_TIME | rust_type::OPT_NAIVE_TIME => String::from(sqlserver_type::TIME),
        rust_type::NAIVE_DATE_TIME | rust_type::OPT_NAIVE_DATE_TIME => {
            String::from(sqlserver_type::DATETIME)
        }
        &_ => todo!("Not supported datatype for this migrations version"),
    }
}