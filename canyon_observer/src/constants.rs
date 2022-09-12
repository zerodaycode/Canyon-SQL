pub mod postgresql_queries {
    pub static FETCH_PUBLIC_SCHEMA: &'static str = 
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
            	THEN con.conname ELSE NULL END AS primary_key_name
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
    pub const I32: &'static str = "i32";
    pub const OPT_I32: &'static str = "Option<i32>";
    pub const I64: &'static str = "i64";
    pub const OPT_I64: &'static str = "Option<i64>";
    pub const STRING: &'static str = "String";
    pub const OPT_STRING: &'static str = "Option<String>";
    pub const BOOL: &'static str = "bool";
    pub const OPT_BOOL: &'static str = "Option<bool>";
    pub const NAIVE_DATE: &'static str = "NaiveDate";
    pub const OPT_NAIVE_DATE: &'static str = "Option<NaiveDate>";
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