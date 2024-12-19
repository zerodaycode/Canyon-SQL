/// This file contains `Rust` types that represents an entry on the `CanyonRegister`
/// where `Canyon` tracks the user types that has to manage
pub const NUMERIC_PK_DATATYPE: [&str; 6] = ["i16", "u16", "i32", "u32", "i64", "u64"];

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
