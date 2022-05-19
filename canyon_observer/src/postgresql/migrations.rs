/// File that contains all the datatypes and logic to perform the migrations
/// over a `PostgreSQL` database

use std::{ops::Not, sync::MutexGuard};
use std::fmt::Debug;
use async_trait::async_trait;

use canyon_crud::crud::Transaction;
use regex::Regex;
use crate::{
    QUERIES_TO_EXECUTE, 
    handler::CanyonHandler
};

use super::register_types::CanyonRegisterEntityField;



/// TODO Document, refactor and clarify the code @gbm25
#[derive(Debug)]
pub struct DatabaseSyncOperations {
    operations: Vec<Box<dyn DatabaseOperation>>,
    constrains_operations: Vec<Box<dyn DatabaseOperation>>,
}

impl Transaction<Self> for DatabaseSyncOperations {}
impl DatabaseSyncOperations {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
            constrains_operations: Vec::new(),
        }
    }

    pub async fn fill_operations(&mut self, data: CanyonHandler<'_>) {

        // For each entity (table) on the register
        for mut canyon_register_entity in data.canyon_tables.clone() {

            // Check if the table contains an ID column
            let entity_contains_id = canyon_register_entity.entity_fields.iter()
                .any(|x|x.field_name.to_uppercase() == "ID");

            if entity_contains_id.not(){
                let id_entity = CanyonRegisterEntityField {
                    field_name: "id".to_string(),
                    field_type: "i32".to_string(),
                    annotation: None };

                canyon_register_entity.entity_fields.insert(0,id_entity);
            }

            let table_name = canyon_register_entity.entity_name.to_owned();

            // true if this table on the register is already on the database
            let table_on_database = data.database_tables
                .iter()
                .any(|v| v.table_name == table_name);

            // If the table isn't on the database we push a new operation to the collection
            if table_on_database.not() {
                self.operations.push(
                    Box::new(
                        TableOperation::CreateTable(
                            table_name.clone(),
                            canyon_register_entity.entity_fields.clone(),
                        )
                    )
                );
                for field in canyon_register_entity.entity_fields
                    .clone()
                    .iter()
                    .filter(|column| column.annotation.is_some()) 
                {
                    if field.annotation.as_ref().unwrap().starts_with("Annotation: ForeignKey") {
                        let foreign_key_name = format!("{}_{}_fkey", &table_name, &field.field_name);

                        // Will contain the table name (on index 0) and column name (on index 1) pointed to by the foreign key
                        let annotation_data: Vec<String> = field.annotation.as_ref().unwrap()
                            .split(',')
                            // Deletes the first element of the field annotation String identifier
                            .filter( |x| !x.contains("Annotation: ForeignKey")) // After here, we only have the "table" and the "column" attribute values
                            .map( |x| 
                                x.split(':').collect::<Vec<&str>>()
                                .get(1)
                                .unwrap()
                                .trim()
                                .to_string()
                            ).collect();

                        self.constrains_operations.push(
                            Box::new(
                                TableOperation::AddTableForeignKey(
                                    // table_name, foreign_key_name, column_foreign_key, table_to_reference, column_to_reference
                                    table_name.clone(), foreign_key_name, field.field_name.clone(),
                                    annotation_data.get(0).unwrap().to_string(), annotation_data.get(1).unwrap().to_string(),
                                )
                            )
                        );
                    }
                }
            } else {
                // We check if each of the columns in this table of the register is in the database table.
                // We get the names and add them to a vector of strings
                let columns_in_table: Vec<String> = canyon_register_entity.entity_fields.iter()
                    .filter(|a| data.database_tables.iter()
                        .find(|x| x.table_name == table_name).unwrap().columns
                        .iter()
                        .map(|x| x.column_name.to_string())
                        .any(|x| x == a.field_name))
                    .map(|a| a.field_name.to_string()).collect();

                // For each field (name,type) in this table of the register
                for field in &canyon_register_entity.entity_fields {

                    // Case when the column doesnt exist on the database
                    // We push a new column operation to the collection for each one
                    if columns_in_table.contains(&field.field_name).not() {
                        self.operations.push(
                            Box::new(
                                ColumnOperation::CreateColumn(
                                    table_name.clone(), field.clone(),
                                )
                            )
                        );

                        // If field contains a foreign key annotation, add it to constrains_operations
                        if field.annotation.is_some() && field.annotation.as_ref().unwrap().starts_with("Annotation: ForeignKey") {
                            let foreign_key_name = format!("{}_{}_fkey", &table_name, &field.field_name);
    
                            // Will contain the table name (on index 0) and column name (on index 1) pointed to by the foreign key
                            let annotation_data: Vec<String> = field.annotation.as_ref().unwrap()
                                .split(',')
                                // Deletes the first element of the field annotation String identifier
                                .filter( |x| !x.contains("Annotation: ForeignKey")) // After here, we only have the "table" and the "column" attribute values
                                .map( |x| 
                                    x.split(':').collect::<Vec<&str>>()
                                    .get(1)
                                    .unwrap()
                                    .trim()
                                    .to_string()
                                ).collect();

                            self.constrains_operations.push(
                                Box::new(
                                    TableOperation::AddTableForeignKey(
                                        // table_name, foreign_key_name, column_foreign_key, table_to_reference, column_to_reference
                                        table_name.clone(), foreign_key_name, field.field_name.clone(),
                                        annotation_data.get(0).unwrap().to_string(), annotation_data.get(1).unwrap().to_string(),
                                    )
                                )
                            );
                        }
                    }
                    // Case when the column exist on the database
                    else {
                        let database_field = &data.database_tables.iter()
                            .find(|x| x.table_name == table_name)
                            .unwrap().columns
                            .iter().find(|x| x.column_name == field.field_name).unwrap();

                        let mut database_field_postgres_type: String = String::new();
                        match database_field.postgres_datatype.as_str() {
                            "integer" => {
                                database_field_postgres_type.push_str("i32");
                            }
                            "bigint" => {
                                database_field_postgres_type.push_str("i64");
                            }
                            "text" | "character varying" => {
                                database_field_postgres_type.push_str("String");
                            }
                            "date" => {
                                database_field_postgres_type.push_str("NaiveDate");
                            }
                            _ => {}
                        }

                        if database_field.is_nullable && field.field_type.to_uppercase().starts_with("OPTION") {
                            database_field_postgres_type = format!("Option<{}>", database_field_postgres_type);
                        }

                        if field.field_type != database_field_postgres_type {
                            self.operations.push(
                                Box::new(
                                    ColumnOperation::AlterColumnType(
                                        table_name.clone(), field.clone(),
                                    )
                                )
                            );
                        }

                        // Case when field contains a foreign key annotation, and it's not already on database, add it to constrains_operations
                        if field.annotation.is_some() && database_field.foreign_key_name.is_none() && field.annotation.as_ref().unwrap().starts_with("Annotation: ForeignKey") {
                            let foreign_key_name = format!("{}_{}_fkey", &table_name, &field.field_name);
    
                            // Will contain the table name (on index 0) and column name (on index 1) pointed to by the foreign key
                            let annotation_data: Vec<String> = field.annotation.as_ref().unwrap()
                                .split(',')
                                // Deletes the first element of the field annotation String identifier
                                .filter( |x| !x.contains("Annotation: ForeignKey")) // After here, we only have the "table" and the "column" attribute values
                                .map( |x| 
                                    x.split(':').collect::<Vec<&str>>()
                                    .get(1)
                                    .unwrap()
                                    .trim()
                                    .to_string()
                                ).collect();

                            self.constrains_operations.push(
                                Box::new(
                                    TableOperation::AddTableForeignKey(
                                        // table_name, foreign_key_name, column_foreign_key, table_to_reference, column_to_reference
                                        table_name.clone(), foreign_key_name, field.field_name.clone(),
                                        annotation_data.get(0).unwrap().to_string(), annotation_data.get(1).unwrap().to_string(),
                                    )
                                )
                            );
                        }
                        // Case when field contains a foreign key annotation, and there is already one in the database
                        else if field.annotation.is_some() && database_field.foreign_key_name.is_some() && field.annotation.as_ref().unwrap().starts_with("Annotation: ForeignKey") {
                            let foreign_key_name = format!("{}_{}_fkey", &table_name, &field.field_name);
    
                            // Will contain the table name (on index 0) and column name (on index 1) pointed to by the foreign key
                            let annotation_data: Vec<String> = field.annotation.as_ref().unwrap()
                                .split(',')
                                // Deletes the first element of the field annotation String identifier
                                .filter( |x| !x.contains("Annotation: ForeignKey")) // After here, we only have the "table" and the "column" attribute values
                                .map( |x| 
                                    x.split(':').collect::<Vec<&str>>()
                                    .get(1)
                                    .unwrap()
                                    .trim()
                                    .to_string()
                                ).collect();

                            // Example of information in foreign_key_info: FOREIGN KEY (league) REFERENCES leagues(id)
                            let references_regex = Regex::new(r"\w+\s\w+\s\((?P<current_column>\w+)\)\s\w+\s(?P<ref_table>\w+)\((?P<ref_column>\w+)\)").unwrap();

                            let captures_references = references_regex.captures(database_field.foreign_key_info.as_ref().unwrap()).unwrap();

                            let current_column = captures_references.name("current_column").unwrap().as_str().to_string();
                            let ref_table = captures_references.name("ref_table").unwrap().as_str().to_string();
                            let ref_column = captures_references.name("ref_column").unwrap().as_str().to_string();

                            // If entity foreign key is not equal to the one on database, a constrains_operations is added to delete it and add a new one.
                            if field.field_name != current_column || *annotation_data.get(0).unwrap() != ref_table || *annotation_data.get(1).unwrap() != ref_column {
                                self.constrains_operations.push(
                                    Box::new(
                                        TableOperation::DeleteTableForeignKey(
                                            // table_with_foreign_key,constrain_name
                                            table_name.clone(), database_field.foreign_key_name.as_ref().unwrap().to_string(),
                                        )
                                    )
                                );

                                self.constrains_operations.push(
                                    Box::new(
                                        TableOperation::AddTableForeignKey(
                                            // table_name, foreign_key_name, column_foreign_key, table_to_reference, column_to_reference
                                            table_name.clone(), foreign_key_name, field.field_name.clone(),
                                            annotation_data.get(0).unwrap().to_string(), annotation_data.get(1).unwrap().to_string(),
                                        )
                                    )
                                );
                            }
                        }
                        // Case when field don't contains a foreign key annotation, but there is already one in the database
                        else if field.annotation.is_none() && database_field.foreign_key_name.is_none().not() {
                            self.constrains_operations.push(
                                Box::new(
                                    TableOperation::DeleteTableForeignKey(
                                        table_name.clone(), database_field.foreign_key_name.as_ref().unwrap().to_string(),
                                    )
                                )
                            );
                        }
                    }
                }

                // Filter the list of columns in the corresponding table of the database for the current table of the register,
                // and look for columns that don't exist in the table of the register
                let columns_to_remove: Vec<String> = data.database_tables.iter()
                    .find(|x| x.table_name == table_name).unwrap().columns
                    .iter()
                    .filter(|a| canyon_register_entity.entity_fields.iter()
                        .map(|x| x.field_name.to_string())
                        .any(|x| x == a.column_name).not())
                    .map(|a| a.column_name.to_string()).collect();

                // If we have columns to remove, we push a new operation to the vector for each one
                if columns_to_remove.is_empty().not() {
                    for column in &columns_to_remove {
                        self.operations.push(
                            Box::new(
                                ColumnOperation::DeleteColumn(table_name.clone(), column.to_owned())
                            )
                        );
                    }
                }
            }
        }

        for database_table in data.database_tables {
            
            if database_table.table_name == "canyon_memory" {
                continue;
            }

            let to_delete = !data.canyon_tables.clone()
            .iter()
            .map(|canyon_table| &canyon_table.entity_name)
            .collect::<Vec<&String>>()
            .contains(&&database_table.table_name);
            
            if to_delete {
                self.operations.push(
                    Box::new(
                        TableOperation::DropTable(database_table.table_name)
                    )
                )
            }

        }

        for operation in &self.operations {
            operation.execute().await
        }
        for constrain_operation in &self.constrains_operations {
            constrain_operation.execute().await
        }
    }

    pub async fn from_query_register() {
        let queries: &MutexGuard<Vec<String>> = &QUERIES_TO_EXECUTE.lock().unwrap();
        for i in 0..queries.len() - 1 {
            Self::query(
                queries.get(i).unwrap(), 
                &[]
            ).await
            .ok()
            .unwrap();
        }
    }
}

/// TODO Docs
#[async_trait]
trait DatabaseOperation: Debug {
    async fn execute(&self);
}

/// Helper to relate the operations that Canyon should do when it's managing a schema
#[derive(Debug)]
enum TableOperation {
    CreateTable(String, Vec<CanyonRegisterEntityField>),
    DropTable(String),
    // AlterTableName(String, String)  // TODO Implement
    AddTableForeignKey(String, String, String, String, String), // table_name, foreign_key_name, column_foreign_key, table_to_reference, column_to_reference
    DeleteTableForeignKey(String, String), // table_with_foreign_key,constrain_name
}
impl Transaction<Self> for TableOperation { }

#[async_trait]
impl DatabaseOperation for TableOperation {
    async fn execute(&self) {
        let stmt = match &*self {
            TableOperation::CreateTable(table_name, table_fields) =>
                format!(
                    "CREATE TABLE {table_name} ({:?});",
                    table_fields.iter().map(|entity_field|
                        format!("{} {}", entity_field.field_name, entity_field.field_type_to_postgres())
                    ).collect::<Vec<String>>()
                        .join(", ")
                ).replace('"', ""),
            TableOperation::DropTable(table_name) =>
                format!(
                    "DROP TABLE {table_name} CASCADE;"),
            TableOperation::AddTableForeignKey(table_name, foreign_key_name,
                                               column_foreign_key, table_to_reference,
                                               column_to_reference) =>
                format!("ALTER TABLE {table_name} \
                     ADD CONSTRAINT {foreign_key_name} \
                     FOREIGN KEY ({column_foreign_key}) REFERENCES {table_to_reference} ({column_to_reference});"),

            TableOperation::DeleteTableForeignKey(table_with_foreign_key, constrain_name) =>
                format!("ALTER TABLE {table_with_foreign_key} DROP CONSTRAINT {constrain_name};"),
        };

        QUERIES_TO_EXECUTE.lock().unwrap().push(stmt)
    }
}

/// Helper to relate the operations that Canyon should do when a change on a field should
#[derive(Debug)]
enum ColumnOperation {
    CreateColumn(String, CanyonRegisterEntityField),
    DeleteColumn(String, String),
    // AlterColumnName,
    AlterColumnType(String, CanyonRegisterEntityField),
}

impl Transaction<Self> for ColumnOperation { }

#[async_trait]
impl DatabaseOperation for ColumnOperation {
    async fn execute(&self) {
        let stmt = match &*self {
            ColumnOperation::CreateColumn(table_name, entity_field) =>
                format!("ALTER TABLE {table_name} ADD {} {};",entity_field.field_name, entity_field.field_type_to_postgres()),
            ColumnOperation::DeleteColumn(table_name, column_name) =>
                format!("ALTER TABLE {table_name} DROP COLUMN {column_name};"),
            ColumnOperation::AlterColumnType(table_name, entity_field) =>
                format!("ALTER TABLE {table_name} ALTER COLUMN {} TYPE {};", entity_field.field_name, entity_field.field_type_to_postgres())
        };

        QUERIES_TO_EXECUTE.lock().unwrap().push(stmt)
    }
}