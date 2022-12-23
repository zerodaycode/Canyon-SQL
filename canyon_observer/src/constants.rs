pub mod queries {
    
}

pub mod postgresql_queries {
    pub static CANYON_MEMORY_TABLE: &str =
        "CREATE TABLE IF NOT EXISTS canyon_memory (
            id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
            filepath VARCHAR NOT NULL,
            struct_name VARCHAR NOT NULL
        )";

    pub static FETCH_PUBLIC_SCHEMA: &str =
        "SELECT
            gi.table_name,
            gi.column_name,
            gi.data_type,
            gi.character_maximum_length,
            gi.is_nullable,
            gi.column_default,
            CASE WHEN starts_with(CAST(pg_catalog.pg_get_constraintdef(oid) AS TEXT), 'FOREIGN KEY')
            	THEN CAST(pg_catalog.pg_get_constraintdef(oid) AS TEXT) ELSE NULL END AS foreign_key_info,
            CASE WHEN starts_with(CAST(pg_catalog.pg_get_constraintdef(oid) AS TEXT), 'FOREIGN KEY')
            	THEN con.conname ELSE NULL END AS foreign_key_name,
            CASE WHEN starts_with(CAST(pg_catalog.pg_get_constraintdef(oid) AS TEXT), 'PRIMARY KEY')
            	THEN CAST(pg_catalog.pg_get_constraintdef(oid) AS TEXT) ELSE NULL END AS primary_key_info,
            CASE WHEN starts_with(CAST(pg_catalog.pg_get_constraintdef(oid) AS TEXT), 'PRIMARY KEY')
            	THEN con.conname ELSE NULL END AS primary_key_name,
            gi.is_identity,
            gi.identity_generation
        FROM
            information_schema.columns AS gi
        LEFT JOIN pg_catalog.pg_constraint AS con on
            gi.table_name = CAST(con.conrelid::regclass AS TEXT) AND
            gi.column_name = split_part(split_part(CAST(pg_catalog.pg_get_constraintdef(oid) AS TEXT),')',1),'(',2)
        WHERE
            table_schema = 'public';";
}


pub mod mssql_queries {
    pub static CANYON_MEMORY_TABLE: &str =
        "IF OBJECT_ID(N'[dbo].[canyon_memory]', N'U') IS NULL
        BEGIN
            CREATE TABLE dbo.canyon_memory (
                id					INT PRIMARY KEY IDENTITY,
                filepath			NVARCHAR(250) NOT NULL,
                struct_name			NVARCHAR(100) NOT NULL
            );
        END";

    pub static FETCH_PUBLIC_SCHEMA: &str =
        "SELECT
            gi.table_name,
            gi.column_name,
            gi.data_type,
            CAST(gi.character_maximum_length AS int),
            gi.is_nullable,
            gi.column_default,
            fk.foreign_key_info,
            fk.foreign_key_name,
            pk.CONSTRAINT_NAME as primary_key_info,
            pk.CONSTRAINT_NAME as primary_key_name
            FROM INFORMATION_SCHEMA.COLUMNS gi
            LEFT JOIN (
                SELECT
                    SCHEMA_NAME(f.schema_id) schemaName,
                    OBJECT_NAME(f.parent_object_id) ConstrainedTable,
                    COL_NAME(fc.parent_object_id, fc.parent_column_id) ConstrainedColumn,
                    f.name foreign_key_name,
                    CONCAT('FOREIGN KEY (',  
                    COL_NAME(fc.parent_object_id, fc.parent_column_id), ') REFERENCES ',
                    OBJECT_NAME(f.referenced_object_id),
                    '(',
                    COL_NAME(fc.referenced_object_id, fc.referenced_column_id)
                    , ')') AS foreign_key_info
                FROM
                    sys.foreign_keys AS f
                INNER JOIN
                    sys.foreign_key_columns AS fc
                ON f.OBJECT_ID = fc.constraint_object_id
                INNER JOIN
                    sys.tables t
                ON t.OBJECT_ID = fc.referenced_object_id
            ) AS fk
                ON fk.ConstrainedTable = gi.TABLE_NAME AND fk.ConstrainedColumn = gi.COLUMN_NAME  AND gi.TABLE_SCHEMA = fk.schemaName    
            LEFT JOIN (
                SELECT *
                FROM INFORMATION_SCHEMA.KEY_COLUMN_USAGE kcu
                WHERE OBJECTPROPERTY(OBJECT_ID(kcu.CONSTRAINT_SCHEMA + '.' + QUOTENAME(kcu.CONSTRAINT_NAME)), 'IsPrimaryKey') = 1
            ) AS pk
                ON pk.TABLE_NAME = gi.TABLE_NAME AND pk.CONSTRAINT_SCHEMA = gi.TABLE_SCHEMA AND pk.COLUMN_NAME = gi.COLUMN_NAME
            WHERE gi.TABLE_SCHEMA = 'dbo'";
}

/// Constant string values that holds regex patterns
pub mod regex_patterns {
    pub const EXTRACT_RUST_OPT_REGEX: &str = r"[Oo][Pp][Tt][Ii][Oo][Nn]<(?P<rust_type>[\w<>]+)>";
    pub const EXTRACT_FOREIGN_KEY_INFO: &str =  r"\w+\s\w+\s\((?P<current_column>\w+)\)\s\w+\s(?P<ref_table>\w+)\((?P<ref_column>\w+)\)";
}

/// Constant values that maps the string representation of the Rust
/// built-in types
#[allow(unused)]
pub mod rust_type {
    pub const I8: &str = "i8";
    pub const OPT_I8: &str = "Option<i8>";
    pub const U8: &str = "u8";
    pub const OPT_U8: &str = "Option<u8>";

    pub const I16: &str = "i16";
    pub const OPT_U16: &str = "Option<i16>";
    pub const U16: &str = "u16";
    pub const OPT_I16: &str = "Option<u16>";

    pub const I32: &str = "i32";
    pub const OPT_I32: &str = "Option<i32>";
    pub const U32: &str = "u32";
    pub const OPT_U32: &str = "Option<u32>";

    pub const I64: &str = "i64";
    pub const OPT_I64: &str = "Option<i64>";
    pub const U64: &str = "u64";
    pub const OPT_U64: &str = "Option<u64>";

    pub const F32: &str = "f32";
    pub const OPT_F32: &str = "Option<f32>";
    pub const F64: &str = "f64";
    pub const OPT_F64: &str = "Option<f64>";

    pub const STRING: &str = "String";
    pub const OPT_STRING: &str = "Option<String>";

    pub const BOOL: &str = "bool";
    pub const OPT_BOOL: &str = "Option<bool>";

    pub const NAIVE_DATE: &str = "NaiveDate";
    pub const OPT_NAIVE_DATE: &str = "Option<NaiveDate>";

    pub const NAIVE_TIME: &str = "NaiveTime";
    pub const OPT_NAIVE_TIME: &str = "Option<NaiveTime>";

    pub const NAIVE_DATE_TIME: &str = "NaiveDateTime";
    pub const OPT_NAIVE_DATE_TIME: &str = "Option<NaiveDateTime>";
}

/// TODO
pub mod postgresql_type {
    pub const INT_8: &str = "int8";
    pub const SMALL_INT: &str = "smallint";
    pub const INTEGER: &str = "integer";
    pub const BIGINT: &str = "bigint";
    pub const TEXT: &str = "text";
    pub const BOOLEAN: &str = "boolean";
    pub const DATE: &str = "date";
    pub const TIME: &str = "time";
    pub const DATETIME: &str = "timestamp without time zone";
}

pub mod sqlserver_type {
    pub const TINY_INT: &str = "TINY INT";
    pub const SMALL_INT: &str = "SMALL INT";
    pub const INT: &str = "INT";
    pub const BIGINT: &str = "BIGINT";
    // TODO More information needed, the number of characters may need to be variable and user-defined
    pub const NVARCHAR: &str = "nvarchar(max)";
    pub const BIT: &str = "BIT";
    pub const DATE: &str = "DATE";
    pub const TIME: &str = "TIME";
    pub const DATETIME: &str = "DATETIME2";
}

/// Contains fragments queries to be invoked as const items and to be concatenated
/// with dynamic data
///
/// Ex: ` format!("{} PRIMARY KEY GENERATED ALWAYS AS IDENTITY", postgres_datatype_syntax)`
pub mod query_chunk {
    // TODO @gbm25
}


pub mod mocked_data {
    use canyon_connection::lazy_static::lazy_static;

    use crate::migrations::information_schema::{ColumnMetadata, TableMetadata};

    lazy_static! {
        pub static ref TABLE_METADATA_LEAGUE_EX: TableMetadata = TableMetadata { 
            table_name: "league".to_string(), 
            columns: vec![
                ColumnMetadata {
                    column_name: "id".to_owned(),
                    datatype: "int".to_owned(),
                    character_maximum_length: None,
                    is_nullable: false,
                    column_default: None,
                    foreign_key_info: None,
                    foreign_key_name: None,
                    primary_key_info: Some("PK__league__3213E83FBDA92571".to_owned()),
                    primary_key_name: Some("PK__league__3213E83FBDA92571".to_owned()),
                    is_identity: false,
                    identity_generation: None
                }, 
                ColumnMetadata {
                    column_name: "ext_id".to_owned(),
                    datatype: "bigint".to_owned(),
                    character_maximum_length: None,
                    is_nullable: false,
                    column_default: None,
                    foreign_key_info: None,
                    foreign_key_name: None,
                    primary_key_info: None,
                    primary_key_name: None,
                    is_identity: false,
                    identity_generation: None
                }, 
                ColumnMetadata {
                    column_name: "slug".to_owned(),
                    datatype: "nvarchar".to_owned(),
                    character_maximum_length: None,
                    is_nullable: false,
                    column_default: None,
                    foreign_key_info: None,
                    foreign_key_name: None,
                    primary_key_info: None,
                    primary_key_name: None,
                    is_identity: false,
                    identity_generation: None
                }, 
                ColumnMetadata {
                    column_name: "name".to_owned(),
                    datatype: "nvarchar".to_owned(),
                    character_maximum_length: None,
                    is_nullable: false,
                    column_default: None,
                    foreign_key_info: None,
                    foreign_key_name: None,
                    primary_key_info: None,
                    primary_key_name: None,
                    is_identity: false,
                    identity_generation: None
                }, 
                ColumnMetadata {
                    column_name: "region".to_owned(), 
                    datatype: "nvarchar".to_owned(),
                    character_maximum_length: None,
                    is_nullable: false,
                    column_default: None,
                    foreign_key_info: None,
                    foreign_key_name: None,
                    primary_key_info: None,
                    primary_key_name: None,
                    is_identity: false,
                    identity_generation: None
                }, 
                ColumnMetadata {
                    column_name: "image_url".to_owned(),
                    datatype: "nvarchar".to_owned(),
                    character_maximum_length: None,
                    is_nullable: false,
                    column_default: None,
                    foreign_key_info: None,
                    foreign_key_name: None,
                    primary_key_info: None,
                    primary_key_name: None,
                    is_identity: false,
                    identity_generation: None 
                }
            ]
        };

        pub static ref NON_MATCHING_TABLE_METADATA: TableMetadata = TableMetadata { 
            table_name: "random_name_to_assert_false".to_string(), 
            columns: vec![]
        };
    }
}