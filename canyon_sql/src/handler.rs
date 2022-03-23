/// Provides the necessary entities to let Canyon perform and develop
/// it's full potential, completly managing all the entities written by
/// the user and annotated with the `#[canyon_entity]`

use std::collections::HashMap;
use tokio_postgres::{types::Type, Row};
use partialdebug::placeholder::PartialDebug;
use regex::Regex;
use std::fmt::Debug;

use std::ops::Not;

use crate::crud::Transaction;

use canyon_observer::CANYON_REGISTER;

#[derive(PartialDebug)]
pub struct CanyonHandler<'a> {
    pub canyon_tables: Vec<CanyonRegisterEntity>,
    pub database_tables: Vec<DatabaseTable<'a>>
}
// Makes this structure able to make queries to the database
impl<'a> Transaction<Self> for CanyonHandler<'a> {}

impl<'a> CanyonHandler<'a> {
    pub async fn run() {
        let a = Self::get_entities_from_string();
        let b = Self::fetch_database_status().await;
        // println!("Entities: {:?}", &a);
        // println!("Tables: {:?}", &b);
        let self_ = Self {
            canyon_tables: a,
            database_tables: b
        };
        let mut db_operation = DatabaseSyncOperations::new();
        db_operation.fill_operations(self_).await;
    }

    /// Queries the Canyon Register and manages to retrieve the String that contains all entities
    /// as str, and convert its back to a Vec<CanyonEntity>
    fn get_entities_from_string() -> Vec<CanyonRegisterEntity> {
        let mut entities: Vec<CanyonRegisterEntity> = Vec::new();
        let regex_for_register = Regex::new(r"\[(.*?)\]")
            .unwrap();
        let reg_entities = regex_for_register
            .find_iter(unsafe { &CANYON_REGISTER.join(",") })
            .map(|entity| entity.as_str().to_string())
            .collect::<Vec<String>>();

        // let solotest: Vec<String> = vec![
        //     "[\"Identifier -> players\"; \"Columns -> (id:int4), (first_name:text), (age:int4)\"]".to_string(),
        //     "[\"Identifier -> tournaments\"; \"Columns -> (league:int4),(id:int8),(ext_id:int4),(slug:text),(image_url:text)\"]".to_string(),
        //     "[\"Identifier -> test\"; \"Columns -> (id:int4), (test_col_text:text), (test_col_int:int4)\"]".to_string()
        // ];

        // let reg_entities = regex_for_register
        //     .find_iter(&solotest.join(","))
        //     .map(|entity| entity.as_str().to_string())
        //     .collect::<Vec<String>>();

        for element in reg_entities {
            let mut new_entity = CanyonRegisterEntity::new();
            let entity_data = element
                .replace('[', "")
                .replace(']', "")
                .replace('"', "")
                .split(';')
                .map(|element| element.to_string())
                .collect::<Vec<String>>();


            for (i, element) in entity_data.iter().enumerate() {
                let value = element
                    .split("-> ")
                    .nth(1)
                    .unwrap_or("");
                if i == 0 {
                    new_entity.entity_name = value.to_string();
                } else {
                    for entity_field in value.split(',') {
                        let mut new_entity_field: CanyonRegisterEntityField = CanyonRegisterEntityField::new();
                        let splited = entity_field
                            .replace('(', "")
                            .replace(')', "")
                            .replace(' ', "")
                            .split(':')
                            .map(|x| x.to_string())
                            .collect::<Vec<String>>();

                        new_entity_field.field_name = splited.get(0).unwrap_or(&String::new()).to_owned();
                        new_entity_field.field_type = splited.get(1).unwrap_or(&String::new()).to_owned();

                        new_entity.entity_fields.push(new_entity_field);
                    }
                }
            }

            entities.push(new_entity);
        }
        // println!("Our entities {:?}",entities);
        entities
    }

    async fn fetch_database_status() -> Vec<DatabaseTable<'a>> {
        let query_request =
            "SELECT table_name, column_name, data_type,
                character_maximum_length,is_nullable,column_default,numeric_precision,
                numeric_scale,numeric_precision_radix,datetime_precision,interval_type
            FROM information_schema.columns WHERE table_schema = 'public';";

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
        println!("FINISHED TABLES TOTAL => {}", &schema_info.len());
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
        // for db_table in &database_tables {
        //     println!("\nDatabase table: {:?}", &db_table.table_name);
        //     for table_column in &db_table.columns {
        //         println!("Column: {:?}, Postgres type: {:?}",
        //             &table_column.column_name, &table_column.postgres_datatype
        //         );
        //     }
        // }
        database_tables
    }

    /// Gets the N rows that contais the info about a concrete table column and maps
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
            };
            // Just for split the related column data into what will be the values for
            // every DatabaseTableColumn.
            // Every times that we find an &RelatedColumn which column identifier
            // is == "interval_type", we know that we finished to set the values
            // for a new DatabaseTableColumn
            if &column.column_identifier == "interval_type" {
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
                    Type::NAME | Type::VARCHAR => {
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

struct DatabaseSyncOperations {
    operations: Vec<Box<dyn DatabaseOperations>>,
}

impl DatabaseSyncOperations {
    pub fn new() -> Self {
        Self {
            operations: Vec::new()
        }
    }

    pub async fn fill_operations(&mut self, data: CanyonHandler<'_>) {
        // For each entity (table) on the register
        for canyon_register_entity in data.canyon_tables {
            println!("Current loop table \"{}\":", canyon_register_entity.entity_name.to_owned().to_lowercase());

            // true if this table on the register is already on the database, else false
            let table_on_database = data.database_tables.iter()
                .any(|v| v.table_name == canyon_register_entity.entity_name.to_lowercase());

            println!("      Table \"{}\" already on DB ? => {}", canyon_register_entity.entity_name.to_lowercase(), table_on_database);

            // If the table isn't on the database we push a new operation to the vector
            if table_on_database.not() {
                let database_operation = DatabaseOperation::new(
                    canyon_register_entity.entity_name.to_owned().to_lowercase(),
                    TableOperation::CreateTable(canyon_register_entity.entity_name.to_owned().to_lowercase())).await;

                self.operations.push(Box::new(database_operation));
            } else {
                // We check if each of the columns in this table of the register is in the database table.
                // We get the names and add them to a vector of strings
                let columns_in_table: Vec<String> = canyon_register_entity.entity_fields.iter()
                    .filter(|a| data.database_tables.iter()
                        .find(|x| x.table_name == canyon_register_entity.entity_name.to_lowercase()).unwrap().columns
                        .iter()
                        .map(|x| x.column_name.to_string())
                        .any(|x| x == a.field_name))
                    .map(|a| a.field_name.to_string()).collect();

                println!("      Columns already in table : {:?}", columns_in_table);

                // For each field (name,type) in this table of the register
                for field in &canyon_register_entity.entity_fields {

                    // Case when the column doesnt exist on the database
                    // We push a new operation to the vector for each one
                    if columns_in_table.contains(&field.field_name).not() {
                        let database_operation = DatabaseOperation::new(
                            canyon_register_entity.entity_name.to_owned().to_lowercase(),
                            TableOperation::AlterColumn(ColumnOperation::CreateColumn(
                                field.field_name.to_owned(), field.field_type.to_owned()))).await;

                        self.operations.push(Box::new(database_operation));
                    }
                    // Case when the column exist on the database
                    else {
                        println!("          Checking datatypes for field \"{}\"",field.field_name);
                        let database_field = &data.database_tables.iter()
                            .find(|x| x.table_name == canyon_register_entity.entity_name.to_lowercase())
                            .unwrap().columns
                            .iter().find(|x| x.column_name == field.field_name).unwrap();

                        let mut database_field_postgres_type: String = String::new();
                        println!("          Pre-convertion DB column datatype = {}",database_field.postgres_datatype);
                        match database_field.postgres_datatype.as_str() {
                            "integer" => {
                                database_field_postgres_type = "i32".to_string();
                            }
                            "bigint" => {
                                database_field_postgres_type = "i64".to_string();
                            }
                            "text" | "character varying" => {
                                database_field_postgres_type = "String".to_string();
                            }
                            _ => {
                                database_field_postgres_type = "".to_string();
                            }

                        }

                        if database_field.is_nullable && field.field_type.contains("Option") {
                            database_field_postgres_type = format!("Option<{}>",database_field_postgres_type);
                        }
                        println!("          Post-convertion field datatype = {} | DB column datatype = {}",field.field_type,database_field_postgres_type);
                        if field.field_type != database_field_postgres_type {
                            let database_operation = DatabaseOperation::new(
                                canyon_register_entity.entity_name.to_owned().to_lowercase(),
                                TableOperation::AlterColumn(ColumnOperation::AlterColumnType(
                                    field.field_name.to_owned(), field.field_type.to_owned()))).await;

                            self.operations.push(Box::new(database_operation));
                        }
                    }
                }

                // Filter the list of columns in the corresponding table of the database for the current table of the register,
                // and look for columns that don't exist in the table of the register
                let columns_to_remove: Vec<String> = data.database_tables.iter()
                    .find(|x| x.table_name == canyon_register_entity.entity_name.to_lowercase()).unwrap().columns
                    .iter()
                    .filter(|a| canyon_register_entity.entity_fields.iter()
                        .map(|x| x.field_name.to_string())
                        .collect::<String>().contains(&a.column_name).not())
                    .map(|a| a.column_name.to_string()).collect();

                println!("      Columns only in table (to remove):  {:?}", columns_to_remove);
                // If we have columns to remove, we push a new operation to the vector for each one
                if columns_to_remove.is_empty().not() {
                    for column in &columns_to_remove {
                        let database_operation = DatabaseOperation::new(
                            canyon_register_entity.entity_name.to_owned().to_lowercase(),
                            TableOperation::AlterColumn(ColumnOperation::DeleteColumn(column.to_owned()))).await;
                        self.operations.push(Box::new(database_operation));
                    }
                }
            }
        }

        println!("Operations to do on database: {:?}", &self.operations);
    }
}

/// TODO Docs
trait DatabaseOperations: Debug { }

/// Stores the data for manage the database status after matching the entities on the register
#[derive(Debug)]
struct DatabaseOperation<T: Debug + ?Sized> {
    table: String,
    operation: T,
}

impl<T: Debug + ?Sized> DatabaseOperations for DatabaseOperation<T> {}


impl<T: Debug> DatabaseOperation<T> {
    pub async fn new(table_name: String, operation: T) -> DatabaseOperation<T> {
        Self {
            table: table_name,
            operation: operation,
        }
    }
}

/// Helper to relate the operations that Canyon should do when it's managing a schema
#[derive(Debug)]
enum TableOperation {
    CreateTable(String),
    // AlterTableName,
    AlterColumn(ColumnOperation),
}

/// Helper to relate the operations that Canyon should do when a change on a field should
#[derive(Debug)]
enum ColumnOperation {
    CreateColumn(String, String),
    DeleteColumn(String),
    // AlterColumnName,
    AlterColumnType(String, String),
}


/// TODO Docs
#[derive(Debug)]
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
}


#[derive(Debug)]
pub struct CanyonRegisterEntityField {
    pub field_name: String,
    pub field_type: String,
}

impl CanyonRegisterEntityField {
    pub fn new() -> CanyonRegisterEntityField {
        Self {
            field_name: String::new(),
            field_type: String::new(),
        }
    }
}

/// A struct to manages the retrieved rows with the schema info for the
/// current user's selected database
#[derive(Debug, Clone)]
pub struct DatabaseDataRows {
    pub table_name: String,
    pub columns_types: HashMap<String, String>,
}

/* Models that represents the database entities that belongs to the current schema */
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