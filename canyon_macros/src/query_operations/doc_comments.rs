#![allow(dead_code)]

pub const SELECT_ALL_BASE_DOC_COMMENT: &str = "Performs a `SELECT * FROM table_name`, where `table_name` it's \
        the name of your entity but converted to the corresponding \
        database convention. P.ej. PostgreSQL prefers table names declared \
        with snake_case identifiers.";

pub const SELECT_QUERYBUILDER_DOC_COMMENT: &str = "Generates a [`canyon_sql::query::querybuilder::SelectQueryBuilder`] \
        that allows you to customize the query by adding parameters and constrains dynamically. \
        \
        It performs a `SELECT * FROM  table_name`, where `table_name` it's the name of your \
        entity but converted to the corresponding database convention, \
        unless concrete values are set on the available parameters of the \
        `canyon_macro => table_name = \"table_name\", schema = \"schema\")`";

pub const FIND_BY_PK: &str = "Finds an element on the queried table that matches the \
        value of the field annotated with the `primary_key` attribute, \
        filtering by the column that it's declared as the primary \
        key on the database. \
        \
        *NOTE:* This operation it's only available if the [`CanyonEntity`] contains \
        some field declared as primary key. \
        \
        *returns:* a [`Result<Option<T>, Error>`], wrapping a possible failure \
        querying the database, or, if no errors happens, a success containing \
        and Option<T> with the data found wrapped in the Some(T) variant, \
        or None if the value isn't found on the table.";

pub const DS_ADVERTISING: &str = "The query it's made against the database with the configured datasource \
        described in the configuration file, and selected with the [`&str`] \
        passed as parameter.";

pub const DELETE: &str = "Deletes from a database entity the row that matches
    the current instance of a T type based on the actual value of the primary
    key field, returning a result
    indicating a possible failure querying the database.";
