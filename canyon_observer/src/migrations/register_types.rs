use regex::Regex;

use crate::constants::{rust_type, postgresql_type, sqlserver_type, regex_patterns};

/// This file contains `Rust` types that represents an entry on the `CanyonRegister`
/// where `Canyon` tracks the user types that has to manage

/// Gets the necessary identifiers of a CanyonEntity to make it the comparative
/// against the database schemas
#[derive(Debug, Clone, Default)]
pub struct CanyonRegisterEntity<'a> {
    pub entity_name: &'a str,
    pub user_table_name: Option<&'a str>,
    pub user_schema_name: Option<&'a str>,
    pub entity_fields: Vec<CanyonRegisterEntityField>,
}


/// Complementary type for a field that represents a struct field that maps
/// some real database column data
#[derive(Debug, Clone, Default)]
pub struct CanyonRegisterEntityField {
    pub field_name: String,
    pub field_type: String,
    pub annotations: Vec<String>,
}

impl CanyonRegisterEntityField {
    /// Return the postgres datatype and parameters to create a column for a given rust type
    pub fn to_postgres_syntax(&self) -> String {
        let mut rust_type_clean = self.field_type.replace(' ', "");
        let rs_type_is_optional = self.field_type.to_uppercase().starts_with("OPTION");

        if rs_type_is_optional {
            let type_regex =
                Regex::new(regex_patterns::EXTRACT_RUST_OPT_REGEX).unwrap();
            let capture_rust_type = type_regex.captures(rust_type_clean.as_str()).unwrap();
            rust_type_clean = capture_rust_type
                .name("rust_type")
                .unwrap()
                .as_str()
                .to_string();
        }

        let mut postgres_type = String::new();

        match rust_type_clean.as_str() {
            rust_type::I8 | rust_type::U8 => postgres_type.push_str(&format!("{} NOT NULL", postgresql_type::INTEGER)),
            rust_type::OPT_I8 | rust_type::OPT_U8 => postgres_type.push_str(postgresql_type::INTEGER),

            rust_type::I16 | rust_type::U16 => postgres_type.push_str(&format!("{} NOT NULL", postgresql_type::INTEGER)),
            rust_type::OPT_I16 | rust_type::OPT_U16 => postgres_type.push_str(postgresql_type::INTEGER),
            
            rust_type::I32 | rust_type::U32 => postgres_type.push_str(&format!("{} NOT NULL", postgresql_type::INTEGER)),
            rust_type::OPT_I32 | rust_type::OPT_U32 => postgres_type.push_str(postgresql_type::INTEGER),
            
            rust_type::I64 | rust_type::U64 => postgres_type.push_str(&format!("{} NOT NULL", postgresql_type::BIGINT)),
            rust_type::OPT_I64 | rust_type::OPT_U64 => postgres_type.push_str(postgresql_type::BIGINT),
            
            rust_type::STRING => postgres_type.push_str(&format!("{} NOT NULL", postgresql_type::TEXT)),
            rust_type::OPT_STRING => postgres_type.push_str(postgresql_type::TEXT),
            
            rust_type::BOOL => postgres_type.push_str(&format!("{} NOT NULL", postgresql_type::BOOLEAN)),
            rust_type::OPT_BOOL => postgres_type.push_str(postgresql_type::BOOLEAN),
            
            rust_type::NAIVE_DATE => postgres_type.push_str(&format!("{} NOT NULL", postgresql_type::DATE)),
            rust_type::OPT_NAIVE_DATE => postgres_type.push_str(postgresql_type::DATE),
            rust_type::NAIVE_DATE_TIME => postgres_type.push_str(&format!("{} NOT NULL", postgresql_type::DATETIME)),
            rust_type::OPT_NAIVE_DATE_TIME => postgres_type.push_str(postgresql_type::DATETIME),
            &_ => todo!(),
        }

        postgres_type
    }

    /// Return the postgres datatype and parameters to create a column for a given rust type
    /// for Microsoft SQL Server
    pub fn to_sqlserver_syntax(&self) -> String {
        let mut rust_type_clean = self.field_type.replace(' ', "");
        let rs_type_is_optional = self.field_type.to_uppercase().starts_with("OPTION");

        if rs_type_is_optional {
            let type_regex =
                Regex::new(regex_patterns::EXTRACT_RUST_OPT_REGEX).unwrap();
            let capture_rust_type = type_regex.captures(rust_type_clean.as_str()).unwrap();
            rust_type_clean = capture_rust_type
                .name("rust_type")
                .unwrap()
                .as_str()
                .to_string();
        }

        let mut sqlserver_type = String::new();

        match rust_type_clean.as_str() {
            rust_type::I8 | rust_type::U8 => sqlserver_type.push_str(&format!("{} NOT NULL", sqlserver_type::INT)),
            rust_type::OPT_I8 | rust_type::OPT_U8 => sqlserver_type.push_str(sqlserver_type::INT),

            rust_type::I16 | rust_type::U16 => sqlserver_type.push_str(&format!("{} NOT NULL", sqlserver_type::INT)),
            rust_type::OPT_I16 | rust_type::OPT_U16 => sqlserver_type.push_str(sqlserver_type::INT),
            
            rust_type::I32 | rust_type::U32 => sqlserver_type.push_str(&format!("{} NOT NULL", sqlserver_type::INT)),
            rust_type::OPT_I32 | rust_type::OPT_U32 => sqlserver_type.push_str(sqlserver_type::INT),
            
            rust_type::I64 | rust_type::U64 => sqlserver_type.push_str(&format!("{} NOT NULL", sqlserver_type::BIGINT)),
            rust_type::OPT_I64 | rust_type::OPT_U64 => sqlserver_type.push_str(sqlserver_type::BIGINT),
            
            rust_type::STRING => sqlserver_type.push_str(&format!("{} NOT NULL", sqlserver_type::NVARCHAR)),
            rust_type::OPT_STRING => sqlserver_type.push_str(sqlserver_type::NVARCHAR),
            
            rust_type::BOOL => sqlserver_type.push_str(&format!("{} NOT NULL", sqlserver_type::BIT)),
            rust_type::OPT_BOOL => sqlserver_type.push_str(sqlserver_type::BIT),
            
            rust_type::NAIVE_DATE => sqlserver_type.push_str(&format!("{} NOT NULL", sqlserver_type::DATE)),
            rust_type::OPT_NAIVE_DATE => sqlserver_type.push_str(sqlserver_type::DATE),
            rust_type::NAIVE_DATE_TIME => sqlserver_type.push_str(&format!("{} NOT NULL", sqlserver_type::DATETIME)),
            rust_type::OPT_NAIVE_DATE_TIME => sqlserver_type.push_str(sqlserver_type::DATETIME),
            &_ => todo!(),
        }

        sqlserver_type
    }

    pub fn to_postgres_alter_syntax(&self) -> String {
        let mut rust_type_clean = self.field_type.replace(' ', "");
        let rs_type_is_optional = self.field_type.to_uppercase().starts_with("OPTION");

        if rs_type_is_optional {
            let type_regex =
                Regex::new(regex_patterns::EXTRACT_RUST_OPT_REGEX).unwrap();
            let capture_rust_type = type_regex.captures(rust_type_clean.as_str()).unwrap();
            rust_type_clean = capture_rust_type
                .name("rust_type")
                .unwrap()
                .as_str()
                .to_string();
        }

        let mut postgres_type = String::new();

        match rust_type_clean.as_str() {
            rust_type::I32 => postgres_type.push_str("INTEGER"),
            rust_type::OPT_I32 => postgres_type.push_str("INTEGER"),
            rust_type::I64 => postgres_type.push_str("BIGINT"),
            rust_type::OPT_I64 => postgres_type.push_str("BIGINT"),
            rust_type::STRING => postgres_type.push_str("TEXT"),
            rust_type::OPT_STRING => postgres_type.push_str("TEXT"),
            rust_type::BOOL => postgres_type.push_str("BOOLEAN"),
            rust_type::OPT_BOOL => postgres_type.push_str("BOOLEAN"),
            rust_type::NAIVE_DATE => postgres_type.push_str("DATE"),
            rust_type::OPT_NAIVE_DATE => postgres_type.push_str("DATE"),
            &_ => postgres_type.push_str("DATE"),
        }

        postgres_type
    }

    /// Return the datatype and parameters to create an id column, given the corresponding "CanyonRegisterEntityField"
    ///  with the correct format for PostgreSQL
    fn _to_postgres_id_syntax(&self) -> String {
        let has_pk_annotation = self
            .annotations
            .iter()
            .find(|a| a.starts_with("Annotation: PrimaryKey"));

        let pk_is_autoincremental = match has_pk_annotation {
            Some(annotation) => annotation.contains("true"),
            None => false,
        };

        let numeric = vec!["i16", "i32", "i64"];

        let postgres_datatype_syntax = Self::to_postgres_syntax(self);

        if numeric.contains(&self.field_type.as_str()) && pk_is_autoincremental {
            format!("{postgres_datatype_syntax} PRIMARY KEY GENERATED ALWAYS AS IDENTITY")
        } else {
            format!("{postgres_datatype_syntax} PRIMARY KEY")
        }
    }

        /// Return the datatype and parameters to create an id column, given the corresponding "CanyonRegisterEntityField"
        /// with the correct format for Microsoft SQL Server
        fn _to_sqlserver_id_syntax(&self) -> String {
            let has_pk_annotation = self
                .annotations
                .iter()
                .find(|a| a.starts_with("Annotation: PrimaryKey"));
    
            let pk_is_autoincremental = match has_pk_annotation {
                Some(annotation) => annotation.contains("true"),
                None => false,
            };
    
            let numeric = vec!["i16", "i32", "i64"];
    
            let sqlserver_datatype_syntax = Self::to_sqlserver_syntax(self);
    
            if numeric.contains(&self.field_type.as_str()) && pk_is_autoincremental {
                format!("{sqlserver_datatype_syntax} IDENTITY PRIMARY")
            } else {
                format!("{sqlserver_datatype_syntax} PRIMARY KEY")
            }
        }

    /// Return if the field is autoincremental
    pub fn is_autoincremental(&self) -> bool {
        let has_pk_annotation = self
            .annotations
            .iter()
            .find(|a| a.starts_with("Annotation: PrimaryKey"));

        let pk_is_autoincremental = match has_pk_annotation {
            Some(annotation) => annotation.contains("true"),
            None => false,
        };

        let numeric = vec!["i16", "i32", "i64"];

        numeric.contains(&self.field_type.as_str()) && pk_is_autoincremental
    }

}
