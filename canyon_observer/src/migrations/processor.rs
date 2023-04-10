///! File that contains all the datatypes and logic to perform the migrations
///! over a target database
use async_trait::async_trait;
use canyon_crud::DatabaseType;
use regex::Regex;
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Not;

use crate::canyon_crud::{crud::Transaction, DatasourceConfig};
use crate::constants::regex_patterns;
use crate::save_migrations_query_to_execute;

use super::information_schema::{ColumnMetadata, TableMetadata};
use super::memory::CanyonMemory;
use super::register_types::{CanyonRegisterEntity, CanyonRegisterEntityField};

/// Responsible of generating the queries to sync the database status with the
/// Rust source code managed by Canyon, for successfully make the migrations
#[derive(Debug, Default)]
pub struct MigrationsProcessor {
    operations: Vec<Box<dyn DatabaseOperation>>,
    set_primary_key_operations: Vec<Box<dyn DatabaseOperation>>,
    drop_primary_key_operations: Vec<Box<dyn DatabaseOperation>>,
    constraints_operations: Vec<Box<dyn DatabaseOperation>>,
}
impl Transaction<Self> for MigrationsProcessor {}

impl MigrationsProcessor {
    pub async fn process<'a>(
        &'a mut self,
        canyon_memory: CanyonMemory,
        canyon_entities: Vec<CanyonRegisterEntity<'a>>,
        database_tables: Vec<&'a TableMetadata>,
        datasource: &'_ DatasourceConfig,
    ) {
        // The database type formally represented in Canyon
        let db_type = datasource.get_db_type();
        // For each entity (table) on the register (Rust structs)
        for canyon_register_entity in canyon_entities {
            let entity_name = canyon_register_entity.entity_db_table_name;
            println!("Processing migrations for entity: {entity_name}");

            // 1st operation ->
            self.create_or_rename_tables(
                &canyon_memory,
                entity_name,
                canyon_register_entity.entity_fields.clone(),
                &database_tables,
            );

            let current_table_metadata = MigrationsHelper::get_current_table_metadata(
                &canyon_memory,
                entity_name,
                &database_tables,
            );

            self.delete_fields(
                entity_name,
                canyon_register_entity.entity_fields.clone(),
                current_table_metadata,
                db_type,
            );

            // For each field (column) on the this canyon register entity
            for canyon_register_field in canyon_register_entity.entity_fields {
                let current_column_metadata = MigrationsHelper::get_current_column_metadata(
                    canyon_register_field.field_name.clone(),
                    current_table_metadata,
                );

                // We only create or modify (right now only datatype)
                // the column when the database already contains the table,
                // if not, the columns are already create in the previous operation (create table)
                if current_table_metadata.is_some() {
                    self.create_or_modify_field(
                        entity_name,
                        db_type,
                        canyon_register_field.clone(),
                        current_column_metadata,
                    )
                }

                // Time to check annotations for the current column
                // Case when  we only need to add constraints
                if (current_table_metadata.is_none()
                    && !canyon_register_field.annotations.is_empty())
                    || (current_table_metadata.is_some() && current_column_metadata.is_none())
                {
                    self.add_constraints(entity_name, canyon_register_field.clone())
                }

                // Case when we need to compare the entity with the database contain
                if current_table_metadata.is_some() && current_column_metadata.is_some() {
                    self.add_modify_or_remove_constraints(
                        entity_name,
                        canyon_register_field,
                        current_column_metadata.unwrap(),
                    )
                }
            }
        }

        for operation in &self.operations {
            operation.generate_sql(datasource).await; // This should be moved again to runtime
        }
        for operation in &self.drop_primary_key_operations {
            operation.generate_sql(datasource).await; // This should be moved again to runtime
        }
        for operation in &self.set_primary_key_operations {
            operation.generate_sql(datasource).await; // This should be moved again to runtime
        }
        for operation in &self.constraints_operations {
            operation.generate_sql(datasource).await; // This should be moved again to runtime
        }
        // TODO Still pending to decouple de executions of cargo check to skip the process if this
        // code is not processed by cargo build or cargo run
        // Self::from_query_register(datasource_name).await;
    }

    /// The operation that checks if an entity must be update is name in the database
    fn create_or_rename_tables<'a>(
        &mut self,
        canyon_memory: &'_ CanyonMemory,
        entity_name: &'a str,
        entity_fields: Vec<CanyonRegisterEntityField>,
        database_tables: &'a [&'a TableMetadata],
    ) {
        // 1st operation -> Check if the current entity is already on the target database.
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
                    entity_name.to_string(), // Set the new table name
                )
            } else {
                self.create_table(entity_name.to_string(), entity_fields)
            }
        }
    }

    /// Generates a database agnostic query to change the name of a table
    fn create_table(&mut self, table_name: String, entity_fields: Vec<CanyonRegisterEntityField>) {
        self.operations.push(Box::new(TableOperation::CreateTable(
            table_name,
            entity_fields,
        )));
    }

    /// Generates a database agnostic query to change the name of a table
    fn table_rename(&mut self, old_table_name: String, new_table_name: String) {
        self.operations
            .push(Box::new(TableOperation::AlterTableName(
                old_table_name,
                new_table_name,
            )));
    }

    // Creates or modify (currently only datatype) a column for a given canyon register entity field
    fn delete_fields<'a>(
        &mut self,
        entity_name: &'a str,
        entity_fields: Vec<CanyonRegisterEntityField>,
        current_table_metadata: Option<&'a TableMetadata>,
        db_type: DatabaseType,
    ) {
        if current_table_metadata.is_none() {
            return;
        }
        let columns_name_to_delete: Vec<&ColumnMetadata> = current_table_metadata
            .unwrap()
            .columns
            .iter()
            .filter(|db_column| {
                entity_fields
                    .iter()
                    .map(|canyon_field| canyon_field.field_name.to_string())
                    .any(|canyon_field| canyon_field == db_column.column_name)
                    .not()
            })
            .collect();

        for column_metadata in columns_name_to_delete {
            if db_type == DatabaseType::SqlServer && !column_metadata.is_nullable {
                self.drop_column_not_null(
                    entity_name,
                    column_metadata.column_name.clone(),
                    MigrationsHelper::get_datatype_from_column_metadata(column_metadata),
                )
            }
            self.delete_column(entity_name, column_metadata.column_name.clone());
        }
    }

    // Creates or modify (currently only datatype and nullability) a column for a given canyon register entity field
    fn create_or_modify_field(
        &mut self,
        entity_name: &str,
        db_type: DatabaseType,
        canyon_register_entity_field: CanyonRegisterEntityField,
        current_column_metadata: Option<&ColumnMetadata>,
    ) {
        // If we do not retrieve data for this database column, it does not exist yet
        // and therefore it has to be created
        if current_column_metadata.is_none() {
            self.create_column(
                entity_name.to_string(),
                canyon_register_entity_field.clone(),
            )
        } else if !MigrationsHelper::is_same_datatype(
            db_type,
            &canyon_register_entity_field,
            current_column_metadata.unwrap(),
        ) {
            self.change_column_datatype(
                entity_name.to_string(),
                canyon_register_entity_field.clone(),
            )
        }

        if let Some(column_metadata) = current_column_metadata {
            if canyon_register_entity_field.is_nullable() != column_metadata.is_nullable {
                if column_metadata.is_nullable {
                    self.set_not_null(entity_name.to_string(), canyon_register_entity_field)
                } else {
                    self.drop_not_null(entity_name.to_string(), canyon_register_entity_field)
                }
            }
        }
    }

    fn delete_column(&mut self, table_name: &str, column_name: String) {
        self.operations.push(Box::new(ColumnOperation::DeleteColumn(
            table_name.to_string(),
            column_name,
        )));
    }

    fn drop_column_not_null(
        &mut self,
        table_name: &str,
        column_name: String,
        column_datatype: String,
    ) {
        self.operations
            .push(Box::new(ColumnOperation::DropNotNullBeforeDropColumn(
                table_name.to_string(),
                column_name,
                column_datatype,
            )));
    }

    fn create_column(&mut self, table_name: String, field: CanyonRegisterEntityField) {
        self.operations
            .push(Box::new(ColumnOperation::CreateColumn(table_name, field)));
    }

    fn change_column_datatype(&mut self, table_name: String, field: CanyonRegisterEntityField) {
        self.operations
            .push(Box::new(ColumnOperation::AlterColumnType(
                table_name, field,
            )));
    }

    fn set_not_null(&mut self, table_name: String, field: CanyonRegisterEntityField) {
        self.operations
            .push(Box::new(ColumnOperation::AlterColumnSetNotNull(
                table_name, field,
            )));
    }

    fn drop_not_null(&mut self, table_name: String, field: CanyonRegisterEntityField) {
        self.operations
            .push(Box::new(ColumnOperation::AlterColumnDropNotNull(
                table_name, field,
            )));
    }

    fn add_constraints(
        &mut self,
        entity_name: &str,
        canyon_register_entity_field: CanyonRegisterEntityField,
    ) {
        for attr in &canyon_register_entity_field.annotations {
            if attr.starts_with("Annotation: ForeignKey") {
                let annotation_data = MigrationsHelper::extract_foreign_key_annotation(
                    &canyon_register_entity_field.annotations,
                );

                let table_to_reference = annotation_data.0;
                let column_to_reference = annotation_data.1;

                let foreign_key_name = format!(
                    "{entity_name}_{}_fkey",
                    &canyon_register_entity_field.field_name
                );

                Self::add_foreign_key(
                    self,
                    entity_name,
                    foreign_key_name,
                    table_to_reference,
                    column_to_reference,
                    &canyon_register_entity_field,
                );
            }
            if attr.starts_with("Annotation: PrimaryKey") {
                Self::add_primary_key(self, entity_name, canyon_register_entity_field.clone());

                if canyon_register_entity_field.is_autoincremental() {
                    Self::add_identity(self, entity_name, canyon_register_entity_field.clone());
                }
            }
        }
    }

    fn add_foreign_key(
        &mut self,
        entity_name: &'_ str,
        foreign_key_name: String,
        table_to_reference: String,
        column_to_reference: String,
        canyon_register_entity_field: &CanyonRegisterEntityField,
    ) {
        self.constraints_operations
            .push(Box::new(TableOperation::AddTableForeignKey(
                entity_name.to_string(),
                foreign_key_name,
                canyon_register_entity_field.field_name.clone(),
                table_to_reference,
                column_to_reference,
            )));
    }

    fn add_primary_key(
        &mut self,
        entity_name: &str,
        canyon_register_entity_field: CanyonRegisterEntityField,
    ) {
        self.set_primary_key_operations
            .push(Box::new(TableOperation::AddTablePrimaryKey(
                entity_name.to_string(),
                canyon_register_entity_field,
            )));
    }

    fn add_identity(&mut self, entity_name: &str, field: CanyonRegisterEntityField) {
        self.constraints_operations
            .push(Box::new(ColumnOperation::AlterColumnAddIdentity(
                entity_name.to_string(),
                field.clone(),
            )));

        self.constraints_operations
            .push(Box::new(SequenceOperation::ModifySequence(
                entity_name.to_string(),
                field,
            )));
    }

    fn add_modify_or_remove_constraints(
        &mut self,
        entity_name: &str,
        canyon_register_entity_field: CanyonRegisterEntityField,
        current_column_metadata: &ColumnMetadata,
    ) {
        let field_is_primary_key = canyon_register_entity_field
            .annotations
            .iter()
            .any(|anno| anno.starts_with("Annotation: PrimaryKey"));

        let field_is_foreign_key = canyon_register_entity_field
            .annotations
            .iter()
            .any(|anno| anno.starts_with("Annotation: ForeignKey"));

        // ------------ PRIMARY KEY ---------------
        // Case when field contains a primary key annotation, and it's not already on database, add it to constrains_operations
        if field_is_primary_key && current_column_metadata.primary_key_info.is_none() {
            Self::add_primary_key(self, entity_name, canyon_register_entity_field.clone());

            if canyon_register_entity_field.is_autoincremental() {
                Self::add_identity(self, entity_name, canyon_register_entity_field.clone());
            }
        }
        // Case when the field contains a primary key annotation, and it's already on the database
        else if field_is_primary_key && current_column_metadata.primary_key_info.is_some() {
            let is_autoincr_rust = canyon_register_entity_field.is_autoincremental();
            let is_autoincr_in_db = current_column_metadata.is_identity;

            if !is_autoincr_rust && is_autoincr_in_db {
                Self::drop_identity(self, entity_name, canyon_register_entity_field.clone())
            } else if is_autoincr_rust && !is_autoincr_in_db {
                Self::add_identity(self, entity_name, canyon_register_entity_field.clone())
            }
        }
        // Case when field doesn't contains a primary key annotation, but there is one in the database column
        else if !field_is_primary_key && current_column_metadata.primary_key_info.is_some() {
            Self::drop_primary_key(
                self,
                entity_name,
                current_column_metadata
                    .primary_key_name
                    .as_ref()
                    .expect("PrimaryKey constrain name not found")
                    .to_string(),
            );

            if current_column_metadata.is_identity {
                Self::drop_identity(self, entity_name, canyon_register_entity_field.clone());
            }
        }

        // -------------------- FOREIGN KEY CASE ----------------------------
        // Case when field contains a foreign key annotation, and it's not already on database, add it to constraints_operations
        if field_is_foreign_key && current_column_metadata.foreign_key_name.is_none() {
            if current_column_metadata.foreign_key_name.is_none() {
                let annotation_data = MigrationsHelper::extract_foreign_key_annotation(
                    &canyon_register_entity_field.annotations,
                );

                let foreign_key_name = format!(
                    "{entity_name}_{}_fkey",
                    &canyon_register_entity_field.field_name
                );

                Self::add_foreign_key(
                    self,
                    entity_name,
                    foreign_key_name,
                    annotation_data.0,
                    annotation_data.1,
                    &canyon_register_entity_field,
                );
            }
        }
        // Case when field contains a foreign key annotation, and there is already one in the database
        else if field_is_foreign_key && current_column_metadata.foreign_key_name.is_some() {
            // Will contain the table name (on index 0) and column name (on index 1) pointed to by the foreign key
            let annotation_data = MigrationsHelper::extract_foreign_key_annotation(
                &canyon_register_entity_field.annotations,
            );

            let foreign_key_name = format!(
                "{entity_name}_{}_fkey",
                &canyon_register_entity_field.field_name
            );

            // Example of information in foreign_key_info: FOREIGN KEY (league) REFERENCES leagues(id)
            let references_regex = Regex::new(regex_patterns::EXTRACT_FOREIGN_KEY_INFO).unwrap();

            let captures_references = references_regex
                .captures(
                    current_column_metadata
                        .foreign_key_info
                        .as_ref()
                        .expect("Regex - foreign key info"),
                )
                .expect("Regex - foreign key info not found");

            let current_column = captures_references
                .name("current_column")
                .expect("Regex - Current column not found")
                .as_str()
                .to_string();
            let ref_table = captures_references
                .name("ref_table")
                .expect("Regex - Ref tablenot found")
                .as_str()
                .to_string();
            let ref_column = captures_references
                .name("ref_column")
                .expect("Regex - Ref column not found")
                .as_str()
                .to_string();

            // If entity foreign key is not equal to the one on database, a constrains_operations is added to delete it and add a new one.
            if canyon_register_entity_field.field_name != current_column
                || annotation_data.0 != ref_table
                || annotation_data.1 != ref_column
            {
                Self::delete_foreign_key(
                    self,
                    entity_name,
                    current_column_metadata
                        .foreign_key_name
                        .as_ref()
                        .expect("Annotation foreign key constrain name not found")
                        .to_string(),
                );

                Self::add_foreign_key(
                    self,
                    entity_name,
                    foreign_key_name,
                    annotation_data.0,
                    annotation_data.1,
                    &canyon_register_entity_field,
                )
            }
        } else if !field_is_foreign_key && current_column_metadata.foreign_key_name.is_some() {
            // Case when field don't contains a foreign key annotation, but there is already one in the database column
            Self::delete_foreign_key(
                self,
                entity_name,
                current_column_metadata
                    .foreign_key_name
                    .as_ref()
                    .expect("ForeignKey constrain name not found")
                    .to_string(),
            );
        }
    }

    fn drop_primary_key(&mut self, entity_name: &str, primary_key_name: String) {
        self.drop_primary_key_operations
            .push(Box::new(TableOperation::DeleteTablePrimaryKey(
                entity_name.to_string(),
                primary_key_name,
            )));
    }

    fn drop_identity(
        &mut self,
        entity_name: &str,
        canyon_register_entity_field: CanyonRegisterEntityField,
    ) {
        self.constraints_operations
            .push(Box::new(ColumnOperation::AlterColumnDropIdentity(
                entity_name.to_string(),
                canyon_register_entity_field,
            )));
    }

    fn delete_foreign_key(&mut self, entity_name: &str, constrain_name: String) {
        self.constraints_operations
            .push(Box::new(TableOperation::DeleteTableForeignKey(
                // table_with_foreign_key,constrain_name
                entity_name.to_string(),
                constrain_name,
            )));
    }

    /// Make the detected migrations for the next Canyon-SQL run
    #[allow(clippy::await_holding_lock)]
    pub async fn from_query_register(queries_to_execute: &HashMap<&str, Vec<&str>>) {
        for datasource in queries_to_execute.iter() {
            for query_to_execute in datasource.1 {
                let res = Self::query(query_to_execute, [], datasource.0).await;

                match res {
                    Ok(_) => println!(
                        "\t[OK] - {:?} - Query: {:?}",
                        datasource.0, &query_to_execute
                    ),
                    Err(e) => println!(
                        "\t[ERR] - {:?} - Query: {:?}\nCause: {:?}",
                        datasource.0, &query_to_execute, e
                    ),
                }
                // TODO Ask for user input?
            }
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
        database_tables
            .iter()
            .any(|db_table_data| db_table_data.table_name == entity_name)
    }
    /// Get the table metadata for a given entity name or his old entity name if the table was renamed.
    fn get_current_table_metadata<'a>(
        canyon_memory: &'_ CanyonMemory,
        entity_name: &'a str,
        database_tables: &'a [&'_ TableMetadata],
    ) -> Option<&'a TableMetadata> {
        let correct_entity_name = canyon_memory
            .renamed_entities
            .get(&entity_name.to_lowercase())
            .map(|e| e.to_owned())
            .unwrap_or_else(|| entity_name.to_string());

        database_tables
            .iter()
            .find(|table_metadata| {
                table_metadata.table_name.to_lowercase() == *correct_entity_name.to_lowercase()
            })
            .map(|e| e.to_owned())
    }

    /// Get the column metadata for a given column name
    fn get_current_column_metadata(
        column_name: String,
        current_table_metadata: Option<&TableMetadata>,
    ) -> Option<&ColumnMetadata> {
        if let Some(metadata_table) = current_table_metadata {
            metadata_table
                .columns
                .iter()
                .find(|column| column.column_name == column_name)
        } else {
            None
        }
    }

    fn get_datatype_from_column_metadata(current_column_metadata: &ColumnMetadata) -> String {
        // TODO Add all SQL Server text datatypes
        if vec!["nvarchar", "varchar"]
            .contains(&current_column_metadata.datatype.to_lowercase().as_str())
        {
            let varchar_len = match &current_column_metadata.character_maximum_length {
                Some(v) => v.to_string(),
                None => "max".to_string(),
            };

            format!("{}({})", current_column_metadata.datatype, varchar_len)
        } else {
            current_column_metadata.datatype.to_string()
        }
    }

    fn is_same_datatype(
        db_type: DatabaseType,
        canyon_register_entity_field: &CanyonRegisterEntityField,
        current_column_metadata: &ColumnMetadata,
    ) -> bool {
        if db_type == DatabaseType::PostgreSql {
            canyon_register_entity_field
                .to_postgres_alter_syntax()
                .to_lowercase()
                == current_column_metadata.datatype
        } else if db_type == DatabaseType::SqlServer {
            // TODO Search a better way to get the datatype without useless info (like "VARCHAR(MAX)")
            canyon_register_entity_field
                .to_sqlserver_alter_syntax()
                .to_lowercase()
                == current_column_metadata.datatype
        } else {
            todo!()
        }
    }

    fn extract_foreign_key_annotation(field_annotations: &[String]) -> (String, String) {
        let opt_fk_annotation = field_annotations
            .iter()
            .find(|anno| anno.starts_with("Annotation: ForeignKey"));
        if let Some(fk_annotation) = opt_fk_annotation {
            let annotation_data = fk_annotation
                .split(',')
                .filter(|x| !x.contains("Annotation: ForeignKey")) // After here, we only have the "table" and the "column" attribute values
                .map(|x| {
                    x.split(':')
                        .collect::<Vec<&str>>()
                        .get(1)
                        .expect("Error. Unable to split annotations")
                        .trim()
                        .to_string()
                })
                .collect::<Vec<String>>();

            let table_to_reference = annotation_data
                .get(0)
                .expect("Error extracting table ref from FK annotation")
                .to_string();
            let column_to_reference = annotation_data
                .get(1)
                .expect("Error extracting column ref from FK annotation")
                .to_string();

            (table_to_reference, column_to_reference)
        } else {
            panic!("Detected a Foreign Key attribute when does not exists on the user's code");
        }
    }
}

#[cfg(test)]
mod migrations_helper_tests {
    use super::*;
    use crate::constants;

    const MOCKED_ENTITY_NAME: &str = "league";

    #[test]
    fn test_entity_already_on_database() {
        let parse_result_empty_db_tables =
            MigrationsHelper::entity_already_on_database(MOCKED_ENTITY_NAME, &[]);
        // Always should be false
        assert!(!parse_result_empty_db_tables);

        // Rust has a League entity. Database has a `league` entity. Case should be normalized
        // and a match must raise
        let mocked_league_entity_on_database = MigrationsHelper::entity_already_on_database(
            MOCKED_ENTITY_NAME,
            &[&constants::mocked_data::TABLE_METADATA_LEAGUE_EX],
        );
        assert!(mocked_league_entity_on_database);

        let mocked_league_entity_on_database = MigrationsHelper::entity_already_on_database(
            MOCKED_ENTITY_NAME,
            &[&constants::mocked_data::NON_MATCHING_TABLE_METADATA],
        );
        assert!(!mocked_league_entity_on_database)
    }
}

/// Trait that enables implementors to generate the migration queries
#[async_trait]
trait DatabaseOperation: Debug {
    async fn generate_sql(&self, datasource: &DatasourceConfig);
}

/// Helper to relate the operations that Canyon should do when it's managing a schema
#[derive(Debug)]
#[allow(dead_code)]
enum TableOperation {
    CreateTable(String, Vec<CanyonRegisterEntityField>),
    // old table_name, new table_name
    AlterTableName(String, String),
    // table_name, foreign_key_name, column_foreign_key, table_to_reference, column_to_reference
    AddTableForeignKey(String, String, String, String, String),
    // table_with_foreign_key, constraint_name
    DeleteTableForeignKey(String, String),
    // table_name, entity_field, column_name
    AddTablePrimaryKey(String, CanyonRegisterEntityField),
    // table_name, constraint_name
    DeleteTablePrimaryKey(String, String),
}

impl<T: Debug> Transaction<T> for TableOperation {}

#[async_trait]
impl DatabaseOperation for TableOperation {
    async fn generate_sql(&self, datasource: &DatasourceConfig) {
        let db_type = datasource.get_db_type();

        let stmt = match self {
            TableOperation::CreateTable(table_name, table_fields) => {
                if db_type == DatabaseType::PostgreSql {
                    format!(
                        "CREATE TABLE \"{table_name}\" ({});",
                        table_fields
                            .iter()
                            .map(|entity_field| format!(
                                "\"{}\" {}",
                                entity_field.field_name,
                                entity_field.to_postgres_syntax()
                            ))
                            .collect::<Vec<String>>()
                            .join(", ")
                    )
                } else if db_type == DatabaseType::SqlServer {
                    format!(
                        "CREATE TABLE {:?} ({:?});",
                        table_name,
                        table_fields
                            .iter()
                            .map(|entity_field| format!(
                                "{} {}",
                                entity_field.field_name,
                                entity_field.to_sqlserver_syntax()
                            ))
                            .collect::<Vec<String>>()
                            .join(", ")
                    )
                    .replace('"', "")
                } else {
                    todo!("There's no other databases supported in Canyon-SQL right now")
                }
            }

            TableOperation::AlterTableName(old_table_name, new_table_name) => {
                if db_type == DatabaseType::PostgreSql {
                    format!("ALTER TABLE {old_table_name} RENAME TO {new_table_name};")
                } else if db_type == DatabaseType::SqlServer {
                    /*
                        Notes: Brackets around `old_table_name`, p.e.
                            exec sp_rename ['league'], 'leagues'  // NOT VALID!
                        is only allowed for compound names split by a dot.
                            exec sp_rename ['random.league'], 'leagues'  // OK

                        CARE! This doesn't mean that we are including the schema.
                            exec sp_rename ['dbo.random.league'], 'leagues' // OK
                            exec sp_rename 'dbo.league', 'leagues' // OK - Schema doesn't need brackets

                        Due to the automatic mapped name from Rust to DB and vice-versa, this won't
                        be an allowed behaviour for now, only with the table_name parameter on the
                        CanyonEntity annotation.
                    */
                    format!("exec sp_rename '{old_table_name}', '{new_table_name}';")
                } else {
                    todo!()
                }
            }

            TableOperation::AddTableForeignKey(
                table_name,
                foreign_key_name,
                column_foreign_key,
                table_to_reference,
                column_to_reference,
            ) => {
                if db_type == DatabaseType::PostgreSql {
                    format!(
                        "ALTER TABLE {table_name} ADD CONSTRAINT {foreign_key_name} \
                        FOREIGN KEY ({column_foreign_key}) REFERENCES {table_to_reference} ({column_to_reference});"
                    )
                } else if db_type == DatabaseType::SqlServer {
                    todo!("[MS-SQL -> Operation still won't supported by Canyon for Sql Server]")
                } else {
                    todo!()
                }
            }

            TableOperation::DeleteTableForeignKey(table_with_foreign_key, constraint_name) => {
                if db_type == DatabaseType::PostgreSql {
                    format!(
                        "ALTER TABLE {table_with_foreign_key} DROP CONSTRAINT {constraint_name};",
                    )
                } else if db_type == DatabaseType::SqlServer {
                    todo!("[MS-SQL -> Operation still won't supported by Canyon for Sql Server]")
                } else {
                    todo!()
                }
            }

            TableOperation::AddTablePrimaryKey(table_name, entity_field) => {
                if db_type == DatabaseType::PostgreSql {
                    format!(
                        "ALTER TABLE \"{table_name}\" ADD PRIMARY KEY (\"{}\");",
                        entity_field.field_name
                    )
                } else if db_type == DatabaseType::SqlServer {
                    todo!("[MS-SQL -> Operation still won't supported by Canyon for Sql Server]")
                } else {
                    todo!()
                }
            }

            TableOperation::DeleteTablePrimaryKey(table_name, primary_key_name) => {
                if db_type == DatabaseType::PostgreSql || db_type == DatabaseType::SqlServer {
                    format!("ALTER TABLE {table_name} DROP CONSTRAINT {primary_key_name} CASCADE;")
                } else {
                    todo!()
                }
            }
        };

        save_migrations_query_to_execute(stmt, &datasource.name);
    }
}

/// Helper to relate the operations that Canyon should do when a change on a field should
#[derive(Debug)]
#[allow(dead_code)]
enum ColumnOperation {
    CreateColumn(String, CanyonRegisterEntityField),
    DeleteColumn(String, String),
    // AlterColumnName,
    AlterColumnType(String, CanyonRegisterEntityField),
    AlterColumnDropNotNull(String, CanyonRegisterEntityField),
    // SQL server specific operation - SQL server can't drop a NOT NULL column
    DropNotNullBeforeDropColumn(String, String, String),
    AlterColumnSetNotNull(String, CanyonRegisterEntityField),
    // TODO if implement through annotations, modify for both GENERATED {ALWAYS, BY DEFAULT}
    AlterColumnAddIdentity(String, CanyonRegisterEntityField),
    AlterColumnDropIdentity(String, CanyonRegisterEntityField),
}

impl Transaction<Self> for ColumnOperation {}

#[async_trait]
impl DatabaseOperation for ColumnOperation {
    async fn generate_sql(&self, datasource: &DatasourceConfig) {
        let db_type = datasource.get_db_type();

        let stmt = match self {
            ColumnOperation::CreateColumn(table_name, entity_field) =>
            if db_type == DatabaseType::PostgreSql {
                format!(
                "ALTER TABLE \"{}\" ADD COLUMN \"{}\" {};",
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
                // TODO Check if operation for SQL server is different
                format!("ALTER TABLE \"{table_name}\" DROP COLUMN \"{column_name}\";")
            },
            ColumnOperation::AlterColumnType(table_name, entity_field) =>
            if db_type == DatabaseType::PostgreSql {
                format!(
                    "ALTER TABLE \"{table_name}\" ALTER COLUMN \"{}\" TYPE {};",
                    entity_field.field_name, entity_field.to_postgres_alter_syntax()
                )
            }  else if db_type == DatabaseType::SqlServer {
                todo!("[MS-SQL -> Operation still won't supported by Canyon for Sql Server]")
            } else {
                todo!()
            }
            ,
            ColumnOperation::AlterColumnDropNotNull(table_name, entity_field) =>
            if db_type == DatabaseType::PostgreSql {
                format!("ALTER TABLE \"{table_name}\" ALTER COLUMN \"{}\" DROP NOT NULL;", entity_field.field_name)
            }  else if db_type == DatabaseType::SqlServer {
                format!(
                    "ALTER TABLE \"{table_name}\" ALTER COLUMN {} {} NULL",
                    entity_field.field_name, entity_field.to_sqlserver_alter_syntax()
                )
            } else {
                todo!()
            }

            ColumnOperation::DropNotNullBeforeDropColumn(table_name, column_name, column_datatype) =>
                format!(
                "ALTER TABLE {table_name} ALTER COLUMN {column_name} {column_datatype} NULL; DECLARE @tableName VARCHAR(MAX) = '{table_name}'
                DECLARE @columnName VARCHAR(MAX) = '{column_name}'
                DECLARE @ConstraintName nvarchar(200)
                SELECT @ConstraintName = Name
                FROM SYS.DEFAULT_CONSTRAINTS
                WHERE PARENT_OBJECT_ID = OBJECT_ID(@tableName)
                AND PARENT_COLUMN_ID = (
                    SELECT column_id FROM sys.columns
                    WHERE NAME = @columnName AND object_id = OBJECT_ID(@tableName))
                IF @ConstraintName IS NOT NULL
                    EXEC('ALTER TABLE '+@tableName+' DROP CONSTRAINT ' + @ConstraintName);"
            ),

            ColumnOperation::AlterColumnSetNotNull(table_name, entity_field) => format!(
                "ALTER TABLE \"{table_name}\" ALTER COLUMN \"{}\" SET NOT NULL;", entity_field.field_name
            ),

            ColumnOperation::AlterColumnAddIdentity(table_name, entity_field) => format!(
                "ALTER TABLE \"{table_name}\" ALTER COLUMN \"{}\" ADD GENERATED ALWAYS AS IDENTITY;", entity_field.field_name
            ),

            ColumnOperation::AlterColumnDropIdentity(table_name, entity_field) => format!(
                "ALTER TABLE \"{table_name}\" ALTER COLUMN \"{}\" DROP IDENTITY;", entity_field.field_name
            ),
        };

        save_migrations_query_to_execute(stmt, &datasource.name);
    }
}

/// Helper for operations involving sequences
#[derive(Debug)]
#[allow(dead_code)]
enum SequenceOperation {
    ModifySequence(String, CanyonRegisterEntityField),
}

impl Transaction<Self> for SequenceOperation {}

#[async_trait]
impl DatabaseOperation for SequenceOperation {
    async fn generate_sql(&self, datasource: &DatasourceConfig) {
        let db_type = datasource.get_db_type();

        let stmt = match self {
            SequenceOperation::ModifySequence(table_name, entity_field) => {
                if db_type == DatabaseType::PostgreSql {
                    format!(
                    "SELECT setval(pg_get_serial_sequence('\"{table_name}\"', '{}'), max(\"{}\")) from \"{table_name}\";",
                    entity_field.field_name, entity_field.field_name
                )
                } else if db_type == DatabaseType::SqlServer {
                    todo!("[MS-SQL -> Operation still won't supported by Canyon for Sql Server]")
                } else {
                    todo!()
                }
            }
        };

        save_migrations_query_to_execute(stmt, &datasource.name);
    }
}
