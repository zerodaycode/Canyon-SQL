/// Provides the necessary entities to let Canyon perform and develop
/// it's full potential, completly managing all the entities written by
/// the user and annotated with the `#[canyon_entity]`

use std::collections::HashMap;
use tokio_postgres::{types::Type, Row};
use partialdebug::placeholder::PartialDebug;
use regex::Regex;

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
    pub async fn new() -> CanyonHandler<'a> {
        let a = Self::get_entities_from_string();
        let b = Self::fetch_database_status().await;
        println!("Entities: {:?}", &a);
        println!("Tables: {:?}", &b);
        Self {
            canyon_tables: a,
            database_tables: b,
        }
    }

    /// Queries the Canyon Register and manages to retrieve the String that contains all entities
    /// as str, and convert its back to a Vec<CanyonEntity>
    fn get_entities_from_string() -> Vec<CanyonRegisterEntity> {
        let mut entities: Vec<CanyonRegisterEntity> = Vec::new();
        let regex_for_register = Regex::new(r"\[(.*?)\]")
            .unwrap();
        let reg_entities = regex_for_register
            .find_iter(unsafe {&CANYON_REGISTER.join(",") })
            .map( |entity| entity.as_str().to_string() )
            .collect::<Vec<String>>();

        for element in reg_entities {
            let mut new_entity = CanyonRegisterEntity::new();
            let entity_data = element
                .replace('[', "")
                .replace(']', "")
                .replace('"', "")
                .split(";")
                .map( |element| element.to_string())
                .collect::<Vec<String>>();
            
            for (i, element) in entity_data.iter().enumerate() {
                let value = element
                    .split("-> ")
                    .nth(1)
                    .unwrap_or("");
                if i == 0 {
                    new_entity.entity_name = value.to_string();
                } else {
                    for entity_field in value.split(",") {
                        let mut new_entity_field: CanyonRegisterEntityField = CanyonRegisterEntityField::new();
                        let splited = entity_field
                            .replace('(', "")
                            .replace(')', "")
                            .replace(' ', "")
                            .split(":")
                            .map(|x|x.to_string())
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
            match schema_info.iter_mut().find( |table| {
                table.table_name == res_row.get::<&str, String>("table_name")
            }) {
                unique_table => {
                    match unique_table {
                        Some(table) => {
                            /* If a table entity it's already present on the collection, we add it
                                the founded columns related to the table */
                            Self::get_row_postgre_columns_for_table(&res_row, table);
                        }
                        None => {
                            /* If there's no table for a given "table_name" property on the 
                                collection yet, we must create a new instance and attach it 
                                the founded columns data in this iteration */
                            let mut new_table = RowTable { 
                                table_name: res_row.get::<&str, String>("table_name"),
                                columns: Vec::new(),
                            };
                            Self::get_row_postgre_columns_for_table(&res_row, &mut new_table);
                            schema_info.push(new_table);
                        }
                    }
                }
            }
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
            match database_tables.iter_mut().find( |table: &&mut DatabaseTable| {
                table.table_name == mapped_table.table_name
            }) {
                unique_database_table => {
                    match unique_database_table {
                        Some(table) => {
                            Self::map_splitted_column_info_into_entity(
                                mapped_table, table
                            )
                        },
                        None => {
                            let mut new_unique_database_table = DatabaseTable {
                                table_name: mapped_table.table_name.clone(),
                                columns: Vec::new()
                            };
                            Self::map_splitted_column_info_into_entity(
                                mapped_table, &mut new_unique_database_table
                            );
                            database_tables.push(new_unique_database_table);
                        }
                    }
                }
            }
        }
        for db_table in &database_tables {
            println!("\nDatabase table: {:?}", &db_table.table_name);
            for table_column in &db_table.columns {
                println!("Column: {:?}, Postgres type: {:?}",
                    &table_column.column_name, &table_column.postgres_datatype
                );
            }
        }
        database_tables
    }

    /// Gets the N rows that contais the info about a concrete table column and maps
    /// them into a single entity
    fn map_splitted_column_info_into_entity(mapped_table: &RowTable,
                                            table_entity: &mut DatabaseTable) {

        let mut entity_column = DatabaseTableColumn::new();
        for (idx, column) in mapped_table.columns.iter().enumerate() {
            match &column.column_identifier {
                column_identifier => {
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
                            entity_column.is_nullable = match &value.as_ref().unwrap().as_str() {
                                &"YES" => true,
                                _ => false
                            }
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
                    }
                }
            }
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

            if postgre_column.name().to_string() != "table_name" { // Discards the column "table_name"
                let mut new_column = RelatedColumn {
                    column_identifier: postgre_column.name().to_string(), 
                    datatype: postgre_column.type_().to_string(), 
                    value: ColumnTypeValue::NoneValue
                };
    
                match *postgre_column.type_() {
                    Type::NAME | Type::VARCHAR => {
                        new_column.value = ColumnTypeValue::StringValue(
                            res_row.get::<&str, Option<String>>(postgre_column.name())
                        );
                    },
                    Type::INT4 => {
                        new_column.value = ColumnTypeValue::IntValue(    
                            res_row.get::<&str, Option<i32>>(postgre_column.name())
                        );
                    },
                    _ => new_column.value = ColumnTypeValue::NoneValue
                }
                table.columns.push(new_column)
            }
        }
    }
}

/// Stores the data for manage the database status after matching the entities on the register
struct DatabaseSyncOperation<T> {
    pub table: String,
    pub target: String,
    pub operation: T
}

/// Helper to relate the operations that Canyon should do when it's managing a schema
enum TableOperation {
    CreateTable,
    AlterTableName,
    AlterColumn(Vec<ColumnOperation>)
}

/// Helper to relate the operations that Canyon should do when a change on a field should
enum ColumnOperation {
    CreateColumn,
    DeleteColumn,
    AlterColumnName,
    AlterColumnType
}


/// TODO Docs
#[derive(Debug)]
pub struct CanyonRegisterEntity {
    pub entity_name: String,
    pub entity_fields: Vec<CanyonRegisterEntityField>
}
impl CanyonRegisterEntity{
    pub fn new() -> CanyonRegisterEntity {
        Self {
            entity_name: String::new(),
            entity_fields:Vec::new()
        }
    }
}


#[derive(Debug)]
pub struct CanyonRegisterEntityField {
    pub field_name: String,
    pub field_type: String
}
impl CanyonRegisterEntityField{
    pub fn new() -> CanyonRegisterEntityField {
        Self {
            field_name: String::new(),
            field_type: String::new()
        }
    }
}

/// A struct to manages the retrieved rows with the schema info for the
/// current user's selected database
#[derive(Debug, Clone)]
pub struct DatabaseDataRows {
    pub table_name: String,
    pub columns_types: HashMap<String, String>
}

/* Models that represents the database entities that belongs to the current schema */
#[derive(Debug)]
pub struct DatabaseTable<'a> {
    pub table_name: String,
    pub columns: Vec<DatabaseTableColumn<'a>>
}
#[derive(Debug, Clone)]
pub struct DatabaseTableColumn<'a> {
    pub column_name: String,
    pub postgres_datatype: String,
    pub character_maximum_length: Option<i32>,
    pub is_nullable: bool,  // Care, postgres type is varchar
    pub column_default: Option<String>,
    pub numeric_precision: Option<i32>,
    pub numeric_scale: Option<i32>,
    pub numeric_precision_radix: Option<i32>,
    pub datetime_precision: Option<i32>,
    pub interval_type: Option<String>,
    pub phantom: &'a str  // TODO
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
            phantom: ""
        }
    }
}


/* POSTGRESQL entities for map the multiple rows that are related to one table, and the multiple
    columns that are nested related to those row table */

#[derive(Debug)]
pub struct RowTable {
    pub table_name: String,
    pub columns: Vec<RelatedColumn>
} 
#[derive(Debug)]
pub struct RelatedColumn {
    pub column_identifier: String,
    pub datatype: String,
    pub value: ColumnTypeValue
}

#[derive(Debug)]
pub enum ColumnTypeValue {
    StringValue(Option<String>),
    IntValue(Option<i32>),
    NoneValue
}