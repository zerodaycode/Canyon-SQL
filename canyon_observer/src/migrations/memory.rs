use crate::constants;
use canyon_crud::bounds::QueryParameter;
use canyon_crud::{bounds::RowOperations, crud::Transaction, DatabaseType, DatasourceConfig};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use walkdir::WalkDir;

use super::register_types::CanyonRegisterEntity;

/// Convenient struct that contains the necessary data and operations to implement
/// the `Canyon Memory`.
///
/// Canyon Memory it's just a convenient way of relate the data of a Rust source
/// code file and the `CanyonEntity` (if so), helping Canyon to know what source
/// file contains a `#[canyon_entity]` annotation and restricting it to just one
/// annotated struct per file.
///
/// This limitation it's imposed by design. Canyon, when manages all the entities in
/// the user's source code, needs to know for future migrations the old data about a structure
/// and the new modified one.
///
/// For example, let's say that you have a:
/// ```
/// pub struct Person {
///    /* some fields */
/// }
/// ```
///
/// and you decided to modify it's Ident and change it to `Human`.
///
/// Canyon will take care about modifying the Database, and `ALTER TABLE` to edit the actual data for you,
/// but, if it's not able to get the data to know that the old one is `Person` and the new one it's `Human`.
/// it will simply drop the table (losing all your data) and creating a new table `Human`.
///
/// So, we decised to follow the next approach:
/// Every entity annotated with a `#[canyon_entity]` annotation will be related to only unique Rust source
/// code file. If we find more, Canyon will raise and error saying that it does not allows to having more than
/// one managed entity per source file.
///
/// Then, we will store the entities data in a special table only for Canyon, where we will create the relation
/// between the source file, the entity and it's fields and data.
///
/// So, if the user wants or needs to modify the data of it's entity, Canyon can secure that will perform the
/// correct operations because we can't "remember" how that entity was, and how it should be now, avoiding
/// potentially dangerous operations due to lack of knowing what entity relates with new data.
///
/// The `memory field` HashMap is made by the filepath as a key, and the struct's name as value
#[derive(Debug)]
pub struct CanyonMemory {
    pub memory: Vec<CanyonMemoryAnalyzer>,
    pub renamed_entities: HashMap<String, String>,
}

// Makes this structure able to make queries to the database
impl Transaction<Self> for CanyonMemory {}

impl CanyonMemory {
    /// Queries the database to retrieve internal data about the structures
    /// tracked by `CanyonSQL`
    #[allow(clippy::nonminimal_bool)]
    pub async fn remember(
        datasource: &DatasourceConfig<'static>,
        canyon_entities: &Vec<CanyonRegisterEntity<'_>>,
    ) -> Self {
        // Creates the memory table if not exists
        Self::create_memory(datasource.name, &datasource.properties.db_type).await;

        // Retrieve the last status data from the `canyon_memory` table
        // TODO still pending on the target schema, for now they are created on the default one
        let res = Self::query("SELECT * FROM canyon_memory", [], datasource.name)
            .await
            .expect("Error querying Canyon Memory");
        let mem_results = res.as_canyon_rows();

        // Manually maps the results
        let mut db_rows = Vec::new();
        for row in mem_results.iter() {
            let db_row = CanyonMemoryRow {
                id: row.get::<i32>("id"),
                filepath: row.get::<&str>("filepath"),
                struct_name: row.get::<&str>("struct_name"),
                declared_table_name: row.get::<&str>("declared_table_name"),
            };
            db_rows.push(db_row);
        }
        println!("Data in the canyon_memory table: {db_rows:?}");

        // Parses the source code files looking for the #[canyon_entity] annotated classes
        let mut mem = Self {
            memory: Vec::new(),
            renamed_entities: HashMap::new(),
        };
        Self::find_canyon_entity_annotated_structs(&mut mem, canyon_entities).await;

        // Insert into the memory table the new discovered entities
        // Care, insert the new ones, delete the olds
        // Also, updates the registry when the fields changes
        let mut updates = Vec::new();

        for _struct in &mem.memory {
            // When the filepath and the struct hasn't been modified and are already on db
            let already_in_db = db_rows.iter().any(|el| {
                (el.filepath == _struct.filepath && el.struct_name == _struct.struct_name)
                    || ((el.filepath != _struct.filepath && el.struct_name == _struct.struct_name)
                        || (el.filepath == _struct.filepath
                            && el.struct_name != _struct.struct_name))
            });
            if !already_in_db {
                match CanyonMemory::query(
                    constants::queries::INSERT_INTO_CANYON_MEMORY,
                    [
                        &_struct.filepath as &dyn QueryParameter,
                        &_struct.struct_name,
                        &_struct.declared_table_name,
                    ],
                    datasource.name,
                )
                .await
                {
                    Ok(v) => println!("Query insert CM OK: {v:?}"),
                    Err(e) => println!("Error update CM: {e:?}"),
                }
            }

            // When the struct or the filepath it's already on db but one of the two has been modified
            let need_to_update = db_rows.iter().find(|el| {
                (el.filepath == _struct.filepath || el.struct_name == _struct.struct_name)
                    && !(el.filepath == _struct.filepath && el.struct_name == _struct.struct_name)
            });

            // updated means: the old one. The value to update
            if let Some(old) = need_to_update {
                updates.push(old.struct_name);

                match CanyonMemory::query(
                    constants::queries::UPDATE_CANYON_MEMORY,
                    [
                        &_struct.filepath as &dyn QueryParameter,
                        &_struct.struct_name,
                        &_struct.declared_table_name,
                        &old.id,
                    ],
                    datasource.name,
                )
                .await
                {
                    Ok(v) => println!("Query update CM OK: {v:?}"),
                    Err(e) => println!("Error update CM: {e:?}"),
                }

                // if the updated element is the struct name, we add it to the table_rename Hashmap
                let rename_table = old.struct_name != _struct.struct_name;

                if rename_table {
                    mem.renamed_entities.insert(
                        _struct.struct_name.to_string(), // The new one
                        old.struct_name.to_string(),     // The old one
                    );
                }
            }
        }

        // Deletes the records when a table is dropped on the previous Canyon run
        db_rows.into_iter().for_each(|db_row| {
            if !mem
                .memory
                .iter()
                .any(|entity| entity.struct_name == db_row.struct_name)
                && !updates.contains(&db_row.struct_name)
            {
                // crate::add_cm_query_to_execute(stmt, datasource.name, &[&db_row.struct_name]);
            }
        });
        mem
    }

    /// Parses the Rust source code files to find the one who contains Canyon entities
    /// ie -> annotated with `#[canyon_entity]`
    async fn find_canyon_entity_annotated_structs(
        &mut self,
        canyon_entities: &[CanyonRegisterEntity<'_>],
    ) {
        for file in WalkDir::new("./src")
            .into_iter()
            .filter_map(|file| file.ok())
        {
            if file.metadata().unwrap().is_file()
                && file.path().display().to_string().ends_with(".rs")
            {
                // Opening the source code file
                let contents =
                    fs::read_to_string(file.path()).expect("Something went wrong reading the file");

                let mut canyon_entity_macro_counter = 0;
                let mut struct_name = String::new();
                for line in contents.split('\n') {
                    if line.contains("#[") // separated checks for possible different paths
                        && line.contains("canyon_entity")
                        && !line.starts_with("//")
                    {
                        canyon_entity_macro_counter += 1;
                    }

                    let re = Regex::new(r#"\bstruct\s+(\w+)"#).unwrap();
                    if let Some(captures) = re.captures(line) {
                        struct_name.push_str(captures.get(1).unwrap().as_str());
                    }
                }

                // This limitation will be removed in future versions, when the memory
                // will be able to track every aspect of an entity
                match canyon_entity_macro_counter {
                    0 => (),
                    1 => {
                        let canyon_entity = canyon_entities
                            .iter()
                            .find(|ce| ce.entity_name == struct_name);
                        if let Some(c_entity) = canyon_entity {
                            self.memory.push(CanyonMemoryAnalyzer {
                                filepath: file.path().display().to_string().replace('\\', "/"),
                                struct_name: struct_name.clone(),
                                declared_table_name: c_entity.entity_db_table_name.to_string(),
                            })
                        }
                    }
                    _ => panic!(
                        "Canyon-SQL does not support having multiple structs annotated
                        with `#[canyon::entity]` on the same file when the migrations are enabled"
                    ),
                }
            }
        }
    }

    /// Generates, if not exists the `canyon_memory` table
    async fn create_memory(datasource_name: &str, database_type: &DatabaseType) {
        let query = if database_type == &DatabaseType::PostgreSql {
            constants::postgresql_queries::CANYON_MEMORY_TABLE
        } else {
            constants::mssql_queries::CANYON_MEMORY_TABLE
        };

        Self::query(query, [], datasource_name)
            .await
            .expect("Error creating the 'canyon_memory' table");
    }
}

/// Represents a single row from the `canyon_memory` table
#[derive(Debug)]
struct CanyonMemoryRow<'a> {
    id: i32,
    filepath: &'a str,
    struct_name: &'a str,
    declared_table_name: &'a str,
}

/// Represents the data that will be serialized in the `canyon_memory` table
#[derive(Debug)]
pub struct CanyonMemoryAnalyzer {
    pub filepath: String,
    pub struct_name: String,
    pub declared_table_name: String,
}
