///! File that contains all the datatypes and logic to perform the migrations
///! over a target database
 
use async_trait::async_trait;
use canyon_crud::DatabaseType;
use std::fmt::Debug;
use std::{ops::Not, sync::MutexGuard};

use crate::memory::CanyonMemory;
use crate::QUERIES_TO_EXECUTE;
use canyon_crud::crud::Transaction;

use super::information_schema::{TableMetadata, ColumnMetadata};
use super::register_types::{CanyonRegisterEntity, CanyonRegisterEntityField};

/// Responsible of generating the queries to sync the database status with the
/// Rust source code managed by Canyon, for succesfully make the migrations
#[derive(Debug, Default)]
pub struct MigrationsProcessor {
    operations: Vec<Box<dyn DatabaseOperation>>,
}
impl Transaction<Self> for MigrationsProcessor {}

impl MigrationsProcessor {
    pub async fn process<'a>(
        &'a mut self,
        canyon_memory: CanyonMemory,
        canyon_entities: Vec<CanyonRegisterEntity<'a>>,
        database_tables: Vec<&'a TableMetadata>,
        datasource_name: &'a str,
        db_type: DatabaseType
    ) {
        println!("Database tables to play with: {:?}", &database_tables.len());
        // For each entity (table) on the register (Rust structs)
        for canyon_register_entity in canyon_entities {
            let entity_name = canyon_register_entity.entity_name.to_lowercase();
            // 1st operation -> 
            self.create_or_rename_tables(
                &canyon_memory,
                entity_name.as_str(),
                canyon_register_entity.entity_fields.clone(),
                &database_tables
            );

            let current_table_metadata = MigrationsHelper::get_current_table_metadata(
                &canyon_memory,
                entity_name.as_str(),
                &database_tables
            );
            
            self.delete_fields(
                entity_name.as_str(),
                canyon_register_entity.entity_fields.clone(),
                current_table_metadata,
                db_type
            );

            // For each field (column) on the this canyon register entity
            for canyon_register_field in canyon_register_entity.entity_fields {
                
                // We only create oe modify (right now only datatype)
                // the column when the database already contains the table, 
                // if not, the columns are already create in the previous operation (create table)
                if current_table_metadata.is_some(){

                    let current_column_metadata = MigrationsHelper::get_current_column_metadata( 
                        canyon_register_field.field_name.clone(), 
                        current_table_metadata.unwrap()
                    );

                    self.create_or_modify_field (
                        entity_name.as_str(),
                        canyon_register_field,
                        current_column_metadata
                    )
                }
                
            }
        }

        
        // Self::operations_executor().await;
        for operation in &self.operations {
            println!("Operation query: {:?}", &operation);
            operation.execute(db_type).await; // This should be moved again to runtime
        }
        // Self::from_query_register(datasource_name).await;
    }

    /// TODO
    fn create_or_rename_tables<'a>(
        &mut self,
        canyon_memory: &'_ CanyonMemory,
        entity_name: &'a str,
        entity_fields: Vec<CanyonRegisterEntityField>,
        database_tables: &'a [&'a TableMetadata]
    ) {
        // 1st operation -> Check if the current entity is already on the target database.
        // If isn't present (this if case), we 
        if !MigrationsHelper::entity_already_on_database(entity_name, database_tables) {
            // [`CanyonMemory`] holds a HashMap with the tables who changed their name in
            // the Rust side. If this table name is present, we don't create a new table,
            // just rename the already known one
            if canyon_memory.renamed_entities.contains_key(entity_name) {
                self.table_rename(
                    canyon_memory
                        .renamed_entities
                        .get(entity_name) // Get the old entity name (the value)
                        .unwrap()
                        .to_owned(),
                    entity_name.to_string() // Set the new table name
                )
            } else {
                self.create_table(entity_name.to_string(), entity_fields)
            }
            // else { 
            //     todo!()
                // Return some control flag to indicate that we must begin to
                // parse inner elements (columns, constraints...) with the data
                // Also, we must indicate a relation between the old table name
                // and the new one, because the parsing are against the legacy
                // table name, but the queries must be generated against the new
                // table name
            // }
        }
    }




    /// Generates a database agnostic query to change the name of a table
    fn create_table(&mut self, table_name: String, entity_fields: Vec<CanyonRegisterEntityField>) {
        self.operations
            .push(Box::new(TableOperation::CreateTable(table_name, entity_fields)));
    }

    /// Generates a database agnostic query to change the name of a table
    fn table_rename(&mut self, old_table_name: String, new_table_name: String) {
        self.operations
            .push(Box::new(TableOperation::AlterTableName(old_table_name, new_table_name)));
    }


    // Creates or modify (currently only datatype) a column for a given canyon register entity field
    fn delete_fields<'a > (
        &mut self,
        entity_name: &'a str,
        entity_fields: Vec<CanyonRegisterEntityField>,
        current_table_metadata: Option<&'a TableMetadata>,
        db_type: DatabaseType
    ) {
        if current_table_metadata.is_some() {
            println!("Current table metadata :{:?}",current_table_metadata);
            let columns_name_to_delete : Vec<&ColumnMetadata> = current_table_metadata
            .unwrap()
            .columns
            .iter()
            .filter(|db_column| {
                entity_fields
                    .iter()
                    .map(|canyon_field| canyon_field.field_name.to_string())
                    .any (|canyon_field| canyon_field == db_column.column_name)
                    .not()
            })
            .collect();

            for column_metadata in columns_name_to_delete {
                if db_type == DatabaseType::SqlServer && !column_metadata.is_nullable {
                    self.drop_column_not_null(
                        entity_name, 
                        column_metadata.column_name.clone(), 
                        MigrationsHelper::get_datatype_from_column_metadata(
                        entity_name.to_string(), column_metadata
                        )
                    )
                }
                self.delete_column(entity_name, column_metadata.column_name.clone());
            }
        }
    }

    // Creates or modify (currently only datatype) a column for a given canyon register entity field
    fn create_or_modify_field<'a > (
        &mut self,
        entity_name: &'a str,
        canyon_register_entity_field: CanyonRegisterEntityField,
        current_column_metadata: Option<&ColumnMetadata>,
    ) {
        // If we do not retrieve data for this database column, it does not exist yet
        // and therefore has to be created
        if current_column_metadata.is_none() {
            self.create_column(entity_name.to_string(), canyon_register_entity_field)
        }
        else {
            // TODO hay que
        }
    }

    fn delete_column(&mut self, table_name: &str, column_name: String) {
        self.operations
            .push(Box::new(ColumnOperation::DeleteColumn(table_name.to_string(), column_name)));
    }

    fn drop_column_not_null(&mut self, table_name: &str, column_name: String, column_datatype: String) {
        self.operations
            .push(Box::new(ColumnOperation::DropNotNullBeforeDropColumn(table_name.to_string(), column_name, column_datatype)));
    }

    fn create_column(&mut self, table_name: String, field: CanyonRegisterEntityField) {
        self.operations
            .push(Box::new(ColumnOperation::CreateColumn(table_name, field)));
    }

    /// Make the detected migrations for the next Canyon-SQL run
    #[allow(clippy::await_holding_lock)]
    pub async fn from_query_register(datasource_name: &str) {
        let queries: &MutexGuard<Vec<String>> = &QUERIES_TO_EXECUTE.lock().unwrap();

        if queries.len() > 0 {
            for i in 0..queries.len() - 1 {
                let query_to_execute = queries.get(i).unwrap_or_else(|| {
                    panic!("Failed to retrieve query from the register at index: {i}")
                });
    
                Self::query(query_to_execute, [], datasource_name)
                    .await
                    .ok()
                    .unwrap_or_else(|| {
                        panic!("Failed the migration query: {:?}", queries.get(i).unwrap())
                    });
                // TODO Represent failable operation by logging (if configured by the user) to a text file the Result variant
                // TODO Ask for user input?
            }
        } else {
            println!("No migrations queries found to apply")
        }
    }
}

/// Contains helper methods to parse and process the external and internal input data 
/// for the migrations
struct MigrationsHelper;
impl MigrationsHelper {
    /// Checks if a tracked Canyon entity is already present in the database
    fn entity_already_on_database<'a>(
        entity_name: &'a str,
        database_tables: &'a [&'_ TableMetadata],
    ) -> bool {
        println!("Looking for entity: {:?}", entity_name);
        database_tables.iter().any( |v| 
            v.table_name.to_lowercase() == entity_name.to_lowercase()
        )
    }

    // Get the table metadata for a given entity name or his old entity name if the table was renamed.
    fn get_current_table_metadata<'a> (
        canyon_memory: &'_ CanyonMemory,
        entity_name: &'a str,
        database_tables: &'a [&'_ TableMetadata],
    ) -> Option<&'a TableMetadata> {
        let correct_entity_name = canyon_memory.renamed_entities
            .get(&entity_name.to_lowercase())
            .map(|e| e.to_owned())
            .unwrap_or(entity_name.to_string());
        
        database_tables.iter()
            .find(|table_metadata| table_metadata.table_name.to_lowercase() == *correct_entity_name.to_lowercase())
            .map(|e| e.to_owned().clone())

    }

    // Get the column metadata for a given column name
    fn get_current_column_metadata<'a>  (
        column_name: String,
        current_table_metadata: &'a TableMetadata,
    ) -> Option<&'a ColumnMetadata> {
        current_table_metadata.columns.iter()
            .find(|column| column.column_name == column_name)
            .map(|e| e.to_owned().clone())
    }

    fn get_datatype_from_column_metadata<'a> (
        column_name: String,
        current_column_metadata: &'a ColumnMetadata,
    ) -> String {
        // TODO Add all SQL Server text datatypes
        if vec!["nvarchar","varchar"].contains(&current_column_metadata.postgres_datatype.to_lowercase().as_str()) {
            let varchar_len = match &current_column_metadata.character_maximum_length {
                Some(v) => v.to_string(),
                None => "max".to_string()
            };
            
            format!("{}({})", current_column_metadata.postgres_datatype, varchar_len)
        } else { 
            format!("{}", current_column_metadata.postgres_datatype)
        }
    }
}

#[cfg(test)]
mod migrations_helper_tests {
    use crate::constants;
    use super::*;

    const MOCKED_ENTITY_NAME: &str = "League";

    #[test]
    fn test_entity_already_on_database() {
        let parse_result_empty_db_tables = MigrationsHelper::entity_already_on_database(
            MOCKED_ENTITY_NAME, &[]
        );
        // Always should be false
        assert_eq!(parse_result_empty_db_tables, false);

        // Rust has a League entity. Database has a `league` entity. Case should be normalized
        // and a match must raise
        let mocked_league_entity_on_database = MigrationsHelper::entity_already_on_database(
            MOCKED_ENTITY_NAME, &[&constants::mocked_data::TABLE_METADATA_LEAGUE_EX]
        );
        assert!(mocked_league_entity_on_database);

        let mocked_league_entity_on_database = MigrationsHelper::entity_already_on_database(
            MOCKED_ENTITY_NAME, &[&constants::mocked_data::NON_MATCHING_TABLE_METADATA]
        );
        assert_eq!(mocked_league_entity_on_database, false)
    }
}


// /// Responsible of generating the queries to sync the database status with the
// /// Rust source code managed by Canyon, for succesfully make the migrations
// #[derive(Debug, Default)]
// pub struct DatabaseSyncOperations {
//     operations: Vec<Box<dyn DatabaseOperation>>,
//     drop_primary_key_operations: Vec<Box<dyn DatabaseOperation>>,
//     set_primary_key_operations: Vec<Box<dyn DatabaseOperation>>,
//     constrains_operations: Vec<Box<dyn DatabaseOperation>>,
// }

// impl Transaction<Self> for DatabaseSyncOperations {}

// impl DatabaseSyncOperations {
//     pub async fn fill_operations<'a>(
//         &mut self,
//         canyon_memory: CanyonMemory,
//         canyon_tables: Vec<CanyonRegisterEntity<'static>>,
//         database_tables: Vec<TableMetadata>,
//     ) {
//         // For each entity (table) on the register (Rust structs)
//         for canyon_register_entity in canyon_tables {
//             let table_name = canyon_register_entity.entity_name;

//             // true if this table on the register is already on the database
//             let table_on_database = Self::check_table_on_database(table_name, &database_tables);

//             // If the table isn't on the database we push a new operation to the collection,
//             // either to create a new table or to rename an existing one.
//             if !table_on_database {
//                 let table_renamed = canyon_memory.renamed_entities.contains_key(table_name);

//                 // canyon_memory holds a hashmap of the tables who must changed their name.
//                 // If this table name is present, we dont create a new one, just rename
//                 if table_renamed {
//                     // let old_table_name = data.canyon_memory.table_rename.to_owned().get(&table_name.to_owned());
//                     let otn = canyon_memory
//                         .renamed_entities
//                         .get(table_name)
//                         .unwrap()
//                         .to_owned()
//                         .clone();

//                     Self::push_table_rename::<String, &str>(self, otn, table_name);

//                     // TODO Change foreign_key constrain name on database
//                     continue;
//                 }
//                 // If not, we push an operation to create a new one
//                 else {
//                     Self::add_new_table::<&str>(
//                         self,
//                         table_name,
//                         canyon_register_entity.entity_fields.clone(),
//                     );
//                 }

//                 let cloned_fields = canyon_register_entity.entity_fields.clone();
//                 // We iterate over the fields/columns seeking for constrains to add
//                 for field in cloned_fields
//                     .iter()
//                     .filter(|column| !column.annotations.is_empty())
//                 {
//                     field.annotations.iter().for_each(|attr| {
//                         if attr.starts_with("Annotation: ForeignKey") {
//                             Self::add_foreign_key_with_annotation::<&str, &String>(
//                                 self,
//                                 &field.annotations,
//                                 table_name,
//                                 &field.field_name,
//                             );
//                         }
//                         if attr.starts_with("Annotation: PrimaryKey") {
//                             Self::add_primary_key::<&str>(self, table_name, field.clone());

//                             Self::add_identity::<&str>(self, table_name, field.clone());
//                         }
//                     });
//                 }
//             } else {
//                 // We check if each of the columns in this table of the register is in the database table.
//                 // We get the names and add them to a vector of strings
//                 let columns_in_table = Self::columns_in_table(
//                     canyon_register_entity.entity_fields.clone(),
//                     &database_tables,
//                     table_name,
//                 );

//                 // For each field (name, type) in this table of the register
//                 for field in canyon_register_entity.entity_fields.clone() {
//                     // Case when the column doesn't exist on the database
//                     // We push a new column operation to the collection for each one
//                     if !columns_in_table.contains(&field.field_name) {
//                         Self::add_column_to_table::<&str>(self, table_name, field.clone());

//                         // We added the founded constraints on the field attributes
//                         for attr in &field.annotations {
//                             if attr.starts_with("Annotation: ForeignKey") {
//                                 Self::add_foreign_key_with_annotation::<&str, &String>(
//                                     self,
//                                     &field.annotations,
//                                     table_name,
//                                     &field.field_name,
//                                 );
//                             }
//                             if attr.starts_with("Annotation: PrimaryKey") {
//                                 Self::add_primary_key::<&str>(self, table_name, field.clone());

//                                 Self::add_identity::<&str>(self, table_name, field.clone());
//                             }
//                         }
//                     }
//                     // Case when the column exist on the database
//                     else {
//                         let database_table = database_tables
//                             .iter()
//                             .find(|x| x.table_name == table_name)
//                             .unwrap();

//                         let database_field = database_table
//                             .columns
//                             .iter()
//                             .find(|x| x.column_name == field.field_name)
//                             .expect("Field annt exists");

//                         let mut database_field_postgres_type: String = String::new();
//                         match database_field.postgres_datatype.as_str() {
//                             "integer" => {
//                                 database_field_postgres_type.push_str("i32");
//                             }
//                             "bigint" => {
//                                 database_field_postgres_type.push_str("i64");
//                             }
//                             "text" | "character varying" => {
//                                 database_field_postgres_type.push_str("String");
//                             }
//                             "date" => {
//                                 database_field_postgres_type.push_str("NaiveDate");
//                             }
//                             _ => {}
//                         }

//                         if database_field.is_nullable {
//                             database_field_postgres_type =
//                                 format!("Option<{database_field_postgres_type}>");
//                         }

//                         if field.field_type != database_field_postgres_type {
//                             if field.field_type.starts_with("Option") {
//                                 self.constrains_operations.push(Box::new(
//                                     ColumnOperation::AlterColumnDropNotNull(
//                                         table_name,
//                                         field.clone(),
//                                     ),
//                                 ));
//                             } else {
//                                 self.constrains_operations.push(Box::new(
//                                     ColumnOperation::AlterColumnSetNotNull(
//                                         table_name,
//                                         field.clone(),
//                                     ),
//                                 ));
//                             }
//                             Self::change_column_type(self, table_name, field.clone());
//                         }

//                         let field_is_primary_key = field
//                             .annotations
//                             .iter()
//                             .any(|anno| anno.starts_with("Annotation: PrimaryKey"));

//                         let field_is_foreign_key = field
//                             .annotations
//                             .iter()
//                             .any(|anno| anno.starts_with("Annotation: ForeignKey"));
//                         // TODO Checking Foreign Key attrs. Refactor to a database rust attributes matcher
//                         // TODO Evaluate changing the name of the primary key if it already exists in the database

//                         // -------- PRIMARY KEY CASE ----------------------------

//                         // Case when field contains a primary key annotation, and it's not already on database, add it to constrains_operations
//                         if field_is_primary_key && database_field.primary_key_info.is_none() {
//                             Self::add_primary_key::<&str>(self, table_name, field.clone());
//                             Self::add_identity::<&str>(self, table_name, field.clone());
//                         }
//                         // Case when field don't contains a primary key annotation, but there is already one in the database column
//                         else if !field_is_primary_key && database_field.primary_key_info.is_some()
//                         {
//                             Self::drop_primary_key::<String>(
//                                 self,
//                                 table_name.to_string(),
//                                 database_field
//                                     .primary_key_name
//                                     .as_ref()
//                                     .expect("PrimaryKey constrain name not found")
//                                     .to_string(),
//                             );

//                             if database_field.is_identity {
//                                 Self::drop_identity::<&str>(self, table_name, field.clone());
//                             }
//                         }

//                         // -------- FOREIGN KEY CASE ----------------------------

//                         // Case when field contains a foreign key annotation, and it's not already on database, add it to constrains_operations
//                         if field_is_foreign_key && database_field.foreign_key_name.is_none() {
//                             if database_field.foreign_key_name.is_none() {
//                                 Self::add_foreign_key_with_annotation::<&str, &String>(
//                                     self,
//                                     &field.annotations,
//                                     table_name,
//                                     &field.field_name,
//                                 )
//                             }
//                         }
//                         // Case when field contains a foreign key annotation, and there is already one in the database
//                         else if field_is_foreign_key && database_field.foreign_key_name.is_some()
//                         {
//                             // Will contain the table name (on index 0) and column name (on index 1) pointed to by the foreign key
//                             let annotation_data =
//                                 Self::extract_foreign_key_annotation(&field.annotations);

//                             // Example of information in foreign_key_info: FOREIGN KEY (league) REFERENCES leagues(id)
//                             let references_regex = Regex::new(r"\w+\s\w+\s\((?P<current_column>\w+)\)\s\w+\s(?P<ref_table>\w+)\((?P<ref_column>\w+)\)").unwrap();

//                             let captures_references = references_regex
//                                 .captures(
//                                     database_field
//                                         .foreign_key_info
//                                         .as_ref()
//                                         .expect("Regex - foreign key info"),
//                                 )
//                                 .expect("Regex - foreign key info not found");

//                             let current_column = captures_references
//                                 .name("current_column")
//                                 .expect("Regex - Current column not found")
//                                 .as_str()
//                                 .to_string();
//                             let ref_table = captures_references
//                                 .name("ref_table")
//                                 .expect("Regex - Ref tablenot found")
//                                 .as_str()
//                                 .to_string();
//                             let ref_column = captures_references
//                                 .name("ref_column")
//                                 .expect("Regex - Ref column not found")
//                                 .as_str()
//                                 .to_string();

//                             // If entity foreign key is not equal to the one on database, a constrains_operations is added to delete it and add a new one.
//                             if field.field_name != current_column
//                                 || annotation_data.0 != ref_table
//                                 || annotation_data.1 != ref_column
//                             {
//                                 Self::delete_foreign_key_with_references::<String>(
//                                     self,
//                                     table_name.to_string(),
//                                     database_field
//                                         .foreign_key_name
//                                         .as_ref()
//                                         .expect("Annotation foreign key constrain name not found")
//                                         .to_string(),
//                                 );

//                                 Self::add_foreign_key_with_references(
//                                     self,
//                                     annotation_data.0,
//                                     annotation_data.1,
//                                     table_name,
//                                     field.field_name.clone(),
//                                 )
//                             }
//                         }
//                         // Case when field don't contains a foreign key annotation, but there is already one in the database column
//                         else if !field_is_foreign_key && database_field.foreign_key_name.is_some()
//                         {
//                             Self::delete_foreign_key_with_references::<String>(
//                                 self,
//                                 table_name.to_string(),
//                                 database_field
//                                     .foreign_key_name
//                                     .as_ref()
//                                     .expect("ForeignKey constrain name not found")
//                                     .to_string(),
//                             );
//                         }
//                     }
//                 }

//                 // Filter the list of columns in the corresponding table of the database for the current table of the register,
//                 // and look for columns that don't exist in the table of the register
//                 let columns_to_remove: Vec<String> = Self::columns_to_remove(
//                     &database_tables,
//                     canyon_register_entity.entity_fields.clone(),
//                     table_name,
//                 );

//                 // If we have columns to remove, we push a new operation to the vector for each one
//                 if columns_to_remove.is_empty().not() {
//                     for column in &columns_to_remove {
//                         Self::delete_column_from_table(self, table_name, column.to_owned())
//                     }
//                 }
//             }
//         }

        // for operation in &self.operations {
        //     operation.execute().await
        // }
        // for drop_primary_key_operation in &self.drop_primary_key_operations {
        //     drop_primary_key_operation.execute().await
        // }
        // for set_primary_key_operation in &self.set_primary_key_operations {
        //     set_primary_key_operation.execute().await
        // }
        // for constrain_operation in &self.constrains_operations {
        //     constrain_operation.execute().await
        // }
    // }

//     /// Make the detected migrations for the next Canyon-SQL run
//     /// TODO This should be deprecated, migrations queries can be run without need
//     /// an static item to hold the operations
//     #[allow(clippy::await_holding_lock)]
//     pub async fn from_query_register() {
//         let queries: &MutexGuard<Vec<String>> = &QUERIES_TO_EXECUTE.lock().unwrap();

//         for i in 0..queries.len() - 1 {
//             let query_to_execute = queries.get(i).unwrap_or_else(|| {
//                 panic!("Failed to retrieve query from the register at index: {i}")
//             });

//             Self::query(query_to_execute, [], "")
//                 .await
//                 .ok()
//                 .unwrap_or_else(|| {
//                     panic!("Failed the migration query: {:?}", queries.get(i).unwrap())
//                 });
//             // TODO Represent failable operation by logging (if configured by the user) to a text file the Result variant
//             // TODO Ask for user input?
//         }
//     }

//     fn check_table_on_database<'a>(
//         table_name: &'a str,
//         database_tables: &[TableMetadata],
//     ) -> bool {
//         database_tables.iter().any(|v| v.table_name == table_name)
//     }

//     fn columns_in_table(
//         canyon_columns: Vec<CanyonRegisterEntityField>,
//         database_tables: &[TableMetadata],
//         table_name: &str,
//     ) -> Vec<String> {
//         canyon_columns
//             .iter()
//             .filter(|a| {
//                 database_tables
//                     .iter()
//                     .find(|x| x.table_name == table_name)
//                     .expect("Error collecting database tables")
//                     .columns
//                     .iter()
//                     .map(|x| x.column_name.to_string())
//                     .any(|x| x == a.field_name)
//             })
//             .map(|a| a.field_name.to_string())
//             .collect()
//     }

//     fn columns_to_remove(
//         database_tables: &[TableMetadata],
//         canyon_columns: Vec<CanyonRegisterEntityField>,
//         table_name: &str,
//     ) -> Vec<String> {
//         database_tables
//             .iter()
//             .find(|x| x.table_name == table_name)
//             .expect("Error parsing the columns to remove")
//             .columns
//             .iter()
//             .filter(|a| {
//                 canyon_columns
//                     .iter()
//                     .map(|x| x.field_name.to_string())
//                     .any(|x| x == a.column_name)
//                     .not()
//             })
//             .map(|a| a.column_name.to_string())
//             .collect()
//     }

//     fn push_table_rename<T, U>(&mut self, old_table_name: T, new_table_name: U)
//     where
//         T: Into<String> + Debug + Display + Sync + 'static,
//         U: Into<String> + Debug + Display + Sync + 'static,
//     {
//         self.operations
//             .push(Box::new(TableOperation::AlterTableName::<
//                 _,
//                 _,
//                 &str,
//                 &str,
//                 &str,
//             >(old_table_name, new_table_name)));
//     }

//     fn add_new_table<T>(&mut self, table_name: T, columns: Vec<CanyonRegisterEntityField>)
//     where
//         T: Into<String> + Debug + Display + Sync + 'static,
//     {
//         // self.operations.push(Box::new(TableOperation::CreateTable::<
//         //     _,
//         //     &str,
//         //     &str,
//         //     &str,
//         //     &str,
//         // >(table_name, columns)));
//     }

//     fn extract_foreign_key_annotation(field_annotations: &[String]) -> (String, String) {
//         let opt_fk_annotation = field_annotations
//             .iter()
//             .find(|anno| anno.starts_with("Annotation: ForeignKey"));
//         if let Some(fk_annotation) = opt_fk_annotation {
//             let annotation_data = fk_annotation
//                 .split(',')
//                 .filter(|x| !x.contains("Annotation: ForeignKey")) // After here, we only have the "table" and the "column" attribute values
//                 .map(|x| {
//                     x.split(':')
//                         .collect::<Vec<&str>>()
//                         .get(1)
//                         .expect("Error. Unable to split annotations")
//                         .trim()
//                         .to_string()
//                 })
//                 .collect::<Vec<String>>();

//             let table_to_reference = annotation_data
//                 .get(0)
//                 .expect("Error extracting table ref from FK annotation")
//                 .to_string();
//             let column_to_reference = annotation_data
//                 .get(1)
//                 .expect("Error extracting column ref from FK annotation")
//                 .to_string();

//             (table_to_reference, column_to_reference)
//         } else {
//             panic!("Detected a Foreign Key attribute when does not exists on the user's code");
//         }
//     }

//     fn add_foreign_key_with_annotation<U, V>(
//         &mut self,
//         field_annotations: &[String],
//         table_name: U,
//         column_foreign_key: V,
//     ) where
//         U: Into<String> + Debug + Display + Sync,
//         V: Into<String> + Debug + Display + Sync,
//     {
//         let annotation_data = Self::extract_foreign_key_annotation(field_annotations);

//         let table_to_reference = annotation_data.0;
//         let column_to_reference = annotation_data.1;

//         let foreign_key_name = format!("{table_name}_{}_fkey", &column_foreign_key);

//         self.constrains_operations
//             .push(Box::new(TableOperation::AddTableForeignKey::<
//                 String,
//                 String,
//                 String,
//                 String,
//                 String,
//             >(
//                 table_name.to_string(),
//                 foreign_key_name,
//                 column_foreign_key.to_string(),
//                 table_to_reference,
//                 column_to_reference,
//             )));
//     }

//     fn add_foreign_key_with_references<T, U, V, W>(
//         &mut self,
//         table_to_reference: T,
//         column_to_reference: U,
//         table_name: V,
//         column_foreign_key: W,
//     ) where
//         T: Into<String> + Debug + Display + Sync + 'static,
//         U: Into<String> + Debug + Display + Sync + 'static,
//         V: Into<String> + Debug + Display + Sync + 'static,
//         W: Into<String> + Debug + Display + Sync + 'static,
//     {
//         let foreign_key_name = format!("{}_{}_fkey", &table_name, &column_foreign_key);

//         self.constrains_operations
//             .push(Box::new(TableOperation::AddTableForeignKey(
//                 table_name,
//                 foreign_key_name,
//                 column_foreign_key,
//                 table_to_reference,
//                 column_to_reference,
//             )));
//     }

//     fn delete_foreign_key_with_references<T>(
//         &mut self,
//         table_with_foreign_key: T,
//         constrain_name: T,
//     ) where
//         T: Into<String> + Debug + Display + Sync + 'static,
//     {
//         self.constrains_operations
//             .push(Box::new(TableOperation::DeleteTableForeignKey::<
//                 T,
//                 T,
//                 T,
//                 T,
//                 T,
//             >(
//                 // table_with_foreign_key,constrain_name
//                 table_with_foreign_key,
//                 constrain_name,
//             )));
//     }

//     fn add_primary_key<T>(&mut self, table_name: T, field: CanyonRegisterEntityField)
//     where
//         T: Into<String> + Debug + Display + Sync + 'static,
//     {
//         self.set_primary_key_operations
//             .push(Box::new(
//                 TableOperation::AddTablePrimaryKey::<T, T, T, T, T>(table_name, field),
//             ));
//     }

//     fn drop_primary_key<T>(&mut self, table_name: T, primary_key_name: T)
//     where
//         T: Into<String> + Debug + Display + Sync + 'static,
//     {
//         self.drop_primary_key_operations
//             .push(Box::new(TableOperation::DeleteTablePrimaryKey::<
//                 T,
//                 T,
//                 T,
//                 T,
//                 T,
//             >(table_name, primary_key_name)));
//     }

//     fn add_identity<T>(&mut self, table_name: T, field: CanyonRegisterEntityField)
//     where
//         T: Into<String> + Debug + Display + Sync + 'static,
//     {
//         self.constrains_operations
//             .push(Box::new(ColumnOperation::AlterColumnAddIdentity(
//                 table_name.to_string(),
//                 field.clone(),
//             )));

//         self.constrains_operations
//             .push(Box::new(SequenceOperation::ModifySequence(
//                 table_name, field,
//             )));
//     }

//     fn drop_identity<T>(&mut self, table_name: T, field: CanyonRegisterEntityField)
//     where
//         T: Into<String> + Debug + Display + Sync + 'static,
//     {
//         self.constrains_operations
//             .push(Box::new(ColumnOperation::AlterColumnDropIdentity(
//                 table_name, field,
//             )));
//     }

//     fn add_column_to_table<T>(&mut self, table_name: T, field: CanyonRegisterEntityField)
//     where
//         T: Into<String> + Debug + Display + Sync + 'static,
//     {
//         self.operations
//             .push(Box::new(ColumnOperation::CreateColumn(table_name, field)));
//     }

//     fn change_column_type<T>(&mut self, table_name: T, field: CanyonRegisterEntityField)
//     where
//         T: Into<String> + Debug + Display + Sync + 'static,
//     {
//         self.operations
//             .push(Box::new(ColumnOperation::AlterColumnType(
//                 table_name, field,
//             )));
//     }

//     fn delete_column_from_table<T>(&mut self, table_name: T, column: String)
//     where
//         T: Into<String> + Debug + Display + Sync + 'static,
//     {
//         self.operations
//             .push(Box::new(ColumnOperation::DeleteColumn(table_name, column)));
//     }
// }

/// Trait that enables implementors to execute migration queries
#[async_trait]
trait DatabaseOperation: Debug {
    async fn execute(&self, db_type: DatabaseType);
}

/// Helper to relate the operations that Canyon should do when it's managing a schema
#[derive(Debug)]
enum TableOperation {
    CreateTable(String, Vec<CanyonRegisterEntityField>),
    // old table_name, new table_name
    AlterTableName(String, String),
    // table_name, foreign_key_name, column_foreign_key, table_to_reference, column_to_reference
    AddTableForeignKey(String, String, String, String, String),
    // table_with_foreign_key, constrain_name
    DeleteTableForeignKey(String, String),
    // table_name, entity_field, column_name
    AddTablePrimaryKey(String, CanyonRegisterEntityField),
    // table_name, constrain_name
    DeleteTablePrimaryKey(String, String),
}

impl<T: Debug> Transaction<T> for TableOperation {}

#[async_trait]
impl DatabaseOperation for TableOperation
{
    async fn execute(&self, db_type: DatabaseType) {
        let stmt = match self {
            TableOperation::CreateTable(table_name, table_fields) => {
                if db_type == DatabaseType::PostgreSql {
                    format!(
                        "CREATE TABLE {:?} ({:?});",
                        table_name,
                        table_fields.iter()
                            .map(|entity_field| format!(
                                "{} {}",
                                entity_field.field_name,
                                entity_field.to_postgres_syntax()
                            )
                        ).collect::<Vec<String>>()
                        .join(", ")
                    ).replace('"', "")
                } else if db_type == DatabaseType::SqlServer {
                    format!(
                        "CREATE TABLE {:?} ({:?});",
                        table_name,
                        table_fields.iter()
                            .map(|entity_field| format!(
                                "{} {}",
                                entity_field.field_name,
                                entity_field.to_sqlserver_syntax()
                            )
                        ).collect::<Vec<String>>()
                        .join(", ")
                    ).replace('"', "")
                } else {
                    todo!()
                }
            },

            TableOperation::AlterTableName(old_table_name, new_table_name) => {
                if db_type == DatabaseType::PostgreSql {
                    format!("ALTER TABLE {:?} RENAME TO {:?};", old_table_name, new_table_name)
                } else if db_type == DatabaseType::SqlServer {
                    format!("exec sp_rename '[{:?}]', '{:?}';", old_table_name, new_table_name)
                } else {
                    todo!()
                }                 
            },

            TableOperation::AddTableForeignKey(
                table_name,
                foreign_key_name,
                column_foreign_key,
                table_to_reference,
                column_to_reference
            ) => format!(
                "ALTER TABLE {:?} ADD CONSTRAINT {:?} \
                FOREIGN KEY ({:?}) REFERENCES {:?} ({:?});",
                table_name, foreign_key_name, column_foreign_key, table_to_reference, column_to_reference
            ),

            TableOperation::DeleteTableForeignKey(table_with_foreign_key, constrain_name) =>
                format!(
                    "ALTER TABLE {:?} DROP CONSTRAINT {:?};",
                    table_with_foreign_key, constrain_name
            ),

            TableOperation::AddTablePrimaryKey(
                table_name,
                entity_field
            ) => format!(
                "ALTER TABLE {:?} ADD PRIMARY KEY (\"{}\");",
                table_name,
                entity_field.field_name
            ),

            TableOperation::DeleteTablePrimaryKey(
                table_name,
                primary_key_name
            ) => format!(
                "ALTER TABLE {:?} DROP CONSTRAINT {:?} CASCADE;",
                table_name,
                primary_key_name
            ),

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
    AlterColumnDropNotNull(String, CanyonRegisterEntityField),
    // SQL server specific operation - SQL server can't drop a NOT NULL column
    DropNotNullBeforeDropColumn(String, String, String),
    AlterColumnSetNotNull(String, CanyonRegisterEntityField),
    // TODO if implement throught annotations, modify for both GENERATED {ALWAYS, BY DEFAULT}
    AlterColumnAddIdentity(String, CanyonRegisterEntityField),
    AlterColumnDropIdentity(String, CanyonRegisterEntityField),
}

impl Transaction<Self> for ColumnOperation {} 

#[async_trait]
impl DatabaseOperation for ColumnOperation
{
    async fn execute(&self, db_type: DatabaseType) {
        let stmt = match self {
            ColumnOperation::CreateColumn(table_name, entity_field) => 
            if db_type == DatabaseType::PostgreSql {
                format!(
                "ALTER TABLE {} ADD COLUMN \"{}\" {};",
                table_name,
                entity_field.field_name,
                entity_field.to_postgres_syntax())
            }  else if db_type == DatabaseType::SqlServer {
                format!(
                    "ALTER TABLE {} ADD \"{}\" {};",
                    table_name,
                    entity_field.field_name,
                    entity_field.to_sqlserver_syntax()
                )
            } else {
                todo!()
            },
            ColumnOperation::DeleteColumn(table_name, column_name) => {
                // TODO Check if operation for SQL server is diferent
                format!("ALTER TABLE {} DROP COLUMN {};", table_name, column_name)
            },
            ColumnOperation::AlterColumnType(table_name, entity_field) => format!(
                "ALTER TABLE {} ALTER COLUMN \"{}\" TYPE {};",
                table_name,
                entity_field.field_name,
                entity_field.to_postgres_alter_syntax()
            ),
            ColumnOperation::AlterColumnDropNotNull(table_name, entity_field) => 
            if db_type == DatabaseType::PostgreSql { 
                format!(
                "ALTER TABLE {:?} ALTER COLUMN \"{}\" DROP NOT NULL;",
                table_name, entity_field.field_name
               )
            }  else if db_type == DatabaseType::SqlServer {
                format!(
                "ALTER TABLE {} ALTER COLUMN {} {} NULL",
                table_name, entity_field.field_name, entity_field.to_sqlserver_alter_syntax()
                )
            } else {
                todo!()
            }

            ColumnOperation::DropNotNullBeforeDropColumn(table_name, column_name, column_datatype) => 
                format!(
                "ALTER TABLE {} ALTER COLUMN {} {} NULL; DECLARE @tableName VARCHAR(MAX) = '{table_name}'
                DECLARE @columnName VARCHAR(MAX) = '{column_name}'
                DECLARE @ConstraintName nvarchar(200)
                SELECT @ConstraintName = Name 
                FROM SYS.DEFAULT_CONSTRAINTS
                WHERE PARENT_OBJECT_ID = OBJECT_ID(@tableName) 
                AND PARENT_COLUMN_ID = (
                    SELECT column_id FROM sys.columns
                    WHERE NAME = @columnName AND object_id = OBJECT_ID(@tableName))
                IF @ConstraintName IS NOT NULL
                    EXEC('ALTER TABLE '+@tableName+' DROP CONSTRAINT ' + @ConstraintName);",
                table_name, column_name, column_datatype
            ),

            ColumnOperation::AlterColumnSetNotNull(table_name, entity_field) => format!(
                "ALTER TABLE {:?} ALTER COLUMN \"{}\" SET NOT NULL;",
                table_name, entity_field.field_name
            ),

            ColumnOperation::AlterColumnAddIdentity(table_name, entity_field) => format!(
                "ALTER TABLE {:?} ALTER COLUMN \"{}\" ADD GENERATED ALWAYS AS IDENTITY;",
                table_name, entity_field.field_name
            ),

            ColumnOperation::AlterColumnDropIdentity(table_name, entity_field) => format!(
                "ALTER TABLE {:?} ALTER COLUMN \"{}\" DROP IDENTITY;",
                table_name, entity_field.field_name
            ),
        };

        QUERIES_TO_EXECUTE.lock().unwrap().push(stmt)
    }
}

/// Helper for operations involving sequences
#[derive(Debug)]
enum SequenceOperation<T: Into<String> + std::fmt::Debug + Sync> {
    ModifySequence(T, CanyonRegisterEntityField),
}

impl<T> Transaction<Self> for SequenceOperation<T> 
    where T: Into<String> + std::fmt::Debug + Sync {}

#[async_trait]
impl<T> DatabaseOperation for SequenceOperation<T>
    where T: Into<String> + std::fmt::Debug + Sync
{
    async fn execute(&self, db_type: DatabaseType) {
        let stmt = match self {
            SequenceOperation::ModifySequence(table_name, entity_field) => format!(
                "SELECT setval(pg_get_serial_sequence('{:?}', '{}'), max(\"{}\")) from {:?};",
                table_name, entity_field.field_name, entity_field.field_name, table_name
            ),
        };
        QUERIES_TO_EXECUTE.lock().unwrap().push(stmt)
    }
}
