use regex::Regex;

use crate::constants::{
    rust_type,
    // postgresql_type
};

/// This file contains `Rust` types that represents an entry of the [`CanyonRegister`]
/// where `Canyon` tracks the user types that has to manage for him

/// Gets the necessary identifiers of a CanyonEntity to make it the comparative
/// against the database schemas
#[derive(Debug, Clone, Default)]
pub struct CanyonRegisterEntity<'a> {
    pub entity_name: &'a str,
    pub user_table_name: Option<&'a str>,
    pub user_schema_name: Option<&'a str>,
    pub entity_fields: Vec<CanyonRegisterEntityField>,
}

impl<'a> CanyonRegisterEntity<'a> {
    /// Returns the String representation for the current "CanyonRegisterEntity" instance.
    /// Being "CanyonRegisterEntity" the representation of a table, the String will be formed by each of its "CanyonRegisterEntityField",
    /// formatting each as "name of the column" "postgres representation of the type" "parameters for the column"
    pub fn entity_fields_as_string(&self) -> String {

        let mut fields_strings:Vec<String> = Vec::new();

        for field in &self.entity_fields {

            let column_postgres_syntax = field.field_type_to_postgres();
            let field_as_string = format!("{} {}", field.field_name, column_postgres_syntax);
            fields_strings.push(field_as_string);
        }

            fields_strings.join(" ")
        }
    }

/// Complementary type for a field that represents a struct field that maps
/// some real database column data
#[derive(Debug, Clone, Default)]
pub struct CanyonRegisterEntityField {
    pub field_name: String,
    pub field_type: String,
    pub annotations: Vec<String>
}

impl CanyonRegisterEntityField {
    // pub fn new() -> CanyonRegisterEntityField {
    //     Self {
    //         field_name: String::new(),
    //         field_type: String::new(),
    //         annotations: Vec::new()
    //     }
    // }

    /// Return the postgres datatype and parameters to create a column for a given rust type
    fn to_postgres_syntax(&self) -> String {
        let mut rust_type_clean = self.field_type.replace(' ',"");
        let rs_type_is_optional =  self.field_type.to_uppercase().starts_with("OPTION");

        if rs_type_is_optional {
            let type_regex = Regex::new(r"[Oo][Pp][Tt][Ii][Oo][Nn]<(?P<rust_type>[\w<>]+)>").unwrap();
            let capture_rust_type = type_regex.captures(rust_type_clean.as_str()).unwrap();
            rust_type_clean = capture_rust_type.name("rust_type").unwrap().as_str().to_string();
        }

        let mut postgres_type = String::new();

        match rust_type_clean.as_str() {
            rust_type::I32 => postgres_type.push_str("INTEGER NOT NULL"),
            rust_type::OPT_I32 => postgres_type.push_str("INTEGER"),
            rust_type::I64 =>  postgres_type.push_str("BIGINT NOT NULL"),
            rust_type::OPT_I64 =>  postgres_type.push_str("BIGINT"),
            rust_type::STRING =>  postgres_type.push_str("TEXT NOT NULL"),
            rust_type::OPT_STRING =>  postgres_type.push_str("TEXT"),
            rust_type::BOOL =>  postgres_type.push_str("BOOLEAN NOT NULL"),
            rust_type::OPT_BOOL =>  postgres_type.push_str("BOOLEAN"),
            rust_type::NAIVE_DATE =>  postgres_type.push_str("DATE NOT NULL"),
            rust_type::OPT_NAIVE_DATE =>  postgres_type.push_str("DATE"),
            &_ => postgres_type.push_str("DATE")
        }

        postgres_type
    }

    pub fn to_postgres_alter_syntax(&self) -> String {
        let mut rust_type_clean = self.field_type.replace(' ',"");
        let rs_type_is_optional =  self.field_type.to_uppercase().starts_with("OPTION");

        if rs_type_is_optional {
            let type_regex = Regex::new(r"[Oo][Pp][Tt][Ii][Oo][Nn]<(?P<rust_type>[\w<>]+)>").unwrap();
            let capture_rust_type = type_regex.captures(rust_type_clean.as_str()).unwrap();
            rust_type_clean = capture_rust_type.name("rust_type").unwrap().as_str().to_string();
        }

        let mut postgres_type = String::new();

        match rust_type_clean.as_str() {
            rust_type::I32 => postgres_type.push_str("INTEGER"),
            rust_type::OPT_I32 => postgres_type.push_str("INTEGER"),
            rust_type::I64 =>  postgres_type.push_str("BIGINT"),
            rust_type::OPT_I64 =>  postgres_type.push_str("BIGINT"),
            rust_type::STRING =>  postgres_type.push_str("TEXT"),
            rust_type::OPT_STRING =>  postgres_type.push_str("TEXT"),
            rust_type::BOOL =>  postgres_type.push_str("BOOLEAN"),
            rust_type::OPT_BOOL =>  postgres_type.push_str("BOOLEAN"),
            rust_type::NAIVE_DATE =>  postgres_type.push_str("DATE"),
            rust_type::OPT_NAIVE_DATE =>  postgres_type.push_str("DATE"),
            &_ => postgres_type.push_str("DATE")
        }
        
        postgres_type
    }

    /// Return the datatype and parameters to create an id column, given the corresponding "CanyonRegisterEntityField"
    fn to_postgres_id_syntax(&self) -> String {
        let has_pk_annotation = self.annotations.iter().find(
            |a| a.starts_with("Annotation: PrimaryKey")
        );

        let pk_is_autoincremental = match has_pk_annotation {
            Some(annotation) => annotation.contains("true"),
            None => false
        };

        let numeric = vec!["i16", "i32", "i64"];

        let postgres_datatype_syntax = Self::to_postgres_syntax(self);

        if numeric.contains(&self.field_type.as_str()) && pk_is_autoincremental {
            format!("{} PRIMARY KEY GENERATED ALWAYS AS IDENTITY", postgres_datatype_syntax)
        } else {
            format!("{} PRIMARY KEY", postgres_datatype_syntax)
        }
    }

    /// Return if the field is autoincremental
    pub fn is_autoincremental(&self) -> bool {
        let has_pk_annotation = self.annotations.iter().find(
            |a| a.starts_with("Annotation: PrimaryKey")
        );

        let pk_is_autoincremental = match has_pk_annotation {
            Some(annotation) => annotation.contains("true"),
            None => false
        };

        let numeric = vec!["i16", "i32", "i64"];

        numeric.contains(&self.field_type.as_str()) && pk_is_autoincremental
    }

    pub fn field_type_to_postgres(&self) -> String {
        let is_pk = self.annotations.iter().find(
            |a| a.starts_with("Annotation: PrimaryKey")
        );
        
        match is_pk {
            Some(_) => Self::to_postgres_id_syntax(self),
            None => Self::to_postgres_syntax(self)
        }
    }
}