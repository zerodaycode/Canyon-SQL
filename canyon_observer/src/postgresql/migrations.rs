/// File that contains all the datatypes and logic to perform the migrations
/// over a `PostgreSQL` database

use std::{ops::Not, sync::MutexGuard};
use std::fmt::{Debug, Display};
use async_trait::async_trait;

use canyon_crud::crud::Transaction;
use regex::Regex;
use crate::memory::CanyonMemory;
use crate::QUERIES_TO_EXECUTE;
use crate::postgresql::information_schema::rows_to_table_mapper::DatabaseTable;

use super::register_types::{CanyonRegisterEntityField, CanyonRegisterEntity};


/// Responsible of generating the queries to sync the database status with the
/// Rust source code managed by Canyon, for succesfully make the migrations
#[derive(Debug)]
pub struct DatabaseSyncOperations {
    operations: Vec<Box<dyn DatabaseOperation>>,
    drop_primary_key_operations: Vec<Box<dyn DatabaseOperation>>,
    set_primary_key_operations: Vec<Box<dyn DatabaseOperation>>,
    constrains_operations: Vec<Box<dyn DatabaseOperation>>
}

impl Transaction<Self> for DatabaseSyncOperations {}

impl DatabaseSyncOperations {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
            drop_primary_key_operations: Vec::new(),
            set_primary_key_operations: Vec::new(),
            constrains_operations: Vec::new()
        }
    }

    pub async fn fill_operations<'a>(
        &mut self,
        canyon_memory: CanyonMemory,
        canyon_tables: Vec<CanyonRegisterEntity<'static>>,
        database_tables: Vec<DatabaseTable<'a>>
    ) {
        // For each entity (table) on the register
        for canyon_register_entity in canyon_tables {

            let table_name = canyon_register_entity.entity_name;

            // true if this table on the register is already on the database
            let table_on_database = Self::check_table_on_database(&table_name, &database_tables);

            // If the table isn't on the database we push a new operation to the collection,
            // either to create a new table or to rename an existing one.
            if !table_on_database {
                let table_renamed = canyon_memory.table_rename.contains_key(&*table_name);

                // canyon_memory holds a hashmap of the tables who must changed their name.
                // If this table name is present, we dont create a new one, just rename
                if table_renamed {
                    // let old_table_name = data.canyon_memory.table_rename.to_owned().get(&table_name.to_owned());
                    let otn = canyon_memory.table_rename.get(table_name).unwrap().to_owned().clone();

                    Self::push_table_rename::<String, &str>(self, otn,&table_name);

                    // TODO Change foreign_key constrain name on database
                    continue;
                }
                // If not, we push an operation to create a new one
                else {
                    Self::add_new_table::<&str>(self, table_name, canyon_register_entity.entity_fields.clone());
                }

                let cloned_fields = canyon_register_entity.entity_fields.clone();
                // We iterate over the fields/columns seeking for constrains to add
                for field in cloned_fields
                    .iter()
                    .filter( 
                        |column| column.annotations.len() > 0
                    ) {
                        field.annotations.iter()
                            .for_each( |attr|
                                {
                                    if attr.starts_with("Annotation: ForeignKey") {
                                        Self::add_foreign_key_with_annotation::<&str, &String>(
                                            self, &field.annotations, table_name, &field.field_name,
                                        );
                                    }
                                    if attr.starts_with("Annotation: PrimaryKey") {
                                        Self::add_primary_key::<&str>(
                                            self, table_name, field.clone()
                                        );

                                        Self::add_identity::<&str>(
                                            self, table_name, field.clone()
                                        );
                                    }
                                }
                            );
                    }
            } else {
                // We check if each of the columns in this table of the register is in the database table.
                // We get the names and add them to a vector of strings
                let columns_in_table = Self::columns_in_table(
                    canyon_register_entity.entity_fields.clone(),
                    &database_tables,
                    &table_name,
                );

                // For each field (name, type) in this table of the register
                for field in canyon_register_entity.entity_fields.clone() {
                    // Case when the column doesn't exist on the database
                    // We push a new column operation to the collection for each one
                    if !columns_in_table.contains(&field.field_name) {
                        Self::add_column_to_table::<&str>(self, &table_name, field.clone());

                        // We added the founded constraints on the field attributes
                        for attr in &field.annotations {
                            if attr.starts_with("Annotation: ForeignKey") {
                                Self::add_foreign_key_with_annotation::<&str, &String>(
                                    self, &field.annotations, table_name, &field.field_name,
                                );
                            }
                            if attr.starts_with("Annotation: PrimaryKey") {

                                Self::add_primary_key::<&str>(
                                    self, table_name, field.clone(),
                                );

                                Self::add_identity::<&str>(
                                    self, table_name, field.clone(),
                                );
                            }
                        }


                    }
                    // Case when the column exist on the database
                    else {

                        let d = database_tables.clone();
                        let database_table = d
                            .into_iter()
                            .find(|x| x.table_name == *table_name)
                            .unwrap();

                        let database_field = database_table.columns
                            .iter().find(|x| x.column_name == field.field_name)
                            .expect("Field annt exists");

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
                        
                        if database_field.is_nullable {
                            database_field_postgres_type = format!("Option<{}>", database_field_postgres_type);
                        }
                        
                        if field.field_type != database_field_postgres_type {
                            if field.field_type.starts_with("Option") {
                                self.constrains_operations.push(
                                    Box::new(
                                        ColumnOperation::AlterColumnDropNotNull(table_name, field.clone())
                                    )
                                );
                            } else{
                                self.constrains_operations.push(
                                    Box::new(
                                        ColumnOperation::AlterColumnSetNotNull(table_name, field.clone())
                                    )
                                );
                            }
                            Self::change_column_type(self, table_name, field.clone());
                        }


                        let field_is_primary_key = field.annotations.iter()
                            .any(|anno| anno.starts_with("Annotation: PrimaryKey"));

                        let field_is_foreign_key = field.annotations.iter()
                            .any(|anno| anno.starts_with("Annotation: ForeignKey"));
                        // TODO Checking Foreign Key attrs. Refactor to a database rust attributes matcher
                        // TODO Evaluate changing the name of the primary key if it already exists in the database

                        // -------- PRIMARY KEY CASE ----------------------------

                        // Case when field contains a primary key annotation, and it's not already on database, add it to constrains_operations
                        if field_is_primary_key && database_field.primary_key_info.is_none() {
                            Self::add_primary_key::<&str>(
                                self, table_name, field.clone(),
                            );
                            Self::add_identity::<&str>(
                                self, table_name, field.clone(),
                            );
                        }

                        // Case when field don't contains a primary key annotation, but there is already one in the database column
                        else if !field_is_primary_key && database_field.primary_key_info.is_some() {
                            Self::drop_primary_key::<String>(
                                self,
                                table_name.to_string(),
                                database_field.primary_key_name
                                    .as_ref()
                                    .expect("PrimaryKey constrain name not found")
                                    .to_string(),
                            );

                            if database_field.is_identity {
                                Self::drop_identity::<&str>(
                                            self, table_name, field.clone()
                                        );
                            }

                        }

                        // -------- FOREIGN KEY CASE ----------------------------

                        // Case when field contains a foreign key annotation, and it's not already on database, add it to constrains_operations
                        if field_is_foreign_key && database_field.foreign_key_name.is_none() {
                            if database_field.foreign_key_name.is_none() {
                                Self::add_foreign_key_with_annotation::<&str, &String>(
                                    self, &field.annotations, table_name, &field.field_name,
                                )
                            }
                        }
                        // Case when field contains a foreign key annotation, and there is already one in the database
                        else if field_is_foreign_key && database_field.foreign_key_name.is_some() {
                            // Will contain the table name (on index 0) and column name (on index 1) pointed to by the foreign key
                            let annotation_data = Self::extract_foreign_key_annotation(&field.annotations);

                            // Example of information in foreign_key_info: FOREIGN KEY (league) REFERENCES leagues(id)
                            let references_regex = Regex::new(r"\w+\s\w+\s\((?P<current_column>\w+)\)\s\w+\s(?P<ref_table>\w+)\((?P<ref_column>\w+)\)").unwrap();

                            let captures_references = references_regex.captures(database_field.foreign_key_info.as_ref().expect("Regex - foreign key info")).expect("Regex - foreign key info not found");

                            let current_column = captures_references.name("current_column").expect("Regex - Current column not found").as_str().to_string();
                            let ref_table = captures_references.name("ref_table").expect("Regex - Ref tablenot found").as_str().to_string();
                            let ref_column = captures_references.name("ref_column").expect("Regex - Ref column not found").as_str().to_string();

                            // If entity foreign key is not equal to the one on database, a constrains_operations is added to delete it and add a new one.
                            if field.field_name != current_column || annotation_data.0 != ref_table || annotation_data.1 != ref_column {
                                Self::delete_foreign_key_with_references::<String>(
                                    self,
                                    table_name.to_string(),
                                    database_field.foreign_key_name
                                        .as_ref()
                                        .expect("Annotation foreign key constrain name not found")
                                        .to_string()
                                );

                                Self::add_foreign_key_with_references(
                                    self,
                                    annotation_data.0,
                                    annotation_data.1,
                                    table_name,
                                    field.field_name.clone(),
                                )
                            }
                        }
                        // Case when field don't contains a foreign key annotation, but there is already one in the database column
                        else if !field_is_foreign_key && database_field.foreign_key_name.is_some() {
                         Self::delete_foreign_key_with_references::<String>(
                                    self,
                                    table_name.to_string(),
                                    database_field.foreign_key_name
                                        .as_ref()
                                        .expect("ForeignKey constrain name not found")
                                        .to_string()
                                );
                        }
                    }
                }

                // Filter the list of columns in the corresponding table of the database for the current table of the register,
                // and look for columns that don't exist in the table of the register
                let columns_to_remove: Vec<String> = Self::columns_to_remove(
                    &database_tables,
                    canyon_register_entity.entity_fields.clone(),
                    &table_name,
                );

                // If we have columns to remove, we push a new operation to the vector for each one
                if columns_to_remove.is_empty().not() {
                    for column in &columns_to_remove {
                        Self::delete_column_from_table(self, table_name, column.to_owned())
                    }
                }
            }
        }


        for operation in &self.operations {
            operation.execute().await
        }
        for drop_primary_key_operation in &self.drop_primary_key_operations {
            drop_primary_key_operation.execute().await
        }
        for set_primary_key_operation in &self.set_primary_key_operations {
            set_primary_key_operation.execute().await
        }
        for constrain_operation in &self.constrains_operations {
            constrain_operation.execute().await
        }
    }

    /// Make the detected migrations for the next Canyon-SQL run
    pub async fn from_query_register() {
        let queries: &MutexGuard<Vec<String>> = &QUERIES_TO_EXECUTE.lock().unwrap();

        for i in 0..queries.len() - 1 {
            let query_to_execute = queries
            .get(i)
            .expect(format!("Failed to retrieve query from the register at index: {}", i).as_str());

            Self::query(
                query_to_execute,
                vec![],
                ""
            ).await
                .ok()
                .expect(format!("Failed the migration query: {:?}", queries.get(i).unwrap()).as_str());
                // TODO Represent failable operation by logging (if configured by the user) to a text file the Result variant
                // TODO Ask for user input?
        }
    }

    fn check_table_on_database<'a>(
        table_name: &'a str, database_tables: &Vec<DatabaseTable<'_>>
    ) -> bool {
        database_tables
            .iter()
            .any(|v| &v.table_name == table_name)
    }

    fn columns_in_table(
        canyon_columns: Vec<CanyonRegisterEntityField>,
        database_tables: &[DatabaseTable<'_>],
        table_name: &str,
    ) -> Vec<String> {
        canyon_columns.iter()
            .filter(|a| database_tables.iter()
                .find( |x| x.table_name == table_name).expect("Error collecting database tables")
                .columns
                .iter()
                .map(|x| x.column_name.to_string())
                .any(|x| x == a.field_name))
            .map(|a| a.field_name.to_string()).collect()
    }

    fn columns_to_remove(
        database_tables: &[DatabaseTable<'_>],
        canyon_columns: Vec<CanyonRegisterEntityField>,
        table_name: &str,
    ) -> Vec<String> {
        database_tables.iter()
            .find(|x| x.table_name == table_name).expect("Error parsing the columns to remove")
            .columns
            .iter()
            .filter(|a| canyon_columns.iter()
                .map(|x| x.field_name.to_string())
                .any(|x| x == a.column_name).not())
            .map(|a| a.column_name.to_string()).collect()
    }


    fn push_table_rename<T, U>(&mut self, old_table_name: T, new_table_name: U) 
        where 
            T: Into<String> + Debug + Display + Sync + 'static, 
            U: Into<String> + Debug + Display + Sync + 'static 
    {
        self.operations.push(
            Box::new(
                TableOperation::AlterTableName::<_, _, &str, &str, &str>(
                    old_table_name,
                    new_table_name,
                )
            )
        );
    }

    fn add_new_table<T>(&mut self, table_name: T, columns: Vec<CanyonRegisterEntityField>) 
        where T: Into<String> + Debug + Display + Sync + 'static
    {
        self.operations.push(
            Box::new(
                TableOperation::CreateTable::<_, &str, &str, &str, &str>(
                    table_name,
                    columns,
                )
            )
        );
    }

    fn extract_foreign_key_annotation(field_annotations: &Vec<String>) -> (String, String)
    {
        let opt_fk_annotation = field_annotations.iter().
            find(|anno| anno.starts_with("Annotation: ForeignKey"));
        if let Some(fk_annotation) = opt_fk_annotation {
            let annotation_data = fk_annotation
                .split(',')
                .filter(|x| !x.contains("Annotation: ForeignKey")) // After here, we only have the "table" and the "column" attribute values
                .map(|x|
                    x.split(':').collect::<Vec<&str>>()
                        .get(1)
                        .expect("Error. Unable to split annotations")
                        .trim()
                        .to_string()
                ).collect::<Vec<String>>();

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

    fn add_foreign_key_with_annotation<'a, U, V>(
        &mut self,
        field_annotations: &'a Vec<String>,
        table_name: U,
        column_foreign_key: V,
    ) where 
        U: Into<String> + Debug + Display + Sync,
        V: Into<String> + Debug + Display + Sync  
    {

        let annotation_data = Self::extract_foreign_key_annotation(field_annotations);

        let table_to_reference = annotation_data.0;
        let column_to_reference = annotation_data.1;

        let foreign_key_name = format!("{}_{}_fkey", table_name, &column_foreign_key);

        self.constrains_operations.push(
            Box::new(
                TableOperation::AddTableForeignKey::<String, String, String, String, String>(
                    table_name.to_string(), foreign_key_name, column_foreign_key.to_string(), table_to_reference, column_to_reference,
                )
            )
        );
    }

    fn add_foreign_key_with_references<T, U, V, W>(
        &mut self,
        table_to_reference: T,
        column_to_reference: U,
        table_name: V,
        column_foreign_key: W,
    ) where
        T: Into<String> + Debug + Display + Sync + 'static,
        U: Into<String> + Debug + Display + Sync + 'static,
        V: Into<String> + Debug + Display + Sync + 'static,
        W: Into<String> + Debug + Display + Sync + 'static
    {
        let foreign_key_name = format!("{}_{}_fkey", &table_name, &column_foreign_key);


        self.constrains_operations.push(
            Box::new(
                TableOperation::AddTableForeignKey(
                    table_name, foreign_key_name, column_foreign_key, table_to_reference, column_to_reference,
                )
            )
        );
    }

    fn delete_foreign_key_with_references<T>(
        &mut self,
        table_with_foreign_key: T,
        constrain_name: T,
    ) where 
        T: Into<String> + Debug + Display + Sync + 'static
    {
        self.constrains_operations.push(
            Box::new(
                TableOperation::DeleteTableForeignKey::<T, T, T, T, T>(
                    // table_with_foreign_key,constrain_name
                    table_with_foreign_key, constrain_name,
                )
            )
        );
    }


    fn add_primary_key<T>(&mut self, table_name: T, field: CanyonRegisterEntityField)
        where T: Into<String> + Debug + Display + Sync + 'static
    {
        self.set_primary_key_operations.push(
            Box::new(
                TableOperation::AddTablePrimaryKey::<T, T, T, T, T>(
                    table_name, field
                )
            )
        );
    }

    fn drop_primary_key<T>(&mut self, table_name: T, primary_key_name: T)
        where T: Into<String> + Debug + Display + Sync + 'static
    {
        self.drop_primary_key_operations.push(
            Box::new(
                TableOperation::DeleteTablePrimaryKey::<T, T, T, T, T>(
                    table_name, primary_key_name
                )
            )
        );
    }

    fn add_identity<T>(&mut self, table_name: T, field: CanyonRegisterEntityField)
        where T: Into<String> + Debug + Display + Sync + 'static
    {


        self.constrains_operations.push(
            Box::new(
                ColumnOperation::AlterColumnAddIdentity(
                    table_name.to_string(), field.clone(),
                )
            )
        );

        self.constrains_operations.push(
            Box::new(
                SequenceOperation::ModifySequence(
                    table_name, field,
                )
            )
        );
    }

    fn drop_identity<T>(&mut self, table_name: T, field: CanyonRegisterEntityField)
        where T: Into<String> + Debug + Display + Sync + 'static
    {
        self.constrains_operations.push(
            Box::new(
                ColumnOperation::AlterColumnDropIdentity(
                    table_name, field,
                )
            )
        );
    }



    fn add_column_to_table<T>(&mut self, table_name: T, field: CanyonRegisterEntityField)
        where T: Into<String> + Debug + Display + Sync + 'static
    {
        self.operations.push(
            Box::new(
                ColumnOperation::CreateColumn(
                    table_name, field
                )
            )
        );
    }

    fn change_column_type<T>(&mut self, table_name: T, field: CanyonRegisterEntityField) 
        where T: Into<String> + Debug + Display + Sync + 'static
    {
        self.operations.push(
            Box::new(
                ColumnOperation::AlterColumnType(
                    table_name, field,
                )
            )
        );
    }

    fn delete_column_from_table<T>(&mut self, table_name: T, column: String) 
        where T: Into<String> + Debug + Display + Sync + 'static
    {
        self.operations.push(
            Box::new(
                ColumnOperation::DeleteColumn(table_name, column)
            )
        );
    }
}

/// Trait that enables implementors to execute migration queries 
#[async_trait]
trait DatabaseOperation: Debug {
    async fn execute(&self);
}

/// Helper to relate the operations that Canyon should do when it's managing a schema
#[derive(Debug)]
enum TableOperation<T, U, V, W, X> {
    CreateTable(T, Vec<CanyonRegisterEntityField>),
    // old table_name, new table_name
    AlterTableName(T, U),
    // table_name, foreign_key_name, column_foreign_key, table_to_reference, column_to_reference
    AddTableForeignKey(T, U, V, W, X),
    // table_with_foreign_key, constrain_name
    DeleteTableForeignKey(T, T),
    // table_name, entity_field, column_name
    AddTablePrimaryKey(T, CanyonRegisterEntityField),
    // table_name, constrain_name
    DeleteTablePrimaryKey(T, T)

}


impl<T, U, V, W, X> Transaction<T> for TableOperation<T, U, V, W, X> 
    where 
        T: Into<String> + Debug + Display + Sync,
        U: Into<String> + Debug + Display + Sync,
        V: Into<String> + Debug + Display + Sync,
        W: Into<String> + Debug + Display + Sync,
        X: Into<String> + Debug + Display + Sync
    {}

#[async_trait]
impl<T, U, V, W, X> DatabaseOperation for TableOperation<T, U, V, W, X> 
    where 
        T: Into<String> + Debug + Display + Sync,
        U: Into<String> + Debug + Display + Sync,
        V: Into<String> + Debug + Display + Sync,
        W: Into<String> + Debug + Display + Sync,
        X: Into<String> + Debug + Display + Sync
{
    async fn execute(&self) {
        let stmt = match &*self {
            TableOperation::CreateTable(table_name, table_fields) =>
                format!(
                    "CREATE TABLE {table_name} ({:?});",
                    table_fields.iter().map(|entity_field|
                        format!("{} {}", entity_field.field_name, entity_field.field_type_to_postgres())
                    ).collect::<Vec<String>>().join(", ")
                ).replace('"', ""),

            TableOperation::AlterTableName(old_table_name, new_table_name) =>
                format!("ALTER TABLE {old_table_name} RENAME TO {new_table_name};"),

            TableOperation::AddTableForeignKey(
                table_name, 
                foreign_key_name,
                column_foreign_key, 
                table_to_reference,
                column_to_reference
            ) => format!(
                "ALTER TABLE {table_name} ADD CONSTRAINT {foreign_key_name} \
                FOREIGN KEY ({column_foreign_key}) REFERENCES {table_to_reference} ({column_to_reference});"
            ),

            TableOperation::DeleteTableForeignKey(table_with_foreign_key, constrain_name) =>
                format!("ALTER TABLE {table_with_foreign_key} DROP CONSTRAINT {constrain_name};"),

            TableOperation::AddTablePrimaryKey(
                table_name,
                entity_field
            ) => format!(
                "ALTER TABLE {} ADD PRIMARY KEY (\"{}\");",
                table_name,
                entity_field.field_name
            ),

            TableOperation::DeleteTablePrimaryKey(
                table_name,
                primary_key_name
            ) => format!(
                "ALTER TABLE {table_name} DROP CONSTRAINT {primary_key_name} CASCADE;"
            ),

        };

        QUERIES_TO_EXECUTE.lock().unwrap().push(stmt)
    }
}

/// Helper to relate the operations that Canyon should do when a change on a field should
#[derive(Debug)]
enum ColumnOperation<T: Into<String> + std::fmt::Debug + Display + Sync> {
    CreateColumn(T, CanyonRegisterEntityField),
    DeleteColumn(T, String),
    // AlterColumnName,
    AlterColumnType(T, CanyonRegisterEntityField),
    AlterColumnDropNotNull(T, CanyonRegisterEntityField),
    AlterColumnSetNotNull(T, CanyonRegisterEntityField),
    // TODO if implement throught annotations, modify for both GENERATED {ALWAYS,BY DEFAULT}
    AlterColumnAddIdentity(T, CanyonRegisterEntityField),
    AlterColumnDropIdentity(T, CanyonRegisterEntityField)

}

impl<T> Transaction<Self> for ColumnOperation<T> 
    where T: Into<String> + std::fmt::Debug + Display + Sync
{}

#[async_trait]
impl<T> DatabaseOperation for ColumnOperation<T> 
    where T: Into<String> + std::fmt::Debug + Display + Sync
{
    async fn execute(&self) {
        let stmt = match &*self {
            ColumnOperation::CreateColumn(table_name, entity_field) =>
                format!(
                    "ALTER TABLE {table_name} ADD COLUMN \"{}\" {};", 
                    entity_field.field_name, 
                    entity_field.field_type_to_postgres()
                ),
            ColumnOperation::DeleteColumn(table_name, column_name) =>
                format!("ALTER TABLE {table_name} DROP COLUMN \"{column_name}\";"),
            ColumnOperation::AlterColumnType(table_name, entity_field) =>
                format!(
                    "ALTER TABLE {} ALTER COLUMN \"{}\" TYPE {};",
                    table_name,
                    entity_field.field_name, 
                    entity_field.to_postgres_alter_syntax()
                ),
            ColumnOperation::AlterColumnDropNotNull(table_name, entity_field) =>
                format!(
                    "ALTER TABLE {} ALTER COLUMN \"{}\" DROP NOT NULL;",
                    table_name,
                    entity_field.field_name
                ),
            ColumnOperation::AlterColumnSetNotNull(table_name, entity_field) =>
                format!(
                    "ALTER TABLE {} ALTER COLUMN \"{}\" SET NOT NULL;",
                    table_name,
                    entity_field.field_name
                ),

            ColumnOperation::AlterColumnAddIdentity(table_name, entity_field) =>
                format!(
                    "ALTER TABLE {} ALTER COLUMN \"{}\" ADD GENERATED ALWAYS AS IDENTITY;",
                    table_name,
                    entity_field.field_name
                ),

            ColumnOperation::AlterColumnDropIdentity(table_name, entity_field) =>
                format!(
                    "ALTER TABLE {} ALTER COLUMN \"{}\" DROP IDENTITY;",
                    table_name,
                    entity_field.field_name
                ),

        };

        QUERIES_TO_EXECUTE.lock().unwrap().push(stmt)
    }
}


/// Helper for operations involving sequences
#[derive(Debug)]
enum SequenceOperation<T: Into<String> + std::fmt::Debug + Display + Sync> {
    ModifySequence(T, CanyonRegisterEntityField),
}

impl<T> Transaction<Self> for SequenceOperation<T>
    where T: Into<String> + std::fmt::Debug + Display + Sync
{}

#[async_trait]
impl<T> DatabaseOperation for SequenceOperation<T>
    where T: Into<String> + std::fmt::Debug + Display + Sync
{
    async fn execute(&self) {
        let stmt = match &*self {
            SequenceOperation::ModifySequence(table_name, entity_field) =>
                format!(
                    "SELECT setval(pg_get_serial_sequence('{}', '{}'), max(\"{}\")) from {};",
                    table_name,
                    entity_field.field_name,
                    entity_field.field_name,
                    table_name
                )
        };
        QUERIES_TO_EXECUTE.lock().unwrap().push(stmt)
    }
}