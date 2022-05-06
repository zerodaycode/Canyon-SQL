/// Provides the necessary entities to let Canyon perform and develop
/// it's full potential, completly managing all the entities written by
/// the user and annotated with the `#[canyon_entity]`

use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Not;
use async_trait::async_trait;
use tokio_postgres::{types::Type, Row};
use partialdebug::placeholder::PartialDebug;
use regex::Regex;

use canyon_crud::crud::Transaction;

use super::{
    CANYON_REGISTER_ENTITIES, 
    QUERIES_TO_EXECUTE,
    memory::CanyonMemory
};

#[derive(PartialDebug)]
pub struct CanyonHandler<'a> {
    pub canyon_memory: CanyonMemory,
    pub canyon_tables: Vec<CanyonRegisterEntity>,
    pub database_tables: Vec<DatabaseTable<'a>>,
}
// Makes this structure able to make queries to the database
impl<'a> Transaction<Self> for CanyonHandler<'a> {}

impl<'a> CanyonHandler<'a> {
    pub async fn run() {
        let a = Self::get_info_of_entities();
        let b = Self::fetch_database_status().await;

        let self_ = Self {
            canyon_memory: CanyonMemory::remember().await,
            canyon_tables: a,
            database_tables: b,
        };
        let mut db_operation = DatabaseSyncOperations::new();
        db_operation.fill_operations(self_).await;
    }

    // Converts a CanyonEntity into a CanyonRegisterEntity, where the second just contains
    // raw string info with the identifiers (names) for tables, columns, column types...
    fn get_info_of_entities() -> Vec<CanyonRegisterEntity> {
        let mut entities: Vec<CanyonRegisterEntity> = Vec::new();
        let clone = unsafe { CANYON_REGISTER_ENTITIES.clone() };
        for i in clone.iter() {
            let mut new_entity = CanyonRegisterEntity::new();
            new_entity.entity_name = i.entity_name.clone();

            for field in i.entity_fields.iter() {
                let mut new_entity_field = CanyonRegisterEntityField::new();
                new_entity_field.field_name = field.field_name.clone();
                new_entity_field.field_type = field.field_type.clone();
                new_entity_field.annotation = field.annotation.clone();
                new_entity.entity_fields.push(new_entity_field);
            }
            entities.push(new_entity);
        }
        entities
    }

    async fn fetch_database_status() -> Vec<DatabaseTable<'a>> {
        let query_request = "select
                                    gi.table_name,
                                    gi.column_name,
                                    gi.data_type,
                                    gi.character_maximum_length,
                                    gi.is_nullable,
                                    gi.column_default,
                                    gi.numeric_precision,
                                    gi.numeric_scale,
                                    gi.numeric_precision_radix,
                                    gi.datetime_precision,
                                    gi.interval_type,
                                    CAST(pg_catalog.pg_get_constraintdef(oid) AS TEXT) as foreign_key_info,
                                    fk.conname as foreign_key_name
                                from
                                    information_schema.columns as gi
                                left join pg_catalog.pg_constraint as fk on
                                    gi.table_name = CAST(fk.conrelid::regclass AS TEXT) and
                                    gi.column_name = split_part(split_part(CAST(pg_catalog.pg_get_constraintdef(oid) AS TEXT),')',1),'(',2) and fk.contype = 'f'
                                where
                                    table_schema = 'public';";

        let results = Self::query(query_request, &[]).await.wrapper;

        let mut schema_info: Vec<RowTable> = Vec::new();

        for res_row in results.iter() {
            let unique_table = schema_info.iter_mut().find(|table| {
                table.table_name == res_row.get::<&str, String>("table_name")
            });
            match unique_table {
                Some(table) => {
                    /* If a table entity it's already present on the collection, we add it
                        the founded columns related to the table */
                    Self::get_row_postgre_columns_for_table(res_row, table);
                }
                None => {
                    /* If there's no table for a given "table_name" property on the
                        collection yet, we must create a new instance and attach it
                        the founded columns data in this iteration */
                    let mut new_table = RowTable {
                        table_name: res_row.get::<&str, String>("table_name"),
                        columns: Vec::new(),
                    };
                    Self::get_row_postgre_columns_for_table(res_row, &mut new_table);
                    schema_info.push(new_table);
                }
            };
        }
        Self::generate_mapped_table_entities(schema_info)
    }

    /// Given the entities that binds the rows results for the schema info, converts
    /// them into a collection of "Table" entities that represents the a real database
    /// table
    fn generate_mapped_table_entities(schema_info: Vec<RowTable>) -> Vec<DatabaseTable<'a>> {
        let mut database_tables = Vec::new();

        for mapped_table in &schema_info {
            let unique_database_table = database_tables.iter_mut().find(|table: &&mut DatabaseTable| {
                table.table_name == mapped_table.table_name
            });
            match unique_database_table {
                Some(table) => {
                    Self::map_splitted_column_info_into_entity(
                        mapped_table, table,
                    )
                }
                None => {
                    let mut new_unique_database_table = DatabaseTable {
                        table_name: mapped_table.table_name.clone(),
                        columns: Vec::new(),
                    };
                    Self::map_splitted_column_info_into_entity(
                        mapped_table, &mut new_unique_database_table,
                    );
                    database_tables.push(new_unique_database_table);
                }
            };
        }

        database_tables
    }

    /// Gets the N rows that contains the info about a concrete table column and maps
    /// them into a single entity
    fn map_splitted_column_info_into_entity(mapped_table: &RowTable,
                                            table_entity: &mut DatabaseTable) {
        let mut entity_column = DatabaseTableColumn::new();
        for (idx, column) in mapped_table.columns.iter().enumerate() {
            let column_identifier = &column.column_identifier;
            if column_identifier == "column_name" {
                if let ColumnTypeValue::StringValue(value) = &column.value {
                    entity_column.column_name = value.to_owned().unwrap()
                }
            } else if column_identifier == "data_type" {
                if let ColumnTypeValue::StringValue(value) = &column.value {
                    entity_column.postgres_datatype = value.to_owned().unwrap()
                }
            } else if column_identifier == "character_maximum_length" {
                if let ColumnTypeValue::IntValue(value) = &column.value {
                    entity_column.character_maximum_length = value.to_owned()
                }
            } else if column_identifier == "is_nullable" {
                if let ColumnTypeValue::StringValue(value) = &column.value {
                    entity_column.is_nullable = matches!(value.as_ref().unwrap().as_str(), "YES")
                }
            } else if column_identifier == "column_default" {
                if let ColumnTypeValue::StringValue(value) = &column.value {
                    entity_column.column_default = value.to_owned()
                }
            } else if column_identifier == "numeric_precision" {
                if let ColumnTypeValue::IntValue(value) = &column.value {
                    entity_column.numeric_precision = value.to_owned()
                }
            } else if column_identifier == "numeric_scale" {
                if let ColumnTypeValue::IntValue(value) = &column.value {
                    entity_column.numeric_scale = value.to_owned()
                }
            } else if column_identifier == "numeric_precision_radix" {
                if let ColumnTypeValue::IntValue(value) = &column.value {
                    entity_column.numeric_precision_radix = value.to_owned()
                }
            } else if column_identifier == "datetime_precision" {
                if let ColumnTypeValue::IntValue(value) = &column.value {
                    entity_column.datetime_precision = value.to_owned()
                }
            } else if column_identifier == "interval_type" {
                if let ColumnTypeValue::StringValue(value) = &column.value {
                    entity_column.interval_type = value.to_owned()
                }
            } else if column_identifier == "foreign_key_info" {
                if let ColumnTypeValue::StringValue(value) = &column.value {
                    entity_column.foreign_key_info = value.to_owned()
                }
            } else if column_identifier == "foreign_key_name" {
                if let ColumnTypeValue::StringValue(value) = &column.value {
                    entity_column.foreign_key_name = value.to_owned()
                }
            };
            // Just for split the related column data into what will be the values for
            // every DatabaseTableColumn.
            // Every times that we find an &RelatedColumn which column identifier
            // is == "foreign_key_name", we know that we finished to set the values
            // for a new DatabaseTableColumn
            if &column.column_identifier == "foreign_key_name" {
                table_entity.columns.push(entity_column.clone());
                if idx == mapped_table.columns.len() - 1 {
                    entity_column = DatabaseTableColumn::new();
                }
            }
        }
    }

    /// Retrieves for every row founded related to one table record,
    /// the data and values associated to that row.
    /// So, for every row, here we have rows containing values related to one table, but
    /// are columns that, at the end of the function, just represents the data stored in one
    /// row of results.
    fn get_row_postgre_columns_for_table(res_row: &Row, table: &mut RowTable) {
        for postgre_column in res_row.columns().iter() {
            if postgre_column.name() != "table_name" { // Discards the column "table_name"
                let mut new_column = RelatedColumn {
                    column_identifier: postgre_column.name().to_string(),
                    datatype: postgre_column.type_().to_string(),
                    value: ColumnTypeValue::NoneValue,
                };

                match *postgre_column.type_() {
                    Type::NAME | Type::VARCHAR | Type::TEXT => {
                        new_column.value = ColumnTypeValue::StringValue(
                            res_row.get::<&str, Option<String>>(postgre_column.name())
                        );
                    }
                    Type::INT4 => {
                        new_column.value = ColumnTypeValue::IntValue(
                            res_row.get::<&str, Option<i32>>(postgre_column.name())
                        );
                    }
                    _ => new_column.value = ColumnTypeValue::NoneValue
                }
                table.columns.push(new_column)
            }
        }
    }
}

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
                                            // // table_with_foreign_key,constrain_name
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

        for  database_table in data.database_tables {
            
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
        for i in 0..unsafe { &QUERIES_TO_EXECUTE }.len() - 1 {
            Self::query(unsafe { QUERIES_TO_EXECUTE.get(i).unwrap() }, &[]).await;
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

        unsafe { QUERIES_TO_EXECUTE.push(stmt) }
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

        unsafe { QUERIES_TO_EXECUTE.push(stmt) }
    }
}


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
    /// let my_id_field =  CanyonRegisterEntityField {
    ///                         field_name: "id".to_string(),
    ///                         field_type: "i32".to_string(),
    ///                         annotation: None
    ///                      };
    ///
    /// let my_name_field =  CanyonRegisterEntityField {
    ///                          field_name: "name".to_string(),
    ///                          field_type: "String".to_string(),
    ///                          annotation: None
    ///                      };
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

/// A struct to manages the retrieved rows with the schema info for the
/// current user's selected database
#[derive(Debug, Clone)]
pub struct DatabaseDataRows {
    pub table_name: String,
    pub columns_types: HashMap<String, String>,
}

/// Models that represents the database entities that belongs to the current schema
#[derive(Debug)]
pub struct DatabaseTable<'a> {
    pub table_name: String,
    pub columns: Vec<DatabaseTableColumn<'a>>,
}

#[derive(Debug, Clone)]
pub struct DatabaseTableColumn<'a> {
    pub column_name: String,
    pub postgres_datatype: String,
    pub character_maximum_length: Option<i32>,
    pub is_nullable: bool,
    // Care, postgres type is varchar
    pub column_default: Option<String>,
    pub numeric_precision: Option<i32>,
    pub numeric_scale: Option<i32>,
    pub numeric_precision_radix: Option<i32>,
    pub datetime_precision: Option<i32>,
    pub interval_type: Option<String>,
    pub foreign_key_info: Option<String>,
    pub foreign_key_name: Option<String>,
    pub phantom: &'a str,  // TODO
}

impl<'a> DatabaseTableColumn<'a> {
    pub fn new() -> DatabaseTableColumn<'a> {
        Self {
            column_name: String::new(),
            postgres_datatype: String::new(),
            character_maximum_length: None,
            is_nullable: true,
            column_default: None,
            numeric_precision: None,
            numeric_scale: None,
            numeric_precision_radix: None,
            datetime_precision: None,
            interval_type: None,
            foreign_key_info: None,
            foreign_key_name: None,
            phantom: "",
        }
    }
}


/* POSTGRESQL entities for map the multiple rows that are related to one table, and the multiple
    columns that are nested related to those row table */

#[derive(Debug)]
pub struct RowTable {
    pub table_name: String,
    pub columns: Vec<RelatedColumn>,
}

#[derive(Debug)]
pub struct RelatedColumn {
    pub column_identifier: String,
    pub datatype: String,
    pub value: ColumnTypeValue,
}

#[derive(Debug)]
pub enum ColumnTypeValue {
    StringValue(Option<String>),
    IntValue(Option<i32>),
    NoneValue,
}