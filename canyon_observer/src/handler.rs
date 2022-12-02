use canyon_connection::get_database_type_from_datasource_name;
use partialdebug::placeholder::PartialDebug;

use canyon_crud::{crud::Transaction, bounds::{Row, RowOperations, Column}, DatabaseType, result::DatabaseResult};

use crate::{
    CANYON_REGISTER_ENTITIES,
    constants, 
    migrations::{
        processor:: MigrationsProcessor,
        information_schema::{
            TableMetadata,
            ColumnMetadata,
            ColumnMetadataTypeValue
        }   
    }, memory::CanyonMemory
};

#[derive(PartialDebug)]
pub struct Migrations;
// Makes this structure able to make queries to the database
impl Transaction<Self> for Migrations {}

impl Migrations {
    /// Launches the mechanism to parse the Database schema, the Canyon register
    /// and the database table with the memory of Canyon to perform the
    /// migrations over the targeted database
    pub async fn migrate(datasource_name: &str) {
        let mut migrations_processor = MigrationsProcessor::default();

        let canyon_memory = CanyonMemory::remember(datasource_name).await;
        let canyon_tables = CANYON_REGISTER_ENTITIES.lock().unwrap().to_vec();

        // Tracked entities that must be migrated whenever Canyon starts
        let db_type = get_database_type_from_datasource_name(datasource_name).await;
        let schema_status = Self::fetch_database(datasource_name, db_type).await;
        let database_tables_schema_info = Self::map_rows(schema_status);
        
        // We filter the tables from the schema that aren't Canyon entities
        let mut user_database_tables = vec![];
        for table_parsed in database_tables_schema_info.iter() {
            if canyon_memory.memory.values().into_iter().any(|f| f.to_lowercase() == table_parsed.table_name) 
                || canyon_memory.renamed_entities.contains_key(&table_parsed.table_name.to_lowercase())
            {
                user_database_tables.append(&mut vec![table_parsed]);
            }
        }

        migrations_processor.process(
            canyon_memory, canyon_tables, user_database_tables, datasource_name
        ).await;
    }

    /// Fetches a concrete schema metadata by target the database
    /// choosed by it's datasource name property
    async fn fetch_database(datasource_name: &str, db_type: DatabaseType) -> DatabaseResult<Migrations> {
        let query = match db_type {
            DatabaseType::PostgreSql => constants::postgresql_queries::FETCH_PUBLIC_SCHEMA, 
            DatabaseType::SqlServer => constants::mssql_queries::FETCH_PUBLIC_SCHEMA
        };

        Self::query(query, &[], datasource_name)
            .await
            .expect(
                &format!("Error querying the schema information for the datasource: {}", datasource_name)
            )
    }

    /// Handler for parse the result of query the information of some database schema,
    /// and extract the content of the returned rows into custom structures with
    /// the data well organized for every entity present on that schema
    fn map_rows<'a>(db_results: DatabaseResult<Migrations>) -> Vec<TableMetadata> {
        let mut schema_info: Vec<TableMetadata> = Vec::new();

        for res_row in db_results.as_canyon_rows().into_iter() {
            let unique_table = schema_info
                .iter_mut()
                .find(|table| table.table_name == res_row.get::<&str>("table_name").to_owned());
            match unique_table {
                Some(table) => {
                    /* If a table entity it's already present on the collection, we add it
                    the founded columns related to the table */
                    Self::get_columns_metadata(res_row, table);
                }
                None => {
                    /* If there's no table for a given "table_name" property on the
                    collection yet, we must create a new instance and attach it
                    the founded columns data in this iteration */
                    let mut new_table = TableMetadata {
                        table_name: res_row.get::<&str>("table_name").to_owned(),
                        columns: Vec::new(),
                    };
                    Self::get_columns_metadata(res_row, &mut new_table);
                    schema_info.push(new_table);
                }
            };
        }

        schema_info
    }

    /// Parses all the [`Row`] after query the information of the targeted schema,
    /// grouping them in [`TableMetadata`] structs, by relating every [`Row`] that has
    /// the same "table_name" (asked with column.name()) being one field of the new
    /// [`TableMetadata`], and parsing the other columns that belongs to that entity
    /// and appending as a new [`ColumnMetadata`] element to the columns field.
    fn get_columns_metadata(res_row: &dyn Row, table: &mut TableMetadata) {
        let mut entity_column = ColumnMetadata::default();
        for column in res_row.columns().iter() {
            if column.name() != "table_name" {
                Self::set_column_metadata(res_row, column, &mut entity_column);
            } // Discards the column "table_name", 'cause is already a field of [`TableMetadata`]
        }
        table.columns.push(entity_column);
    }

    /// Sets the concrete value for a field of a [`ColumnMetadata`], by reading the properties
    /// of the source [`Column`], filtering the target value by the source property `column name`
    fn set_column_metadata<'a>(row: &dyn Row, src: &Column, dest: &mut ColumnMetadata) {
        let column_identifier = src.name();
        let column_value = ColumnMetadataTypeValue::get_value(row, src);

        if column_identifier == "column_name" {
            if let ColumnMetadataTypeValue::StringValue(value) = &column_value {
                dest.column_name = value.to_owned().expect("[MIGRATIONS - set_column_metadata -> column_name]")
            }
        } else if column_identifier == "data_type" {
            if let ColumnMetadataTypeValue::StringValue(value) = &column_value {
                dest.postgres_datatype = value.to_owned().expect("[MIGRATIONS - set_column_metadata -> data_type]")
            }
        } else if column_identifier == "character_maximum_length" {
            if let ColumnMetadataTypeValue::IntValue(value) = &column_value {
                dest.character_maximum_length = value.to_owned()
            }
        } else if column_identifier == "is_nullable" {
            if let ColumnMetadataTypeValue::StringValue(value) = &column_value {
                dest.is_nullable = matches!(
                    value.as_ref()
                        .expect("[MIGRATIONS - set_column_metadata -> is_nullable]")
                        .as_str(), 
                    "YES"
                )
            }
        } else if column_identifier == "column_default" {
            if let ColumnMetadataTypeValue::StringValue(value) = &column_value {
                dest.column_default = value.to_owned()
            }
        } else if column_identifier == "foreign_key_info" {
            if let ColumnMetadataTypeValue::StringValue(value) = &column_value {
                dest.foreign_key_info = value.to_owned()
            }
        } else if column_identifier == "foreign_key_name" {
            if let ColumnMetadataTypeValue::StringValue(value) = &column_value {
                dest.foreign_key_name = value.to_owned()
            }
        } else if column_identifier == "primary_key_info" {
            if let ColumnMetadataTypeValue::StringValue(value) = &column_value {
                dest.primary_key_info = value.to_owned()
            }
        } else if column_identifier == "primary_key_name" {
            if let ColumnMetadataTypeValue::StringValue(value) = &column_value {
                dest.primary_key_name = value.to_owned()
            }
        } else if column_identifier == "is_identity" {
            if let ColumnMetadataTypeValue::StringValue(value) = &column_value {
                dest.is_identity = matches!(
                    value.as_ref()
                        .expect("[MIGRATIONS - set_column_metadata -> is_identity]")
                        .as_str(),
                    "YES"
                )
            }
        } else if column_identifier == "identity_generation" {
            if let ColumnMetadataTypeValue::StringValue(value) = &column_value {
                dest.identity_generation = value.to_owned()
            }
        };
    }
}
