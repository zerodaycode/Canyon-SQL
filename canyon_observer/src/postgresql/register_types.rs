use regex::Regex;

/// This file contains `Rust` types that represents an entry of the [`CanyonRegister`]
/// where `Canyon` tracks the user types that has to manage for him

/// Gets the necessary identifiers of a CanyonEntity to make it the comparative
/// against the database schemas
#[derive(Debug, Clone)]
pub struct CanyonRegisterEntity {
    pub entity_name: String,
    pub entity_fields: Vec<CanyonRegisterEntityField>,
}

impl CanyonRegisterEntity {
    pub fn new() -> Self {
        Self {
            entity_name: String::new(),
            entity_fields: Vec::new(),
        }
    }

    /// Returns the String representation for the current "CanyonRegisterEntity" instance.
    /// Being "CanyonRegisterEntity" the representation of a table, the String will be formed by each of its "CanyonRegisterEntityField",
    /// formatting each as "name of the column" "postgres representation of the type" "parameters for the column"
    ///
    ///
    /// ```
    /// let my_id_field = CanyonRegisterEntityField {
    ///                       field_name: "id".to_string(),
    ///                       field_type: "i32".to_string(),
    ///                       annotation: None
    ///                   };
    ///
    /// let my_name_field = CanyonRegisterEntityField {
    ///                          field_name: "name".to_string(),
    ///                          field_type: "String".to_string(),
    ///                          annotation: None
    ///                     };
    ///
    /// let my_canyon_register_entity = CanyonRegisterEntity {
    ///                                    entity_name: String,
    ///                                    entity_fields: vec![my_id_field,my_name_field]
    ///                                 };
    ///
    ///
    /// let expected_result = "id INTEGER NOT NULL PRIMARY KEY GENERATED ALWAYS AS IDENTITY, name TEXT NOT NULL";
    ///
    /// assert_eq!(expected_result, my_canyon_register_entity.entity_fields_as_string());
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
#[derive(Debug, Clone)]
pub struct CanyonRegisterEntityField {
    pub field_name: String,
    pub field_type: String,
    pub annotation: Option<String>
}

impl CanyonRegisterEntityField {
    pub fn new() -> CanyonRegisterEntityField {
        Self {
            field_name: String::new(),
            field_type: String::new(),
            annotation: None
        }
    }

    /// Return the postgres datatype and parameters to create a column for a given rust type
    /// # Examples:
    ///
    /// Basic use:
    /// ```
    /// let my_name_field =  CanyonRegisterEntityField {
    ///                          field_name: "name".to_string(),
    ///                          field_type: "String".to_string(),
    ///                          annotation: None
    ///                      };
    ///
    /// assert_eq!("TEXT NOT NULL", to_postgres_syntax.field_type_to_postgres());
    /// ```
    /// Also support Option:
    /// ```
    /// let my_age_field =  CanyonRegisterEntityField {
    ///                        field_name: "age".to_string(),
    ///                        field_type: "Option<i32>".to_string(),
    ///                        annotation: None
    ///                     };
    ///
    /// assert_eq!("INTEGER", to_postgres_syntax.field_type_to_postgres());
    /// ```
    fn to_postgres_syntax(&self) -> String {
        let mut rust_type_clean = self.field_type.replace(' ',"");
        let rs_type_is_optional =  self.field_type.to_uppercase().starts_with("OPTION");

        if rs_type_is_optional{

            let type_regex = Regex::new(r"[Oo][Pp][Tt][Ii][Oo][Nn]<(?P<rust_type>[\w<>]+)>").unwrap();
            let capture_rust_type = type_regex.captures(rust_type_clean.as_str()).unwrap();
            rust_type_clean = capture_rust_type.name("rust_type").unwrap().as_str().to_string();
        }

        let mut postgres_type = String::new();

        match rust_type_clean.as_str() {
            "i32" => postgres_type.push_str("INTEGER"),
            "i64" =>  postgres_type.push_str("BIGINT"),
            "String" =>  postgres_type.push_str("TEXT"),
            "bool" =>  postgres_type.push_str("BOOLEAN"),
            "NaiveDate" =>  postgres_type.push_str("DATE"),
            &_ => postgres_type.push_str("DATE")
        }

        postgres_type
    }

    /// Return the datatype and parameters to create an id column, given the corresponding "CanyonRegisterEntityField"
    fn to_postgres_id_syntax(&self) -> String {
        let postgres_datatype_syntax = Self::to_postgres_syntax(self);

        format!("{} PRIMARY KEY GENERATED ALWAYS AS IDENTITY", postgres_datatype_syntax)
    }

    pub fn field_type_to_postgres(&self) -> String {
        let column_postgres_syntax = match self.field_name.as_str() {
            "id" => Self::to_postgres_id_syntax(self),
            _ => Self::to_postgres_syntax(self),
        };

        column_postgres_syntax
    }
}