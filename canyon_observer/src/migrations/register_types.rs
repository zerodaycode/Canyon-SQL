use regex::Regex;

use crate::constants::{
    postgresql_type, regex_patterns, rust_type, sqlserver_type, NUMERIC_PK_DATATYPE,
};

/// This file contains `Rust` types that represents an entry on the `CanyonRegister`
/// where `Canyon` tracks the user types that has to manage

/// Gets the necessary identifiers of a CanyonEntity to make it the comparative
/// against the database schemas
#[derive(Debug, Clone, Default)]
pub struct CanyonRegisterEntity<'a> {
    pub entity_name: &'a str,
    pub entity_db_table_name: &'a str,
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
        let rust_type_clean = self.field_type.replace(' ', "");

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
    pub fn to_sqlserver_syntax(&self) -> String {
        let rust_type_clean = self.field_type.replace(' ', "");

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

    pub fn to_postgres_alter_syntax(&self) -> String {
        let mut rust_type_clean = self.field_type.replace(' ', "");
        let rs_type_is_optional = self.field_type.to_uppercase().starts_with("OPTION");

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

    pub fn to_sqlserver_alter_syntax(&self) -> String {
        let mut rust_type_clean = self.field_type.replace(' ', "");
        let rs_type_is_optional = self.field_type.to_uppercase().starts_with("OPTION");

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

        let postgres_datatype_syntax = Self::to_postgres_syntax(self);

        if NUMERIC_PK_DATATYPE.contains(&self.field_type.as_str()) && pk_is_autoincremental {
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

        let sqlserver_datatype_syntax = Self::to_sqlserver_syntax(self);

        if NUMERIC_PK_DATATYPE.contains(&self.field_type.as_str()) && pk_is_autoincremental {
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

        NUMERIC_PK_DATATYPE.contains(&self.field_type.as_str()) && pk_is_autoincremental
    }

    /// Return the nullability of a the field
    pub fn is_nullable(&self) -> bool {
        self.field_type.to_uppercase().starts_with("OPTION")
    }
}
