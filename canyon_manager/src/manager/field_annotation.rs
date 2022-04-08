use std::convert::TryFrom;
use proc_macro2::Ident;
use syn::Attribute;

/// The available annotations for a field that belongs to any struct
/// annotaded with `#[canyon_entity]`
#[derive(Debug, Clone)]
pub enum EntityFieldAnnotation {
    ForeignKey(String, String)
}

impl EntityFieldAnnotation {

    pub fn new(ident: Ident, foreign_key: String) -> Self {
        let table_column_tuple = Self::foreign_key_parser(ident, foreign_key);
        Self::ForeignKey(
            table_column_tuple.clone().expect("Table value in attribute").0, 
            table_column_tuple.clone().expect("Column value in attribute").1
        )
    }

    pub fn get_as_string(&self) -> String {
        match &*self {
            Self::ForeignKey(table, column) => 
                format!("Table: {}, Column: {}",table,column)
        }
    }

    pub fn foreign_key_parser(ident: Ident, foreign_key: String) -> Result<(String, String), syn::Error> {
        
        let table_column:Vec<String> = foreign_key.split(",")
            .map(|x|x.trim().to_string())
            .collect();

        let mut table = String::new();
        let mut column = String::new();
        
        if table_column.iter().any(|c| c.starts_with("column")) && 
            table_column.iter().any(|c| c.starts_with("table"))
        {
            table.push_str(table_column.iter()
                .find(|&t|t.starts_with("table"))
                .unwrap()
                .split("=").collect::<Vec<&str>>().get(1).unwrap()
                .trim()
            );

            column.push_str(table_column.iter()
                .find(|&t|t.starts_with("column"))
                .unwrap()
                .split("=").collect::<Vec<&str>>().get(1).unwrap()
                .trim()
            );

            Ok((table, column))
        } else {
            Err(
                syn::Error::new_spanned(
                    ident,
                    "Malformed values on the {:?}`"
                )
            )
        }
    }
}


impl TryFrom<&&Attribute> for EntityFieldAnnotation {
    type Error = syn::Error;

    fn try_from(attribute: &&Attribute) -> Result<Self, Self::Error> {

        let (ident, _args) = (
            attribute.path.segments[0].ident.clone(),
            attribute.parse_args::<syn::LitStr>()
                .unwrap()
                .value()
        );

        Ok(
            match ident.to_string().as_str() {
                "foreign_key" => EntityFieldAnnotation::new(ident, _args),
                _ => {
                    return Err(
                        syn::Error::new_spanned(
                            ident.clone(), 
                            format!("Unknown attribute `{}`", ident)
                        )
                    )
                }
            }
        )
    }
}

