//! SQL Server catalog: `sys.tables` / `sys.columns` / key and foreign key views.

use super::{IntrospectError, KeyKind, RawColumn, RawKey, Reader, Session};
use super::{schema_filter, version};
use crate::dialect::DdlWriter;
use crate::model::{CatalogSql, ColumnDefault, ColumnExtra, Ident, Length, NativeType};
use canyon_core::query::querybuilder::syntax::dialect::MsSql;
use canyon_core::rows::CanyonRows;
use tiberius::Row;

pub(super) struct MssqlReader;

const VERSION: &str =
    "SELECT CAST(SERVERPROPERTY('ProductVersion') AS nvarchar(128)) AS v, SCHEMA_NAME() AS s";

const COLUMNS: &str = "SELECT SCHEMA_NAME(t.schema_id) AS schema_name, t.name AS table_name, \
    c.name AS column_name, ty.name AS type_name, CAST(c.max_length AS int) AS max_length, \
    CAST(c.precision AS int) AS num_precision, CAST(c.scale AS int) AS num_scale, \
    CAST(c.is_nullable AS int) AS nullable, CAST(c.is_identity AS int) AS is_identity, \
    CAST(c.is_computed AS int) AS is_computed, \
    dc.name AS default_name, dc.definition AS default_sql, c.collation_name AS collation \
    FROM sys.tables t \
    JOIN sys.columns c ON c.object_id = t.object_id \
    JOIN sys.types ty ON ty.user_type_id = c.user_type_id \
    LEFT JOIN sys.default_constraints dc \
      ON dc.parent_object_id = c.object_id AND dc.parent_column_id = c.column_id";
// Tables SQL Server ships (`sysdiagrams`) and system-versioned history tables aren't the
// user's
const OWN_TABLES: &str = " AND t.is_ms_shipped = 0 AND t.temporal_type <> 1";
const COLUMNS_ORDER: &str = " ORDER BY 1, 2, c.column_id";

const PRIMARY_KEYS: &str = "SELECT SCHEMA_NAME(t.schema_id) AS schema_name, t.name AS table_name, \
    kc.name AS name, 'p' AS kind, col.name AS column_name, CAST(ic.key_ordinal AS int) AS ord, \
    CAST(NULL AS nvarchar(128)) AS ref_schema, CAST(NULL AS nvarchar(128)) AS ref_table, \
    CAST(NULL AS nvarchar(128)) AS ref_column \
    FROM sys.key_constraints kc \
    JOIN sys.tables t ON t.object_id = kc.parent_object_id \
    JOIN sys.index_columns ic ON ic.object_id = kc.parent_object_id AND ic.index_id = kc.unique_index_id \
    JOIN sys.columns col ON col.object_id = ic.object_id AND col.column_id = ic.column_id";
const PRIMARY_KEYS_TAIL: &str = " AND kc.type = 'PK'";
const FOREIGN_KEYS: &str = " UNION ALL \
    SELECT SCHEMA_NAME(t.schema_id), t.name, fk.name, 'f', pc.name, \
    CAST(fkc.constraint_column_id AS int), SCHEMA_NAME(rt.schema_id), rt.name, rc.name \
    FROM sys.foreign_keys fk \
    JOIN sys.tables t ON t.object_id = fk.parent_object_id \
    JOIN sys.foreign_key_columns fkc ON fkc.constraint_object_id = fk.object_id \
    JOIN sys.columns pc ON pc.object_id = fkc.parent_object_id AND pc.column_id = fkc.parent_column_id \
    JOIN sys.tables rt ON rt.object_id = fkc.referenced_object_id \
    JOIN sys.columns rc ON rc.object_id = fkc.referenced_object_id AND rc.column_id = fkc.referenced_column_id";
const KEYS_ORDER: &str = " ORDER BY 1, 2, 3, 6";

const SCHEMA_COLUMN: &str = "SCHEMA_NAME(t.schema_id)";

fn rows<'a>(rows: &'a CanyonRows, stage: &'static str) -> Result<&'a [Row], IntrospectError> {
    rows.get_tiberius_rows()
        .map_err(|e| IntrospectError::decode(stage, e))
}

fn text(row: &Row, stage: &'static str, column: &str) -> Result<Option<String>, IntrospectError> {
    row.try_get::<&str, _>(column)
        .map(|v| v.map(str::to_owned))
        .map_err(|e| IntrospectError::decode(stage, format_args!("`{column}`: {e}")))
}

fn required(row: &Row, stage: &'static str, column: &str) -> Result<String, IntrospectError> {
    text(row, stage, column)?.ok_or_else(|| IntrospectError::missing(stage, column))
}

fn int(row: &Row, stage: &'static str, column: &str) -> Result<i32, IntrospectError> {
    row.try_get::<i32, _>(column)
        .map_err(|e| IntrospectError::decode(stage, format_args!("`{column}`: {e}")))?
        .ok_or_else(|| IntrospectError::missing(stage, column))
}

/// `sys.columns` sizes to a native type; `raw` is valid DDL.
pub(super) fn native(type_name: &str, max_length: i32, precision: i32, scale: i32) -> NativeType {
    let name = type_name.to_ascii_lowercase();
    let length = |bytes_per_char: i32| match u32::try_from(max_length / bytes_per_char) {
        Ok(n) if max_length >= 0 => Length::Chars(n),
        _ => Length::Max,
    };
    let (length, digits, raw) = match name.as_str() {
        "nvarchar" | "nchar" => {
            let len = length(2);
            (Some(len), None, sized(&name, len))
        }
        "varchar" | "char" | "varbinary" | "binary" => {
            let len = length(1);
            (Some(len), None, sized(&name, len))
        }
        "time" | "datetime2" | "datetimeoffset" => {
            (None, u8::try_from(scale).ok(), format!("{name}({scale})"))
        }
        "decimal" | "numeric" => (None, None, format!("{name}({precision},{scale})")),
        _ => (None, None, name.clone()),
    };
    NativeType::new(&name, length, digits, &raw)
}

fn sized(name: &str, length: Length) -> String {
    match length {
        Length::Max => format!("{name}(max)"),
        Length::Chars(n) => format!("{name}({n})"),
    }
}

impl Reader for MssqlReader {
    type Dialect = MsSql;

    const VERSION: &'static str = VERSION;
    const SCHEMAS: &'static str = "SELECT name FROM sys.schemas WHERE name IN ";

    fn session(result: &CanyonRows) -> Result<Session, IntrospectError> {
        const S: &str = "version";
        let row = rows(result, S)?
            .first()
            .ok_or_else(|| IntrospectError::missing(S, "row"))?;
        let product = required(row, S, "v")?;
        let server = version::parse_mssql(&product)
            .ok_or_else(|| IntrospectError::decode(S, format_args!("can't parse `{product}`")))?;
        Ok(Session {
            server,
            default_schema: required(row, S, "s")?,
            no_backslash_escapes: false,
        })
    }

    fn columns_query(w: &mut DdlWriter<MsSql>, schemas: &[String]) {
        w.raw(COLUMNS);
        schema_filter(w, SCHEMA_COLUMN, schemas);
        w.raw(OWN_TABLES).raw(COLUMNS_ORDER);
    }

    fn columns(result: &CanyonRows) -> Result<Vec<RawColumn>, IntrospectError> {
        const C: &str = "columns";
        rows(result, C)?
            .iter()
            .map(|row| {
                let default_name = text(row, C, "default_name")?;
                Ok(RawColumn {
                    schema: required(row, C, "schema_name")?,
                    table: required(row, C, "table_name")?,
                    column: required(row, C, "column_name")?,
                    native: native(
                        &required(row, C, "type_name")?,
                        int(row, C, "max_length")?,
                        int(row, C, "num_precision")?,
                        int(row, C, "num_scale")?,
                    ),
                    nullable: int(row, C, "nullable")? == 1,
                    identity: int(row, C, "is_identity")? == 1,
                    default: text(row, C, "default_sql")?.map(|sql| ColumnDefault::Expr {
                        constraint: default_name.as_deref().map(Ident::from_catalog),
                        sql: CatalogSql::new(sql),
                    }),
                    extra: ColumnExtra {
                        collation: text(row, C, "collation")?.map(CatalogSql::new),
                        generated: int(row, C, "is_computed")? == 1,
                        ..ColumnExtra::default()
                    },
                })
            })
            .collect()
    }

    fn keys_query(w: &mut DdlWriter<MsSql>, schemas: &[String]) {
        w.raw(PRIMARY_KEYS);
        schema_filter(w, SCHEMA_COLUMN, schemas);
        w.raw(PRIMARY_KEYS_TAIL).raw(OWN_TABLES).raw(FOREIGN_KEYS);
        schema_filter(w, SCHEMA_COLUMN, schemas);
        w.raw(OWN_TABLES).raw(KEYS_ORDER);
    }

    fn keys(result: &CanyonRows) -> Result<Vec<RawKey>, IntrospectError> {
        const K: &str = "keys";
        rows(result, K)?
            .iter()
            .map(|row| {
                Ok(RawKey {
                    schema: required(row, K, "schema_name")?,
                    table: required(row, K, "table_name")?,
                    name: required(row, K, "name")?,
                    kind: match required(row, K, "kind")?.as_str() {
                        "p" => KeyKind::Primary,
                        _ => KeyKind::Foreign,
                    },
                    column: required(row, K, "column_name")?,
                    ordinal: u32::try_from(int(row, K, "ord")?).unwrap_or(u32::MAX),
                    ref_schema: text(row, K, "ref_schema")?,
                    ref_table: text(row, K, "ref_table")?,
                    ref_column: text(row, K, "ref_column")?,
                })
            })
            .collect()
    }

    fn schema_names(result: &CanyonRows) -> Result<Vec<String>, IntrospectError> {
        rows(result, "schemas")?
            .iter()
            .map(|row| required(row, "schemas", "name"))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_types_from_sys_columns() {
        let n = native("nvarchar", -1, 0, 0);
        assert_eq!(
            (n.length, n.raw.as_str()),
            (Some(Length::Max), "nvarchar(max)")
        );
        let n = native("nvarchar", 40, 0, 0);
        assert_eq!(
            (n.length, n.raw.as_str()),
            (Some(Length::Chars(20)), "nvarchar(20)")
        );
        let n = native("varchar", 30, 0, 0);
        assert_eq!(
            (n.length, n.raw.as_str()),
            (Some(Length::Chars(30)), "varchar(30)")
        );
        let n = native("datetime2", 8, 27, 7);
        assert_eq!((n.precision, n.raw.as_str()), (Some(7), "datetime2(7)"));
        let n = native("time", 4, 12, 3);
        assert_eq!((n.precision, n.raw.as_str()), (Some(3), "time(3)"));
        assert_eq!(native("decimal", 9, 10, 2).raw.as_str(), "decimal(10,2)");
        assert_eq!(native("INT", 4, 10, 0).raw.as_str(), "int");
    }

    #[test]
    fn both_key_halves_are_filtered() {
        let mut w = DdlWriter::<MsSql>::new();
        MssqlReader::keys_query(&mut w, &["dbo".into(), "s1".into()]);
        let st = w.finish();
        assert_eq!(st.sql.matches("SCHEMA_NAME(t.schema_id) IN (@P").count(), 2);
        assert!(st.sql.contains("IN (@P1, @P2)") && st.sql.contains("IN (@P3, @P4)"));
        assert_eq!(st.sql.matches(OWN_TABLES).count(), 2);
        assert!(st.sql.ends_with(KEYS_ORDER));
        assert_eq!(st.params.len(), 4);
    }
}
