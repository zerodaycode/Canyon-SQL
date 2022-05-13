pub mod postgresql_queries {
    pub static FETCH_PUBLIC_SCHEMA: &'static str = 
        "select
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
}