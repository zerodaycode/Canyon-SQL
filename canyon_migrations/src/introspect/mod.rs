//! Reads the database's real schema into the model: tables, columns, defaults, primary
//! and foreign keys, plus the server version. Read-only.

#[cfg(feature = "mssql")]
mod mssql;
#[cfg(feature = "mysql")]
mod mysql;
#[cfg(feature = "postgres")]
mod pg;
pub(crate) mod version;

use crate::dialect::{DdlDialect, DdlWriter};
use crate::model::{
    ColumnDef, ColumnDefault, ColumnExtra, ForeignKey, Generated, Ident, Logical, NativeType,
    PrimaryKey, QualifiedName, SchemaModel, TableDef,
};
use crate::op::{Param, ServerVersion, Statement};
use canyon_core::connection::contracts::DbConnection;
use canyon_core::connection::database_type::DatabaseType;
use canyon_core::error::CanyonError;
use canyon_core::query::parameters::QueryParameter;
use canyon_core::rows::CanyonRows;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// Tables Canyon keeps for itself in the default schema: never part of the user's model.
pub(crate) const CANYON_TABLES: [&str; 3] = [
    "canyon_schema_snapshot",
    "canyon_migration_history",
    "canyon_memory",
];

/// One column as a catalog reports it.
#[derive(Debug, Clone)]
pub(crate) struct RawColumn {
    pub schema: String,
    pub table: String,
    pub column: String,
    pub native: NativeType,
    pub nullable: bool,
    pub identity: bool,
    pub default: Option<ColumnDefault>,
    pub extra: ColumnExtra,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KeyKind {
    Primary,
    Foreign,
}

/// One row per key column. `ordinal` starts at 1.
#[derive(Debug, Clone)]
pub(crate) struct RawKey {
    pub schema: String,
    pub table: String,
    pub name: String,
    pub kind: KeyKind,
    pub column: String,
    pub ordinal: u32,
    pub ref_schema: Option<String>,
    pub ref_table: Option<String>,
    pub ref_column: Option<String>,
}

/// What the database has.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Catalog {
    pub server: ServerVersion,
    /// The connection's default schema (MySQL: its database).
    pub default_schema: String,
    /// The schemas asked for besides the default one that exist.
    pub existing_schemas: Vec<Ident>,
    /// MySQL's `NO_BACKSLASH_ESCAPES` sql_mode; always false on the other backends.
    pub no_backslash_escapes: bool,
    pub model: SchemaModel,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum IntrospectError {
    Query {
        stage: &'static str,
        source: CanyonError,
    },
    Decode {
        stage: &'static str,
        detail: String,
    },
}

impl fmt::Display for IntrospectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Query { stage, source } => {
                write!(f, "reading the database {stage} failed: {source}")
            }
            Self::Decode { stage, detail } => {
                write!(f, "reading the database {stage} failed: {detail}")
            }
        }
    }
}

impl std::error::Error for IntrospectError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Query { source, .. } => Some(source),
            Self::Decode { .. } => None,
        }
    }
}

impl IntrospectError {
    pub(crate) fn decode(stage: &'static str, detail: impl fmt::Display) -> Self {
        Self::Decode {
            stage,
            detail: detail.to_string(),
        }
    }

    pub(crate) fn missing(stage: &'static str, what: &str) -> Self {
        Self::decode(stage, format_args!("no value for `{what}`"))
    }
}

/// The real schema: the connection's default schema plus `schemas` (the other ones the
/// entities use; may be empty), and the server version.
///
/// Run it under the migration lock and with no DDL transaction open: each catalog query
/// may go to another pooled connection, so the reads share no snapshot, and on SQL Server
/// they wait for another session's uncommitted DDL.
///
/// # Errors
/// A catalog query fails, or returns something that can't be read.
pub async fn introspect(
    conn: &impl DbConnection,
    db_type: DatabaseType,
    schemas: &[Ident],
) -> Result<Catalog, IntrospectError> {
    match db_type {
        #[cfg(feature = "postgres")]
        DatabaseType::PostgreSql => read::<pg::PgReader>(conn, schemas).await,
        #[cfg(feature = "mssql")]
        DatabaseType::SqlServer => read::<mssql::MssqlReader>(conn, schemas).await,
        #[cfg(feature = "mysql")]
        DatabaseType::MySQL => read::<mysql::MysqlReader>(conn, schemas).await,
    }
}

/// The first catalog query's answers.
pub(crate) struct Session {
    pub server: ServerVersion,
    pub default_schema: String,
    pub no_backslash_escapes: bool,
}

/// One backend's catalog queries and how to read their rows.
pub(crate) trait Reader {
    type Dialect: DdlDialect;

    /// Version, default schema and the backend's flags, in one row.
    const VERSION: &'static str;
    /// Head of the query for the asked schemas that exist, followed by the bound list.
    const SCHEMAS: &'static str;

    fn session(rows: &CanyonRows) -> Result<Session, IntrospectError>;
    /// The columns query, filtered to `schemas` (the default one first).
    fn columns_query(w: &mut DdlWriter<Self::Dialect>, schemas: &[String]);
    fn columns(rows: &CanyonRows) -> Result<Vec<RawColumn>, IntrospectError>;
    /// The keys query, filtered like the columns one.
    fn keys_query(w: &mut DdlWriter<Self::Dialect>, schemas: &[String]);
    fn keys(rows: &CanyonRows) -> Result<Vec<RawKey>, IntrospectError>;
    fn schema_names(rows: &CanyonRows) -> Result<Vec<String>, IntrospectError>;
}

// Each stage decodes in its own block, so its rows are gone before the next query
async fn read<R: Reader>(
    conn: &impl DbConnection,
    schemas: &[Ident],
) -> Result<Catalog, IntrospectError> {
    let session = {
        let mut w = DdlWriter::<R::Dialect>::new();
        w.raw(R::VERSION);
        R::session(&run(conn, &w.finish(), "version").await?)?
    };
    // The default schema as read above, bound: the server's own `current_schema()` could
    // answer differently on another pooled connection
    let wanted: Vec<String> = std::iter::once(session.default_schema.clone())
        .chain(schemas.iter().map(ToString::to_string))
        .collect();
    let columns = {
        let mut w = DdlWriter::<R::Dialect>::new();
        R::columns_query(&mut w, &wanted);
        R::columns(&run(conn, &w.finish(), "columns").await?)?
    };
    let keys = {
        let mut w = DdlWriter::<R::Dialect>::new();
        R::keys_query(&mut w, &wanted);
        R::keys(&run(conn, &w.finish(), "keys").await?)?
    };
    let existing_schemas = if schemas.is_empty() {
        Vec::new()
    } else {
        let mut w = DdlWriter::<R::Dialect>::new();
        w.raw(R::SCHEMAS);
        bind_list(&mut w, &wanted[1..]);
        R::schema_names(&run(conn, &w.finish(), "schemas").await?)?
            .iter()
            .map(|name| Ident::from_catalog(name))
            .collect()
    };
    let model = build_model(
        &session.default_schema,
        columns,
        keys,
        R::Dialect::to_logical,
    )?;
    Ok(Catalog {
        server: session.server,
        default_schema: session.default_schema,
        existing_schemas,
        no_backslash_escapes: session.no_backslash_escapes,
        model,
    })
}

/// Runs one catalog query with its bound parameters.
async fn run(
    conn: &impl DbConnection,
    statement: &Statement,
    stage: &'static str,
) -> Result<CanyonRows, IntrospectError> {
    let params: Vec<&dyn QueryParameter> =
        statement.params.iter().map(Param::as_query_param).collect();
    conn.query_rows(&statement.sql, &params)
        .await
        .map_err(|source| IntrospectError::Query { stage, source })
}

/// `(<p1>, <p2>, …)` with the names bound.
pub(crate) fn bind_list<D: DdlDialect>(w: &mut DdlWriter<D>, names: &[String]) {
    w.raw("(");
    for (i, name) in names.iter().enumerate() {
        if i > 0 {
            w.raw(", ");
        }
        w.param(Param::Text(name.clone()));
    }
    w.raw(")");
}

/// ` WHERE <column> IN (…)` over the wanted schemas.
pub(crate) fn schema_filter<D: DdlDialect>(
    w: &mut DdlWriter<D>,
    column: &'static str,
    schemas: &[String],
) {
    w.raw(" WHERE ").raw(column).raw(" IN ");
    bind_list(w, schemas);
}

fn qualify(default_schema: &str, schema: &str, table: &str) -> QualifiedName {
    QualifiedName {
        schema: (schema != default_schema).then(|| Ident::from_catalog(schema)),
        name: Ident::from_catalog(table),
    }
}

/// Raw catalog rows to the model. Leaves Canyon's own tables out of the default schema.
///
/// # Errors
/// A foreign key row without its referenced table or column.
pub(crate) fn build_model(
    default_schema: &str,
    columns: Vec<RawColumn>,
    mut keys: Vec<RawKey>,
    to_logical: fn(&NativeType) -> Logical,
) -> Result<SchemaModel, IntrospectError> {
    let ours =
        |schema: &str, table: &str| schema == default_schema && CANYON_TABLES.contains(&table);
    let mut tables: BTreeMap<QualifiedName, TableDef> = BTreeMap::new();
    let mut identities: BTreeSet<(QualifiedName, Ident)> = BTreeSet::new();

    for raw in columns.into_iter().filter(|c| !ours(&c.schema, &c.table)) {
        let name = qualify(default_schema, &raw.schema, &raw.table);
        let column = Ident::from_catalog(&raw.column);
        if raw.identity {
            identities.insert((name.clone(), column.clone()));
        }
        let table = tables.entry(name.clone()).or_insert_with(|| TableDef {
            entity: None,
            name,
            columns: Vec::new(),
            primary_key: None,
            foreign_keys: Vec::new(),
        });
        table.columns.push(ColumnDef {
            name: column,
            ty: to_logical(&raw.native),
            nullable: raw.nullable,
            default: raw.default,
            extra: raw.extra,
        });
    }

    keys.sort_by(|a, b| {
        (&a.schema, &a.table, &a.name, a.ordinal).cmp(&(&b.schema, &b.table, &b.name, b.ordinal))
    });
    for key in keys {
        let Some(table) = tables.get_mut(&qualify(default_schema, &key.schema, &key.table)) else {
            continue;
        };
        let column = Ident::from_catalog(&key.column);
        let name = Some(Ident::from_catalog(&key.name));
        match key.kind {
            KeyKind::Primary => {
                let pk = table.primary_key.get_or_insert_with(|| PrimaryKey {
                    name,
                    columns: Vec::new(),
                    generated: None,
                });
                pk.columns.push(column);
            }
            KeyKind::Foreign => {
                let (Some(ref_table), Some(ref_column)) = (&key.ref_table, &key.ref_column) else {
                    return Err(IntrospectError::decode(
                        "keys",
                        format_args!(
                            "foreign key `{}` on `{}` has no referenced column",
                            key.name, key.table
                        ),
                    ));
                };
                let ref_column = Ident::from_catalog(ref_column);
                match table.foreign_keys.iter_mut().find(|fk| fk.name == name) {
                    Some(fk) => {
                        fk.columns.push(column);
                        fk.ref_columns.push(ref_column);
                    }
                    None => {
                        let ref_schema = key.ref_schema.as_deref().unwrap_or(default_schema);
                        table.foreign_keys.push(ForeignKey {
                            name,
                            columns: vec![column],
                            ref_table: qualify(default_schema, ref_schema, ref_table),
                            ref_columns: vec![ref_column],
                        });
                    }
                }
            }
        }
    }

    // Only a single-column key is an identity in the model
    for table in tables.values_mut() {
        if let Some(pk) = table.primary_key.as_mut()
            && let [column] = pk.columns.as_slice()
            && identities.contains(&(table.name.clone(), column.clone()))
        {
            pk.generated = Some(Generated::Identity);
        }
    }

    Ok(SchemaModel { tables })
}

#[cfg(test)]
mod tests {
    use super::*;
    use canyon_core::types::SqlType;

    fn col(schema: &str, table: &str, column: &str, nullable: bool, identity: bool) -> RawColumn {
        RawColumn {
            schema: schema.into(),
            table: table.into(),
            column: column.into(),
            native: NativeType::named(if column == "name" { "xml" } else { "int" }, "x"),
            nullable,
            identity,
            default: None,
            extra: ColumnExtra::default(),
        }
    }

    fn key(
        (schema, table): (&str, &str),
        name: &str,
        kind: KeyKind,
        column: &str,
        ordinal: u32,
        refs: Option<(&str, &str, &str)>,
    ) -> RawKey {
        RawKey {
            schema: schema.into(),
            table: table.into(),
            name: name.into(),
            kind,
            column: column.into(),
            ordinal,
            ref_schema: refs.map(|r| r.0.into()),
            ref_table: refs.map(|r| r.1.into()),
            ref_column: refs.map(|r| r.2.into()),
        }
    }

    fn logical(native: &NativeType) -> Logical {
        match native.name.as_str() {
            "int" => Logical::Known(SqlType::Int),
            _ => Logical::Other(native.clone()),
        }
    }

    const P: &str = "public";

    fn sample() -> SchemaModel {
        build_model(
            P,
            vec![
                col(P, "league", "id", false, true),
                col(P, "league", "name", true, false),
                col(P, "comp", "x", false, false),
                col(P, "comp", "y", false, false),
                col(P, "child", "lx", true, false),
                col(P, "child", "ly", true, false),
                col(P, "child", "l", true, false),
                col(P, "canyon_migration_history", "id", false, true),
                col("s1", "canyon_migration_history", "id", false, false),
                col("s1", "child", "league", false, false),
            ],
            vec![
                key(
                    (P, "league"),
                    "league_pkey",
                    KeyKind::Primary,
                    "id",
                    1,
                    None,
                ),
                key((P, "comp"), "comp_pkey", KeyKind::Primary, "y", 2, None),
                key((P, "comp"), "comp_pkey", KeyKind::Primary, "x", 1, None),
                key(
                    (P, "child"),
                    "child_l_fkey",
                    KeyKind::Foreign,
                    "l",
                    1,
                    Some((P, "league", "id")),
                ),
                key(
                    (P, "child"),
                    "child_lx_ly_fkey",
                    KeyKind::Foreign,
                    "ly",
                    2,
                    Some((P, "comp", "y")),
                ),
                key(
                    (P, "child"),
                    "child_lx_ly_fkey",
                    KeyKind::Foreign,
                    "lx",
                    1,
                    Some((P, "comp", "x")),
                ),
                key(
                    (P, "child"),
                    "child_other_fkey",
                    KeyKind::Foreign,
                    "l",
                    1,
                    Some(("other", "t", "id")),
                ),
                key(
                    ("s1", "child"),
                    "child_league_fkey",
                    KeyKind::Foreign,
                    "league",
                    1,
                    Some((P, "league", "id")),
                ),
                key(
                    (P, "canyon_migration_history"),
                    "h_pkey",
                    KeyKind::Primary,
                    "id",
                    1,
                    None,
                ),
            ],
            logical,
        )
        .unwrap()
    }

    fn table<'a>(m: &'a SchemaModel, name: &str) -> &'a TableDef {
        m.tables
            .get(&QualifiedName::parse(name).unwrap())
            .unwrap_or_else(|| panic!("no table {name}"))
    }

    #[test]
    fn canyon_tables_are_left_out_of_the_default_schema_only() {
        let names: Vec<_> = sample().tables.keys().map(ToString::to_string).collect();
        assert_eq!(
            names,
            [
                "child",
                "comp",
                "league",
                "s1.canyon_migration_history",
                "s1.child"
            ]
        );
    }

    #[test]
    fn columns_keep_order_types_and_nullability() {
        let m = sample();
        let league = table(&m, "league");
        assert_eq!(league.entity, None);
        assert_eq!(league.columns[0].name.as_str(), "id");
        assert_eq!(league.columns[0].ty, Logical::Known(SqlType::Int));
        assert!(matches!(league.columns[1].ty, Logical::Other(_)));
        assert!(league.columns[1].nullable && !league.columns[0].nullable);
    }

    #[test]
    fn composite_keys_are_ordered_by_ordinal() {
        let m = sample();
        let id = Ident::from_catalog;
        let pk = table(&m, "comp").primary_key.clone().unwrap();
        assert_eq!(pk.columns, [id("x"), id("y")]);
        assert_eq!(pk.name, Some(id("comp_pkey")));
        let child = table(&m, "child");
        let fk = child
            .foreign_keys
            .iter()
            .find(|f| f.columns.len() == 2)
            .unwrap();
        assert_eq!(fk.columns, [id("lx"), id("ly")]);
        assert_eq!(fk.ref_columns, [id("x"), id("y")]);
        assert_eq!(fk.name, Some(id("child_lx_ly_fkey")));
    }

    #[test]
    fn default_schema_references_become_none() {
        let m = sample();
        let child = table(&m, "child");
        let to = |t: &str| {
            child
                .foreign_keys
                .iter()
                .find(|f| f.ref_table.name.as_str() == t)
                .unwrap()
                .ref_table
                .to_string()
        };
        assert_eq!(to("league"), "league");
        assert_eq!(to("t"), "other.t");
        let s1_child = table(&m, "s1.child");
        assert_eq!(s1_child.name.schema.as_ref().unwrap().as_str(), "s1");
        assert_eq!(s1_child.foreign_keys[0].ref_table.to_string(), "league");
    }

    #[test]
    fn identity_on_a_single_column_pk_is_generated() {
        let m = sample();
        let generated = |t: &str| table(&m, t).primary_key.as_ref().unwrap().generated;
        assert_eq!(generated("league"), Some(Generated::Identity));
        assert_eq!(generated("comp"), None);
    }

    #[test]
    fn identity_outside_the_key_is_not_generated() {
        let m = build_model(
            P,
            vec![
                col(P, "t", "id", false, false),
                col(P, "t", "n", false, true),
            ],
            vec![key((P, "t"), "t_pkey", KeyKind::Primary, "id", 1, None)],
            logical,
        )
        .unwrap();
        assert_eq!(table(&m, "t").primary_key.as_ref().unwrap().generated, None);
    }

    #[test]
    fn a_foreign_key_without_its_reference_is_an_error() {
        let mut broken = key((P, "t"), "t_x_fkey", KeyKind::Foreign, "x", 1, None);
        broken.ref_table = Some("u".into());
        let result = build_model(
            P,
            vec![col(P, "t", "x", true, false)],
            vec![broken],
            logical,
        );
        assert!(matches!(
            result,
            Err(IntrospectError::Decode { stage: "keys", .. })
        ));
    }
}
