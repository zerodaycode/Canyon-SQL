/// File that contains all the datatypes and logic to perform the migrations
/// over a `PostgreSQL` database

use std::{ops::Not, sync::MutexGuard};
use std::fmt::Debug;
use async_trait::async_trait;

use canyon_crud::crud::Transaction;
use regex::Regex;
use crate::{
    QUERIES_TO_EXECUTE,
    handler::CanyonHandler,
};
use crate::postgresql::information_schema::rows_to_table_mapper::DatabaseTable;

use super::register_types::{CanyonRegisterEntity, CanyonRegisterEntityField};


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
                .any(|x| x.field_name.to_uppercase() == "ID");

            // If the table doesnt contain ID, we add it
            if !entity_contains_id {
                let id_entity = CanyonRegisterEntityField {
                    field_name: "id".to_string(),
                    field_type: "i32".to_string(),
                    annotation: None,
                };

                canyon_register_entity.entity_fields.insert(0, id_entity);
            }

            let table_name = canyon_register_entity.entity_name.to_owned();

            // true if this table on the register is already on the database
            let table_on_database = Self::check_table_on_database(&table_name, &data.database_tables);

            // If the table isn't on the database we push a new operation to the collection,
            // either to create a new table or to rename an existing one.
            if !table_on_database {

                // canyon_memory holds a hashmap of the tables who must changed their name.
                // If this table name is present, we dont create a new one, just rename
                if data.canyon_memory.table_rename.contains_key(&table_name) {
                    let new_table_name = data.canyon_memory.table_rename.get(&table_name).unwrap().to_string();

                    Self::push_table_rename(self, table_name.clone(), new_table_name);

                    // TODO Change foreign_key constrain name on database
                    continue;
                }
                // If not, we push an operation to create a new one
                else {
                    Self::add_new_table(self, table_name.clone(), canyon_register_entity.entity_fields.clone());
                }


                // We iterate over the fields/columns seeking for foreign key constrains to add
                for field in canyon_register_entity.entity_fields
                    .clone()
                    .iter()
                    .filter(|column| column.annotation.is_some())
                {
                    if field.annotation.as_ref().unwrap().starts_with("Annotation: ForeignKey") {
                        Self::add_foreign_key_with_annotation(self,
                                                              field.annotation.as_ref().unwrap(),
                                                              table_name.clone(),
                                                              field.field_name.clone(),
                        );
                    }
                }
            } else {
                // We check if each of the columns in this table of the register is in the database table.
                // We get the names and add them to a vector of strings
                let columns_in_table = Self::columns_in_table(
                    canyon_register_entity.entity_fields.clone(),
                    &data.database_tables,
                    table_name.clone(),
                );

                // For each field (name,type) in this table of the register
                for field in &canyon_register_entity.entity_fields {

                    // Case when the column doesnt exist on the database
                    // We push a new column operation to the collection for each one
                    if columns_in_table.contains(&field.field_name).not() {
                        Self::add_column_to_table(self, table_name.clone(), field.clone());

                        // If field contains a foreign key annotation, add it to constrains_operations
                        if field.annotation.is_some() && field.annotation.as_ref().unwrap().starts_with("Annotation: ForeignKey") {
                            Self::add_foreign_key_with_annotation(self,
                                                                  field.annotation.as_ref().unwrap(),
                                                                  table_name.clone(),
                                                                  field.field_name.clone(),
                            )
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
                            Self::change_column_type(self, table_name.clone(), field.clone());
                        }

                        // Case when field contains a foreign key annotation, and it's not already on database, add it to constrains_operations
                        if field.annotation.is_some() && database_field.foreign_key_name.is_none() && field.annotation.as_ref().unwrap().starts_with("Annotation: ForeignKey") {
                            Self::add_foreign_key_with_annotation(self,
                                                                  field.annotation.as_ref().unwrap(),
                                                                  table_name.clone(),
                                                                  field.field_name.clone(),
                            )
                        }
                        // Case when field contains a foreign key annotation, and there is already one in the database
                        else if field.annotation.is_some() && database_field.foreign_key_name.is_some() && field.annotation.as_ref().unwrap().starts_with("Annotation: ForeignKey") {

                            // Will contain the table name (on index 0) and column name (on index 1) pointed to by the foreign key
                            let annotation_data: (String, String) = Self::extract_foreign_key_annotation(field.annotation.as_ref().unwrap());

                            // Example of information in foreign_key_info: FOREIGN KEY (league) REFERENCES leagues(id)
                            let references_regex = Regex::new(r"\w+\s\w+\s\((?P<current_column>\w+)\)\s\w+\s(?P<ref_table>\w+)\((?P<ref_column>\w+)\)").unwrap();

                            let captures_references = references_regex.captures(database_field.foreign_key_info.as_ref().unwrap()).unwrap();

                            let current_column = captures_references.name("current_column").unwrap().as_str().to_string();
                            let ref_table = captures_references.name("ref_table").unwrap().as_str().to_string();
                            let ref_column = captures_references.name("ref_column").unwrap().as_str().to_string();

                            // If entity foreign key is not equal to the one on database, a constrains_operations is added to delete it and add a new one.
                            if field.field_name != current_column || annotation_data.0 != ref_table || annotation_data.1 != ref_column {
                                Self::delete_foreign_key_with_references(
                                    self,
                                    table_name.clone(),
                                    database_field.foreign_key_name.as_ref().unwrap().to_string(),
                                );

                                Self::add_foreign_key_with_references(
                                    self,
                                    annotation_data.0,
                                    annotation_data.1,
                                    table_name.clone(),
                                    field.field_name.clone(),
                                )
                            }
                        }
                        // Case when field don't contains a foreign key annotation, but there is already one in the database
                        else if field.annotation.is_none() && database_field.foreign_key_name.is_none().not() {
                            Self::delete_foreign_key_with_references(
                                self,
                                table_name.clone(),
                                database_field.foreign_key_name.as_ref().unwrap().to_string(),
                            );
                        }
                    }
                }

                // Filter the list of columns in the corresponding table of the database for the current table of the register,
                // and look for columns that don't exist in the table of the register
                let columns_to_remove: Vec<String> = Self::columns_to_remove(
                    &data.database_tables,
                    canyon_register_entity.entity_fields.clone(),
                    table_name.clone(),
                );

                // If we have columns to remove, we push a new operation to the vector for each one
                if columns_to_remove.is_empty().not() {
                    for column in &columns_to_remove {
                        Self::delete_column_from_table(self, table_name.clone(), column.to_owned())
                    }
                }
            }
        }

        for database_table in data.database_tables {

            if database_table.table_name == "canyon_memory" {
                continue;
            }

            let to_delete = Self::is_table_in_db(data.canyon_tables.clone(), &database_table.table_name);

            if to_delete {
                Self::delete_table(self, database_table.table_name)
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
                &[],
            ).await
                .ok()
                .unwrap();
        }
    }

    fn check_table_on_database(table_name: &String, database_tables: &Vec<DatabaseTable<'_>>) -> bool {
        database_tables
            .iter()
            .any(|v| &v.table_name == table_name)
    }

    fn columns_in_table(
        canyon_columns: Vec<CanyonRegisterEntityField>,
        database_tables: &[DatabaseTable<'_>],
        table_name: String,
    ) -> Vec<String> {
        canyon_columns.iter()
            .filter(|a| database_tables.iter()
                .find(|x| x.table_name == table_name).unwrap().columns
                .iter()
                .map(|x| x.column_name.to_string())
                .any(|x| x == a.field_name))
            .map(|a| a.field_name.to_string()).collect()
    }

    fn columns_to_remove(
        database_tables: &[DatabaseTable<'_>],
        canyon_columns: Vec<CanyonRegisterEntityField>,
        table_name: String,
    ) -> Vec<String> {
        database_tables.iter()
            .find(|x| x.table_name == table_name).unwrap().columns
            .iter()
            .filter(|a| canyon_columns.iter()
                .map(|x| x.field_name.to_string())
                .any(|x| x == a.column_name).not())
            .map(|a| a.column_name.to_string()).collect()
    }


    fn is_table_in_db(canyon_tables: Vec<CanyonRegisterEntity>, database_table_name: &str) -> bool {
        !canyon_tables
                .iter()
                .map(|canyon_table| &canyon_table.entity_name)
                .any(|canyon_table_name| canyon_table_name == database_table_name)
    }

    fn push_table_rename(&mut self, old_table_name: String, new_table_name: String) {
        self.operations.push(
            Box::new(
                TableOperation::AlterTableName(
                    old_table_name,
                    new_table_name,
                )
            )
        );
    }

    fn add_new_table(&mut self, table_name: String, columns: Vec<CanyonRegisterEntityField>) {
        self.operations.push(
            Box::new(
                TableOperation::CreateTable(
                    table_name,
                    columns,
                )
            )
        );
    }

    fn delete_table(&mut self, table_name: String) {
        self.operations.push(
            Box::new(
                TableOperation::DropTable(table_name)
            )
        )
    }

    fn extract_foreign_key_annotation(field_annotations: &str) -> (String, String) {
        let annotation_data: Vec<String> = field_annotations
            .split(',')
            // TODO check change (x.contains previously contained a negation)
            .filter(|x| x.contains("Annotation: ForeignKey")) // After here, we only have the "table" and the "column" attribute values
            .map(|x|
                x.split(':').collect::<Vec<&str>>()
                    .get(1)
                    .unwrap()
                    .trim()
                    .to_string()
            ).collect();

        let table_to_reference = annotation_data.get(0).unwrap().to_string();
        let column_to_reference = annotation_data.get(1).unwrap().to_string();

        (table_to_reference, column_to_reference)
    }

    fn add_foreign_key_with_annotation(&mut self,
                                       field_annotations: &str,
                                       table_name: String,
                                       column_foreign_key: String,
    ) {

        // Extraemos de las field_annotation la correspondiente a la foreign key
        let annotation_data: (String, String) = Self::extract_foreign_key_annotation(field_annotations);

        let table_to_reference = annotation_data.0;
        let column_to_reference = annotation_data.1;

        let foreign_key_name = format!("{}_{}_fkey", &table_name, &column_foreign_key);


        self.constrains_operations.push(
            Box::new(
                TableOperation::AddTableForeignKey(
                    table_name, foreign_key_name, column_foreign_key, table_to_reference, column_to_reference,
                )
            )
        );
    }

    fn add_foreign_key_with_references(&mut self,
                                       table_to_reference: String,
                                       column_to_reference: String,
                                       table_name: String,
                                       column_foreign_key: String,
    ) {
        let foreign_key_name = format!("{}_{}_fkey", &table_name, &column_foreign_key);


        self.constrains_operations.push(
            Box::new(
                TableOperation::AddTableForeignKey(
                    table_name, foreign_key_name, column_foreign_key, table_to_reference, column_to_reference,
                )
            )
        );
    }

    fn delete_foreign_key_with_references(&mut self,
                                          table_with_foreign_key: String,
                                          constrain_name: String,
    ) {
        self.constrains_operations.push(
            Box::new(
                TableOperation::DeleteTableForeignKey(
                    // table_with_foreign_key,constrain_name
                    table_with_foreign_key, constrain_name,
                )
            )
        );
    }

    fn add_column_to_table(&mut self, table_name: String, field: CanyonRegisterEntityField) {
        self.operations.push(
            Box::new(
                ColumnOperation::CreateColumn(
                    table_name, field,
                )
            )
        );
    }

    fn change_column_type(&mut self, table_name: String, field: CanyonRegisterEntityField) {
        self.operations.push(
            Box::new(
                ColumnOperation::AlterColumnType(
                    table_name, field,
                )
            )
        );
    }

    fn delete_column_from_table(&mut self, table_name: String, column: String) {
        self.operations.push(
            Box::new(
                ColumnOperation::DeleteColumn(table_name, column)
            )
        );
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
    AlterTableName(String, String),
    // old table_name, new table_name
    AddTableForeignKey(String, String, String, String, String),
    // table_name, foreign_key_name, column_foreign_key, table_to_reference, column_to_reference
    DeleteTableForeignKey(String, String), // table_with_foreign_key,constrain_name
}

impl Transaction<Self> for TableOperation {}

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
            TableOperation::AlterTableName(old_table_name, new_table_name) =>
                format!("ALTER TABLE {old_table_name} RENAME TO  {new_table_name};"),
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

impl Transaction<Self> for ColumnOperation {}

#[async_trait]
impl DatabaseOperation for ColumnOperation {
    async fn execute(&self) {
        let stmt = match &*self {
            ColumnOperation::CreateColumn(table_name, entity_field) =>
                format!("ALTER TABLE {table_name} ADD {} {};", entity_field.field_name, entity_field.field_type_to_postgres()),
            ColumnOperation::DeleteColumn(table_name, column_name) =>
                format!("ALTER TABLE {table_name} DROP COLUMN {column_name};"),
            ColumnOperation::AlterColumnType(table_name, entity_field) =>
                format!("ALTER TABLE {table_name} ALTER COLUMN {} TYPE {};", entity_field.field_name, entity_field.field_type_to_postgres())
        };

        QUERIES_TO_EXECUTE.lock().unwrap().push(stmt)
    }
}