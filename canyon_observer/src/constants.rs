pub mod queries {
    pub static CANYON_MEMORY_TABLE: &str =
        "CREATE TABLE IF NOT EXISTS canyon_memory (
            id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
            filepath VARCHAR NOT NULL,
            struct_name VARCHAR NOT NULL
        )";
}

pub mod postgresql_queries {
    pub static FETCH_PUBLIC_SCHEMA: &str =
        "SELECT
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

/// TODO
pub mod regex {
    // TODO @gbm25
}

/// TODO
pub mod rust_type {
    pub const I32: &str = "i32";
    pub const OPT_I32: &str = "Option<i32>";
    pub const I64: &str = "i64";
    pub const OPT_I64: &str = "Option<i64>";
    pub const STRING: &str = "String";
    pub const OPT_STRING: &str = "Option<String>";
    pub const BOOL: &str = "bool";
    pub const OPT_BOOL: &str = "Option<bool>";
    pub const NAIVE_DATE: &str = "NaiveDate";
    pub const OPT_NAIVE_DATE: &str = "Option<NaiveDate>";
}

/// TODO
pub mod postgresql_type {
    // TODO @gbm25
}

/// Contains fragments queries to be invoked as const items and to be concatenated
/// with dynamic data
///
/// Ex: ` format!("{} PRIMARY KEY GENERATED ALWAYS AS IDENTITY", postgres_datatype_syntax)`
pub mod query_chunk {
    // TODO @gbm25
}
