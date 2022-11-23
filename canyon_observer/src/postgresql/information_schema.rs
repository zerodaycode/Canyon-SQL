/// `PostgreSQL` entities for map the multiple rows that are related to one table, and the multiple
/// columns that are related to those table
pub mod information_schema_row_mapper {
    use canyon_connection::{tiberius, tokio_postgres::types::Type as TokioPostgresType};
    use canyon_crud::bounds::{Column, Row, ColumnType, RowOperations};

    /// Represents the relation between a real value stored inside a [`RelatedColumn`]
    /// and the datatype of that value
    #[derive(Debug)]
    pub enum ColumnTypeValue {
        StringValue(Option<String>),
        IntValue(Option<i32>),
        NoneValue,
    }
    impl ColumnTypeValue {
        /// Retrieves the value stored in a [`Column`] for a passed [`Row`]
        pub fn get_value(row: &dyn Row, col: &Column) -> ColumnTypeValue {
            match col.column_type() {
                ColumnType::Postgres(v) => {
                    match *v {
                        TokioPostgresType::NAME | TokioPostgresType::VARCHAR | TokioPostgresType::TEXT => 
                        {
                            ColumnTypeValue::StringValue(
                                row.get_opt::<&str>(col.name())
                                    .map(|opt| opt.to_owned()),
                            )
                        }
                        TokioPostgresType::INT4 => {
                            ColumnTypeValue::IntValue(
                                row.get_opt::<i32>(col.name()),
                            )
                        }
                        _ => ColumnTypeValue::NoneValue,
                    }
                },
                ColumnType::SqlServer(v) =>
                {
                    match v {
                        tiberius::ColumnType::Null => todo!(),
                        tiberius::ColumnType::Bit => todo!(),
                        tiberius::ColumnType::Int1 => todo!(),
                        tiberius::ColumnType::Int2 => todo!(),
                        tiberius::ColumnType::Int4 => todo!(),
                        tiberius::ColumnType::Int8 => todo!(),
                        tiberius::ColumnType::Datetime4 => todo!(),
                        tiberius::ColumnType::Float4 => todo!(),
                        tiberius::ColumnType::Float8 => todo!(),
                        tiberius::ColumnType::Money => todo!(),
                        tiberius::ColumnType::Datetime => todo!(),
                        tiberius::ColumnType::Money4 => todo!(),
                        tiberius::ColumnType::Guid => todo!(),
                        tiberius::ColumnType::Intn => todo!(),
                        tiberius::ColumnType::Bitn => todo!(),
                        tiberius::ColumnType::Decimaln => todo!(),
                        tiberius::ColumnType::Numericn => todo!(),
                        tiberius::ColumnType::Floatn => todo!(),
                        tiberius::ColumnType::Datetimen => todo!(),
                        tiberius::ColumnType::Daten => todo!(),
                        tiberius::ColumnType::Timen => todo!(),
                        tiberius::ColumnType::Datetime2 => todo!(),
                        tiberius::ColumnType::DatetimeOffsetn => todo!(),
                        tiberius::ColumnType::BigVarBin => todo!(),
                        tiberius::ColumnType::BigVarChar => todo!(),
                        tiberius::ColumnType::BigBinary => todo!(),
                        tiberius::ColumnType::BigChar => todo!(),
                        tiberius::ColumnType::NVarchar => todo!(),
                        tiberius::ColumnType::NChar => todo!(),
                        tiberius::ColumnType::Xml => todo!(),
                        tiberius::ColumnType::Udt => todo!(),
                        tiberius::ColumnType::Text => todo!(),
                        tiberius::ColumnType::Image => todo!(),
                        tiberius::ColumnType::NText => todo!(),
                        tiberius::ColumnType::SSVariant => todo!(), 
                    }
                },
            }
        }
    }
}

/// This mod contains the structs necessary to map the data retrieved when the
/// `information schema` PostgreSQL table it's queried
pub mod rows_to_table_mapper {

    /// Model that represents the database entities that belongs to the current schema.
    ///
    /// Basically, it's an agrupation of rows of results when Canyon queries the `information schema`
    /// table, grouping by table name (one [`TableMetadata`] is the rows that contains the information
    /// of a table)
    #[derive(Debug, Clone)]
    pub struct TableMetadata<'a> {
        pub table_name: String,
        pub columns: Vec<ColumnMetadata<'a>>,
    }

    /// Represents the *metadata* associated with a column that belongs to a `PostgreSQL` table.
    #[derive(Debug, Clone, Default)]
    pub struct ColumnMetadata<'a> {
        pub column_name: String,
        pub postgres_datatype: String,
        pub character_maximum_length: Option<i32>,
        pub is_nullable: bool, // Care, postgres type is varchar
        pub column_default: Option<String>,
        pub numeric_precision: Option<i32>,
        pub numeric_scale: Option<i32>,
        pub numeric_precision_radix: Option<i32>,
        pub datetime_precision: Option<i32>,
        pub interval_type: Option<String>,
        pub foreign_key_info: Option<String>,
        pub foreign_key_name: Option<String>,
        pub primary_key_info: Option<String>,
        pub primary_key_name: Option<String>,
        pub is_identity: bool, // Care, postgres type is varchar
        pub identity_generation: Option<String>,
        pub phantom: &'a str, // TODO
    }
}
