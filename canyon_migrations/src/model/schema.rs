//! Tables, columns and keys.

use super::{CatalogSql, Ident, Logical, ModelError, QualifiedName};
use canyon_core::types::SqlType;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A column default read from the database. `constraint` is set only on SQL Server,
/// which names its default constraints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ColumnDefault {
    /// An expression the catalog gives as SQL (PostgreSQL `pg_get_expr`, SQL Server
    /// `definition`, MySQL with `DEFAULT_GENERATED`). Written back as it is.
    Expr {
        constraint: Option<Ident>,
        sql: CatalogSql,
    },
    /// A plain value (MySQL `COLUMN_DEFAULT` without `DEFAULT_GENERATED`, which comes
    /// unquoted). Data, not SQL: written back as an escaped literal.
    Literal {
        constraint: Option<Ident>,
        value: String,
    },
}

impl ColumnDefault {
    pub fn constraint(&self) -> Option<&Ident> {
        match self {
            Self::Expr { constraint, .. } | Self::Literal { constraint, .. } => constraint.as_ref(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ColumnDef {
    pub name: Ident,
    pub ty: Logical,
    pub nullable: bool,
    /// Read from the database only; Canyon doesn't manage defaults yet.
    pub default: Option<ColumnDefault>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Generated {
    Identity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PrimaryKey {
    /// `None` in the desired model: the dialect names it. The real name in the actual model.
    pub name: Option<Ident>,
    pub columns: Vec<Ident>,
    pub generated: Option<Generated>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ForeignKey {
    pub name: Option<Ident>,
    pub columns: Vec<Ident>,
    pub ref_table: QualifiedName,
    pub ref_columns: Vec<Ident>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TableDef {
    /// The Rust struct; `None` for a table that exists only in the database.
    pub entity: Option<String>,
    pub name: QualifiedName,
    pub columns: Vec<ColumnDef>,
    pub primary_key: Option<PrimaryKey>,
    pub foreign_keys: Vec<ForeignKey>,
}

impl TableDef {
    pub fn builder(entity: &str, table: &str) -> TableDefBuilder {
        TableDefBuilder {
            entity: entity.to_owned(),
            schema: None,
            table: table.to_owned(),
            columns: Vec::new(),
            pk: None,
            fks: Vec::new(),
            orphan_key: false,
        }
    }

    pub fn column(&self, name: &str) -> Option<&ColumnDef> {
        self.columns.iter().find(|c| c.name.as_str() == name)
    }

    pub fn is_identity(&self, column: &Ident) -> bool {
        self.primary_key.as_ref().is_some_and(|pk| {
            pk.generated.is_some() && pk.columns.len() == 1 && &pk.columns[0] == column
        })
    }
}

/// Builds a [`TableDef`] the way the generated `EntitySchema` code does.
#[must_use]
pub struct TableDefBuilder {
    entity: String,
    schema: Option<String>,
    table: String,
    columns: Vec<(String, SqlType, bool)>,
    pk: Option<(Vec<String>, Option<Generated>)>,
    fks: Vec<(String, String, String)>,
    orphan_key: bool,
}

impl TableDefBuilder {
    pub fn schema(mut self, schema: &str) -> Self {
        self.schema = Some(schema.to_owned());
        self
    }

    pub fn column(mut self, name: &str, ty: SqlType, nullable: bool) -> Self {
        self.columns.push((name.to_owned(), ty, nullable));
        self
    }

    /// Adds the last declared column to the primary key.
    pub fn primary_key(mut self, generated: Option<Generated>) -> Self {
        match self.columns.last() {
            Some((name, ..)) => {
                let (cols, gen_) = self.pk.get_or_insert_with(|| (Vec::new(), None));
                cols.push(name.clone());
                *gen_ = gen_.or(generated);
            }
            None => self.orphan_key = true,
        }
        self
    }

    /// The last declared column references `table` (`"t"` or `"s.t"`) `.column`.
    pub fn foreign_key(mut self, table: &str, column: &str) -> Self {
        match self.columns.last() {
            Some((name, ..)) => self
                .fks
                .push((name.clone(), table.to_owned(), column.to_owned())),
            None => self.orphan_key = true,
        }
        self
    }

    pub fn build(self) -> Result<TableDef, ModelError> {
        let name = QualifiedName::new(self.schema.as_deref(), &self.table)?;
        if self.orphan_key {
            return Err(ModelError::KeyWithoutColumn {
                table: name.to_string(),
            });
        }
        let mut columns: Vec<ColumnDef> = Vec::with_capacity(self.columns.len());
        for (col, ty, nullable) in self.columns {
            let col = Ident::new(&col)?;
            if columns.iter().any(|c| c.name == col) {
                return Err(ModelError::DuplicateColumn {
                    table: name.to_string(),
                    column: col.to_string(),
                });
            }
            columns.push(ColumnDef {
                name: col,
                ty: Logical::Known(ty),
                nullable,
                default: None,
            });
        }
        if let Some((cols, generated)) = &self.pk {
            let repeated = cols
                .iter()
                .enumerate()
                .find_map(|(i, c)| cols[..i].contains(c).then_some(c));
            if let Some(col) = repeated {
                return Err(ModelError::DuplicateColumn {
                    table: name.to_string(),
                    column: col.clone(),
                });
            }
            // Only a single-column key can be an identity
            if generated.is_some() && cols.len() > 1 {
                return Err(ModelError::IdentityOnCompositeKey {
                    table: name.to_string(),
                });
            }
        }
        let primary_key = self
            .pk
            .map(|(cols, generated)| -> Result<_, ModelError> {
                Ok(PrimaryKey {
                    name: None,
                    columns: cols
                        .iter()
                        .map(|c| Ident::new(c))
                        .collect::<Result<_, _>>()?,
                    generated,
                })
            })
            .transpose()?;
        let foreign_keys = self
            .fks
            .into_iter()
            .map(|(col, table, ref_col)| -> Result<_, ModelError> {
                Ok(ForeignKey {
                    name: None,
                    columns: vec![Ident::new(&col)?],
                    ref_table: QualifiedName::parse(&table)?,
                    ref_columns: vec![Ident::new(&ref_col)?],
                })
            })
            .collect::<Result<_, _>>()?;
        Ok(TableDef {
            entity: Some(self.entity),
            name,
            columns,
            primary_key,
            foreign_keys,
        })
    }
}

/// Every table, by qualified name (sorted: plans come out in a stable order). Serialized
/// as a list of tables, so reading one back goes through [`SchemaModel::from_tables`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "Vec<TableDef>", try_from = "Vec<TableDef>")]
pub struct SchemaModel {
    pub tables: BTreeMap<QualifiedName, TableDef>,
}

impl From<SchemaModel> for Vec<TableDef> {
    fn from(model: SchemaModel) -> Self {
        model.tables.into_values().collect()
    }
}

impl TryFrom<Vec<TableDef>> for SchemaModel {
    type Error = ModelError;
    fn try_from(tables: Vec<TableDef>) -> Result<Self, Self::Error> {
        Self::from_tables(tables)
    }
}

impl SchemaModel {
    pub fn from_tables(tables: Vec<TableDef>) -> Result<Self, ModelError> {
        let mut map = BTreeMap::new();
        for table in tables {
            let key = table.name.clone();
            if map.insert(key.clone(), table).is_some() {
                return Err(ModelError::DuplicateTable(key.to_string()));
            }
        }
        Ok(Self { tables: map })
    }

    pub fn table(&self, name: &QualifiedName) -> Option<&TableDef> {
        self.tables.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use canyon_core::types::SqlType;

    fn player() -> TableDef {
        TableDef::builder("Player", "player")
            .column("id", SqlType::Int, false)
            .primary_key(Some(Generated::Identity))
            .column("name", SqlType::Text { max_len: Some(100) }, false)
            .column("league", SqlType::Int, true)
            .foreign_key("league", "id")
            .build()
            .unwrap()
    }

    #[test]
    fn builder_collects_columns_keys_and_foreign_keys() {
        let t = player();
        assert_eq!(t.entity.as_deref(), Some("Player"));
        assert_eq!(t.name.to_string(), "player");
        assert_eq!(t.columns.len(), 3);
        let pk = t.primary_key.as_ref().unwrap();
        assert_eq!(pk.columns, [Ident::new("id").unwrap()]);
        assert_eq!(pk.generated, Some(Generated::Identity));
        assert_eq!(t.foreign_keys[0].ref_table.to_string(), "league");
        assert_eq!(t.foreign_keys[0].columns, [Ident::new("league").unwrap()]);
        assert!(t.is_identity(&Ident::new("id").unwrap()));
        assert!(!t.is_identity(&Ident::new("name").unwrap()));
    }

    #[test]
    fn builder_reports_bad_names_and_duplicates() {
        let bad = TableDef::builder("X", "x")
            .column("a-b", SqlType::Int, false)
            .build();
        assert!(matches!(bad, Err(ModelError::InvalidIdent(_))));
        let dup = TableDef::builder("X", "x")
            .column("a", SqlType::Int, false)
            .column("a", SqlType::Int, false)
            .build();
        assert!(matches!(dup, Err(ModelError::DuplicateColumn { .. })));
        let twice_in_key = TableDef::builder("X", "x")
            .column("a", SqlType::Int, false)
            .primary_key(None)
            .primary_key(None)
            .build();
        assert!(matches!(
            twice_in_key,
            Err(ModelError::DuplicateColumn { .. })
        ));
        let identity_on_two = TableDef::builder("X", "x")
            .column("a", SqlType::Int, false)
            .primary_key(Some(Generated::Identity))
            .column("b", SqlType::Int, false)
            .primary_key(None)
            .build();
        assert!(matches!(
            identity_on_two,
            Err(ModelError::IdentityOnCompositeKey { .. })
        ));
        let orphan_key = TableDef::builder("X", "x").primary_key(None).build();
        assert!(matches!(
            orphan_key,
            Err(ModelError::KeyWithoutColumn { .. })
        ));
    }

    #[test]
    fn schema_model_rejects_duplicate_tables_and_orders_by_name() {
        let other = TableDef::builder("Team", "team")
            .column("id", SqlType::Int, false)
            .build()
            .unwrap();
        let model = SchemaModel::from_tables(vec![player(), other.clone()]).unwrap();
        let names: Vec<_> = model.tables.keys().map(ToString::to_string).collect();
        assert_eq!(names, ["player", "team"]);
        assert!(matches!(
            SchemaModel::from_tables(vec![other.clone(), other]),
            Err(ModelError::DuplicateTable(_))
        ));
    }

    #[test]
    fn foreign_key_to_another_schema() {
        let t = TableDef::builder("A", "a")
            .column("b_id", SqlType::Int, false)
            .foreign_key("other.b", "id")
            .build()
            .unwrap();
        assert_eq!(t.foreign_keys[0].ref_table.to_string(), "other.b");
    }

    #[test]
    fn schema_model_json_round_trip_goes_through_from_tables() {
        let model = SchemaModel::from_tables(vec![player()]).unwrap();
        let json = serde_json::to_string(&model).unwrap();
        assert_eq!(serde_json::from_str::<SchemaModel>(&json).unwrap(), model);
        let twice = format!("[{0},{0}]", serde_json::to_string(&player()).unwrap());
        assert!(
            serde_json::from_str::<SchemaModel>(&twice).is_err(),
            "duplicate table"
        );
    }

    #[test]
    fn catalog_sql_never_comes_from_json() {
        let json = r#"{"Expr":{"constraint":null,"sql":"0; DROP TABLE x"}}"#;
        assert!(serde_json::from_str::<ColumnDefault>(json).is_err());
        let literal = r#"{"Literal":{"constraint":null,"value":"it's"}}"#;
        assert!(serde_json::from_str::<ColumnDefault>(literal).is_ok());
    }
}
