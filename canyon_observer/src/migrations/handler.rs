use canyon_connection::{datasources::Migrations as MigrationsStatus, DATASOURCES};
use partialdebug::placeholder::PartialDebug;
use canyon_crud::rows::CanyonRows;

use crate::{
    canyon_crud::{
        bounds::{Column, Row, RowOperations},
        crud::Transaction,
        DatabaseType,
    },
    constants,
    migrations::{
        information_schema::{ColumnMetadata, ColumnMetadataTypeValue, TableMetadata},
        memory::CanyonMemory,
        processor::MigrationsProcessor,
    },
    CANYON_REGISTER_ENTITIES,
};

#[derive(PartialDebug)]
pub struct Migrations;
// Makes this structure able to make queries to the database
impl Transaction<Self> for Migrations {}

impl Migrations {
    /// Launches the mechanism to parse the Database schema, the Canyon register
    /// and the database table with the memory of Canyon to perform the
    /// migrations over the targeted database
    pub async fn migrate() {
        for datasource in DATASOURCES.iter() {
            if datasource
                .properties
                .migrations
                .filter(|status| !status.eq(&MigrationsStatus::Disabled))
                .is_none()
            {
                println!(
                    "Skipped datasource: {:?} for being disabled (or not configured)",
                    datasource.name
                );
                continue;
            }
            println!(
                "Processing migrations for datasource: {:?}",
                datasource.name
            );

            let mut migrations_processor = MigrationsProcessor::default();

            let canyon_entities = CANYON_REGISTER_ENTITIES.lock().unwrap().to_vec();
            let canyon_memory = CanyonMemory::remember(datasource, &canyon_entities).await;

            // Tracked entities that must be migrated whenever Canyon starts
            let schema_status =
                Self::fetch_database(&datasource.name, datasource.get_db_type()).await;
            let database_tables_schema_info = Self::map_rows(schema_status, datasource.get_db_type());

            // We filter the tables from the schema that aren't Canyon entities
            let mut user_database_tables = vec![];
            for parsed_table in database_tables_schema_info.iter() {
                if canyon_memory
                    .memory
                    .iter()
                    .any(|f| f.declared_table_name.eq(&parsed_table.table_name))
                    || canyon_memory
                        .renamed_entities
                        .values()
                        .any(|f| *f == parsed_table.table_name)
                {
                    user_database_tables.append(&mut vec![parsed_table]);
                }
            }

            migrations_processor
                .process(
                    canyon_memory,
                    canyon_entities,
                    user_database_tables,
                    datasource,
                )
                .await;
        }
    }

    /// Fetches a concrete schema metadata by target the database
    /// chosen by it's datasource name property
    async fn fetch_database(
        datasource_name: &str,
        db_type: DatabaseType,
    ) -> CanyonRows<Migrations> {
        let query = match db_type {
            #[cfg(feature = "tokio-postgres")] DatabaseType::PostgreSql => constants::postgresql_queries::FETCH_PUBLIC_SCHEMA,
            #[cfg(feature = "tiberius")] DatabaseType::SqlServer => constants::mssql_queries::FETCH_PUBLIC_SCHEMA,
        };

        Self::query(query, [], datasource_name).await
            .unwrap_or_else(|_| {panic!(
                "Error querying the schema information for the datasource: {datasource_name}"
            )})
    }

    /// Handler for parse the result of query the information of some database schema,
    /// and extract the content of the returned rows into custom structures with
    /// the data well organized for every entity present on that schema
    fn map_rows(db_results: CanyonRows<Migrations>, db_type: DatabaseType) -> Vec<TableMetadata> {
        let mut schema_info: Vec<TableMetadata> = Vec::new();

        for res_row in db_results.into_iter()
            // .map(|row| &row as &dyn Row)
        {
            let unique_table = schema_info
                .iter_mut()
                // TODO To be able to remove row from our code, use a match statement to get table name
                .find(|table| check_for_table_name(table, &res_row as &dyn Row));
            match unique_table {
                Some(table) => {
                    /* If a table entity it's already present on the collection, we add it
                    the founded columns related to the table */
                    Self::get_columns_metadata(&res_row as &dyn Row, table);
                }
                None => {
                    /* If there's no table for a given "table_name" property on the
                    collection yet, we must create a new instance and attach it
                    the founded columns data in this iteration */
                    let mut new_table = TableMetadata {
                        table_name: match db_type {
                            #[cfg(feature = "tokio-postgres")] DatabaseType::PostgreSql => get_table_name_from_tp_row(&res_row),
                            #[cfg(feature = "tiberius")] DatabaseType::SqlServer => get_table_name_from_tib_row(&res_row),
                        },
                        columns: Vec::new(),
                    };
                    Self::get_columns_metadata(&res_row as &dyn Row, &mut new_table);
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
    fn set_column_metadata(row: &dyn Row, src: &Column, dest: &mut ColumnMetadata) {
        let column_identifier = src.name();
        let column_value = ColumnMetadataTypeValue::get_value(row, src);

        if column_identifier == "column_name" {
            if let ColumnMetadataTypeValue::StringValue(value) = &column_value {
                dest.column_name = value
                    .to_owned()
                    .expect("[MIGRATIONS - set_column_metadata -> column_name]")
            }
        } else if column_identifier == "data_type" {
            if let ColumnMetadataTypeValue::StringValue(value) = &column_value {
                dest.datatype = value
                    .to_owned()
                    .expect("[MIGRATIONS - set_column_metadata -> data_type]")
            }
        } else if column_identifier == "character_maximum_length" {
            if let ColumnMetadataTypeValue::IntValue(value) = &column_value {
                dest.character_maximum_length = value.to_owned()
            }
        } else if column_identifier == "is_nullable" {
            if let ColumnMetadataTypeValue::StringValue(value) = &column_value {
                dest.is_nullable = matches!(
                    value
                        .as_ref()
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
                    value
                        .as_ref()
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


#[cfg(feature = "tokio-postgres")]
fn get_table_name_from_tp_row(res_row: &tokio_postgres::Row) -> String {
    res_row.get::<&str, String>("table_name")
}
#[cfg(feature = "tiberius")]
fn get_table_name_from_tib_row(res_row: &tiberius::Row) -> String {
    res_row.get::<&str, &str>("table_name").unwrap_or_default().to_string()
}

fn check_for_table_name(table: &&mut TableMetadata, res_row: &dyn Row) -> bool {
    #[cfg(feature = "tokio-postgres")] {
        table.table_name == res_row.get_postgres::<&str>("table_name")
    }
    #[cfg(feature = "tiberius")] {
        table.table_name == row_retriever_fn_ptr(&res_row, "table_name")
    }
}
