/// `PostgreSQL` entities for map the multiple rows that are related to one table, and the multiple
/// columns that are related to those table
pub mod information_schema_row_mapper {
    use canyon_crud::bounds::{Column, Row, ColumnType, ColumnTypeValue};

    /// The representation of a row of results when the `information schema` it's queried
    ///
    /// Too see an example, see the docs of [`crate::handler::CanyonHandler`] on `fn@get_info_of_entities`
    #[derive(Debug)]
    pub struct RowTable {
        pub table_name: String,
        pub columns: Vec<RelatedColumn>,
    }

    /// A column retrived from the `information schema` query that belongs to a [`RowTable`] element,
    /// representing one of the total columns of a table
    #[derive(Debug)]
    pub struct RelatedColumn {
        pub column_identifier: String,
        // pub datatype_str: String,
        // pub datatype: TypeId,
        pub value: ColumnTypeValue,
    }

    // /// Represents the relation between a real value stored inside a [`RelatedColumn`]
    // /// and the datatype of that value
    // #[derive(Debug)]
    // pub enum ColumnTypeValue {
    //     StringValue(Option<String>),
    //     IntValue(Option<i32>),
    //     NoneValue,
    // }
    // impl ColumnTypeValue {
    //     pub fn from_crud(ctv: canyon_crud::bounds::ColumnTypeValue) -> Self {
    //         match ctv {
    //             canyon_crud::bounds::ColumnTypeValue::StringValue(s) => Self::StringValue(s),
    //             canyon_crud::bounds::ColumnTypeValue::IntValue(i) => Self::IntValue(i),
    //             canyon_crud::bounds::ColumnTypeValue::NoneValue => Self::NoneValue,
    //         }
    //     }

    //     pub fn value_from_column(row: &dyn Row, col: Column) {
    //         match col.type_().as_any().downcast_ref::<ColumnType>().unwrap() {
    //             ColumnType::Postgres(v) => {
    //                 match *v {
    //                     TokioPostgresType::NAME | TokioPostgresType::VARCHAR | TokioPostgresType::TEXT => 
    //                     {
    //                         ColumnTypeValue::StringValue(
    //                             row.get_opt::<&str>(col.name())
    //                                 .map(|opt| opt.to_owned()),
    //                         )
    //                     }
    //                     TokioPostgresType::INT4 => {
    //                         ColumnTypeValue::IntValue(
    //                             row.get_opt::<i32>(col.name()),
    //                         )
    //                     }
    //                     _ => ColumnTypeValue::NoneValue,
    //                 }
    //             },
    //             ColumnType::SqlServer(v) => todo!()
    //             // {
    //             //     match v {
    //             //         tiberius::ColumnType::Null => todo!(),
    //             //         tiberius::ColumnType::Bit => todo!(),
    //             //         tiberius::ColumnType::Int1 => todo!(),
    //             //         tiberius::ColumnType::Int2 => todo!(),
    //             //         tiberius::ColumnType::Int4 => todo!(),
    //             //         tiberius::ColumnType::Int8 => todo!(),
    //             //         tiberius::ColumnType::Datetime4 => todo!(),
    //             //         tiberius::ColumnType::Float4 => todo!(),
    //             //         tiberius::ColumnType::Float8 => todo!(),
    //             //         tiberius::ColumnType::Money => todo!(),
    //             //         tiberius::ColumnType::Datetime => todo!(),
    //             //         tiberius::ColumnType::Money4 => todo!(),
    //             //         tiberius::ColumnType::Guid => todo!(),
    //             //         tiberius::ColumnType::Intn => todo!(),
    //             //         tiberius::ColumnType::Bitn => todo!(),
    //             //         tiberius::ColumnType::Decimaln => todo!(),
    //             //         tiberius::ColumnType::Numericn => todo!(),
    //             //         tiberius::ColumnType::Floatn => todo!(),
    //             //         tiberius::ColumnType::Datetimen => todo!(),
    //             //         tiberius::ColumnType::Daten => todo!(),
    //             //         tiberius::ColumnType::Timen => todo!(),
    //             //         tiberius::ColumnType::Datetime2 => todo!(),
    //             //         tiberius::ColumnType::DatetimeOffsetn => todo!(),
    //             //         tiberius::ColumnType::BigVarBin => todo!(),
    //             //         tiberius::ColumnType::BigVarChar => todo!(),
    //             //         tiberius::ColumnType::BigBinary => todo!(),
    //             //         tiberius::ColumnType::BigChar => todo!(),
    //             //         tiberius::ColumnType::NVarchar => todo!(),
    //             //         tiberius::ColumnType::NChar => todo!(),
    //             //         tiberius::ColumnType::Xml => todo!(),
    //             //         tiberius::ColumnType::Udt => todo!(),
    //             //         tiberius::ColumnType::Text => todo!(),
    //             //         tiberius::ColumnType::Image => todo!(),
    //             //         tiberius::ColumnType::NText => todo!(),
    //             //         tiberius::ColumnType::SSVariant => todo!(), 
    //             //     }
    //             // },
    //         }
    //     }
    // }
}

/// This mod contains the structs necessary to map the data retrieved when the
/// `information schema` PostgreSQL table it's queried
pub mod rows_to_table_mapper {

    /// Model that represents the database entities that belongs to the current schema.
    ///
    /// Basically, it's an agrupation of rows of results when Canyon queries the `information schema`
    /// table, grouping by table name (one [`DatabaseTable`] is the rows that contains the information
    /// of a table)
    #[derive(Debug, Clone)]
    pub struct DatabaseTable<'a> {
        pub table_name: String,
        pub columns: Vec<DatabaseTableColumn<'a>>,
    }

    /// Represents the *metadata* associated with a column that belongs to a `PostgreSQL` table.
    #[derive(Debug, Clone, Default)]
    pub struct DatabaseTableColumn<'a> {
        pub column_name: String,
        pub postgres_datatype: String,
        pub character_maximum_length: Option<i32>,
        pub is_nullable: bool,
        // Care, postgres type is varchar
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
        pub is_identity: bool,
        pub identity_generation: Option<String>,
        pub phantom: &'a str, // TODO
    }

    impl<'a> DatabaseTableColumn<'a> {
        pub fn new() -> DatabaseTableColumn<'a> {
            Self {
                column_name: String::new(),
                postgres_datatype: String::new(),
                character_maximum_length: None,
                is_nullable: true,
                column_default: None,
                numeric_precision: None,
                numeric_scale: None,
                numeric_precision_radix: None,
                datetime_precision: None,
                interval_type: None,
                foreign_key_info: None,
                foreign_key_name: None,
                primary_key_info: None,
                primary_key_name: None,
                is_identity: false,
                identity_generation: None,
                phantom: "",
            }
        }
    }
}
