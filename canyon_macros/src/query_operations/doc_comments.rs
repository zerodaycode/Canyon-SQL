pub const SELECT_ALL_BASE_DOC_COMMENT: &str =
    "/// Performs a `SELECT * FROM table_name`, where `table_name` it's \
        /// the name of your entity but converted to the corresponding \
        /// database convention. P.ej. PostgreSQL prefers table names declared \
        /// with snake_case identifiers.";

pub const SELECT_QUERYBUILDER_DOC_COMMENT: &str = 
    "/// Generates a [`canyon_sql::query::SelectQueryBuilder`] \
        /// that allows you to customize the query by adding parameters and constrains dynamically. \
        /// \
        /// It performs a `SELECT * FROM  table_name`, where `table_name` it's the name of your \
        /// entity but converted to the corresponding database convention, \
        /// unless concrete values are set on the available parameters of the \
        /// `canyon_macro => table_name = \"table_name\", schema = \"schema\")`";