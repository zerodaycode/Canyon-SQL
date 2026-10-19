//! PostgreSQL catalog: `pg_attribute` / `pg_constraint`.
//!
//! Every function is called as `pg_catalog.<name>`: an unqualified call can resolve to a
//! function another role created on the search path.

use super::{IntrospectError, KeyKind, RawColumn, RawKey, Reader, Session};
use super::{schema_filter, version};
use crate::dialect::DdlWriter;
use crate::model::{CatalogSql, ColumnDefault, ColumnExtra, Length, NativeType};
use canyon_core::query::querybuilder::syntax::dialect::PgDialect;
use canyon_core::rows::CanyonRows;
use tokio_postgres::Row;

pub(super) struct PgReader;

// `SHOW server_version_num` can't be selected next to another column
const VERSION: &str = "SELECT pg_catalog.current_setting('server_version_num') AS v, \
    pg_catalog.current_schema()::text AS s";

// Generated columns keep their expression in pg_attrdef too: it isn't a default
const COLUMNS: &str = "SELECT n.nspname::text AS schema_name, c.relname::text AS table_name, \
    a.attname::text AS column_name, t.typname::text AS type_name, \
    pg_catalog.format_type(a.atttypid, a.atttypmod) AS raw_type, a.atttypmod AS typmod, \
    NOT a.attnotnull AS nullable, a.attidentity::text AS identity, \
    a.attgenerated::text AS generated, \
    pg_catalog.pg_get_expr(d.adbin, d.adrelid) AS default_sql, \
    CASE WHEN a.attcollation <> t.typcollation \
         THEN pg_catalog.quote_ident(cn.nspname::text) || '.' \
              || pg_catalog.quote_ident(co.collname::text) END AS collation \
    FROM pg_catalog.pg_attribute a \
    JOIN pg_catalog.pg_class c ON c.oid = a.attrelid \
    JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace \
    JOIN pg_catalog.pg_type t ON t.oid = a.atttypid \
    LEFT JOIN pg_catalog.pg_attrdef d \
      ON d.adrelid = a.attrelid AND d.adnum = a.attnum AND a.attgenerated = '' \
    LEFT JOIN pg_catalog.pg_collation co ON co.oid = a.attcollation \
    LEFT JOIN pg_catalog.pg_namespace cn ON cn.oid = co.collnamespace";
// A partitioned table is one table: its partitions are left out
const COLUMNS_TAIL: &str = " AND c.relkind IN ('r', 'p') AND NOT c.relispartition \
    AND a.attnum > 0 AND NOT a.attisdropped ORDER BY 1, 2, a.attnum";

const KEYS: &str = "SELECT n.nspname::text AS schema_name, c.relname::text AS table_name, \
    con.conname::text AS name, con.contype::text AS kind, \
    ARRAY(SELECT a.attname::text FROM pg_catalog.unnest(con.conkey) WITH ORDINALITY k(attnum, ord) \
          JOIN pg_catalog.pg_attribute a ON a.attrelid = con.conrelid AND a.attnum = k.attnum \
          ORDER BY k.ord) AS columns, \
    rn.nspname::text AS ref_schema, rc.relname::text AS ref_table, \
    ARRAY(SELECT a.attname::text FROM pg_catalog.unnest(con.confkey) WITH ORDINALITY k(attnum, ord) \
          JOIN pg_catalog.pg_attribute a ON a.attrelid = con.confrelid AND a.attnum = k.attnum \
          ORDER BY k.ord) AS ref_columns \
    FROM pg_catalog.pg_constraint con \
    JOIN pg_catalog.pg_class c ON c.oid = con.conrelid \
    JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace \
    LEFT JOIN pg_catalog.pg_class rc ON rc.oid = con.confrelid \
    LEFT JOIN pg_catalog.pg_namespace rn ON rn.oid = rc.relnamespace";
const KEYS_TAIL: &str = " AND con.contype IN ('p', 'f') AND NOT c.relispartition \
    ORDER BY 1, 2, 3";

const SCHEMA_COLUMN: &str = "n.nspname::text";

fn rows<'a>(rows: &'a CanyonRows, stage: &'static str) -> Result<&'a [Row], IntrospectError> {
    rows.get_postgres_rows()
        .map_err(|e| IntrospectError::decode(stage, e))
}

fn get<'a, T: tokio_postgres::types::FromSql<'a>>(
    row: &'a Row,
    stage: &'static str,
    column: &str,
) -> Result<T, IntrospectError> {
    row.try_get(column)
        .map_err(|e| IntrospectError::decode(stage, format_args!("`{column}`: {e}")))
}

/// Length and fractional digits from `atttypmod`.
fn native(type_name: String, typmod: i32, raw: &str) -> NativeType {
    let (length, precision) = match type_name.as_str() {
        "varchar" | "bpchar" if typmod >= 4 => {
            (u32::try_from(typmod - 4).ok().map(Length::Chars), None)
        }
        "time" | "timetz" | "timestamp" | "timestamptz" if typmod >= 0 => {
            (None, u8::try_from(typmod).ok())
        }
        _ => (None, None),
    };
    NativeType::new(&type_name, length, precision, raw)
}

impl Reader for PgReader {
    type Dialect = PgDialect;

    const VERSION: &'static str = VERSION;
    const SCHEMAS: &'static str =
        "SELECT nspname::text AS name FROM pg_catalog.pg_namespace WHERE nspname::text IN ";

    fn session(result: &CanyonRows) -> Result<Session, IntrospectError> {
        const S: &str = "version";
        let row = rows(result, S)?
            .first()
            .ok_or_else(|| IntrospectError::missing(S, "row"))?;
        let version_num: String = get(row, S, "v")?;
        let default_schema = get::<Option<String>>(row, S, "s")?
            .ok_or_else(|| IntrospectError::decode(S, "no current schema (empty search_path)"))?;
        let server = version::parse_pg(&version_num).ok_or_else(|| {
            IntrospectError::decode(S, format_args!("can't parse `{version_num}`"))
        })?;
        Ok(Session {
            server,
            default_schema,
            no_backslash_escapes: false,
        })
    }

    fn columns_query(w: &mut DdlWriter<PgDialect>, schemas: &[String]) {
        w.raw(COLUMNS);
        schema_filter(w, SCHEMA_COLUMN, schemas);
        w.raw(COLUMNS_TAIL);
    }

    fn columns(result: &CanyonRows) -> Result<Vec<RawColumn>, IntrospectError> {
        const C: &str = "columns";
        rows(result, C)?
            .iter()
            .map(|row| {
                let identity: String = get(row, C, "identity")?;
                let generated: String = get(row, C, "generated")?;
                let raw_type: String = get(row, C, "raw_type")?;
                Ok(RawColumn {
                    schema: get(row, C, "schema_name")?,
                    table: get(row, C, "table_name")?,
                    column: get(row, C, "column_name")?,
                    native: native(get(row, C, "type_name")?, get(row, C, "typmod")?, &raw_type),
                    nullable: get(row, C, "nullable")?,
                    identity: !identity.is_empty(),
                    default: get::<Option<String>>(row, C, "default_sql")?.map(|sql| {
                        ColumnDefault::Expr {
                            constraint: None,
                            sql: CatalogSql::new(sql),
                        }
                    }),
                    extra: ColumnExtra {
                        collation: get::<Option<String>>(row, C, "collation")?.map(CatalogSql::new),
                        generated: !generated.is_empty(),
                        ..ColumnExtra::default()
                    },
                })
            })
            .collect()
    }

    fn keys_query(w: &mut DdlWriter<PgDialect>, schemas: &[String]) {
        w.raw(KEYS);
        schema_filter(w, SCHEMA_COLUMN, schemas);
        w.raw(KEYS_TAIL);
    }

    fn keys(result: &CanyonRows) -> Result<Vec<RawKey>, IntrospectError> {
        const K: &str = "keys";
        let mut keys = Vec::new();
        for row in rows(result, K)? {
            let schema: String = get(row, K, "schema_name")?;
            let table: String = get(row, K, "table_name")?;
            let name: String = get(row, K, "name")?;
            let kind = match get::<String>(row, K, "kind")?.as_str() {
                "p" => KeyKind::Primary,
                _ => KeyKind::Foreign,
            };
            let ref_schema: Option<String> = get(row, K, "ref_schema")?;
            let ref_table: Option<String> = get(row, K, "ref_table")?;
            let ref_columns: Vec<String> = get(row, K, "ref_columns")?;
            let key_columns: Vec<String> = get(row, K, "columns")?;
            for (i, column) in key_columns.into_iter().enumerate() {
                keys.push(RawKey {
                    schema: schema.clone(),
                    table: table.clone(),
                    name: name.clone(),
                    kind,
                    column,
                    ordinal: u32::try_from(i + 1).unwrap_or(u32::MAX),
                    ref_schema: ref_schema.clone(),
                    ref_table: ref_table.clone(),
                    ref_column: ref_columns.get(i).cloned(),
                });
            }
        }
        Ok(keys)
    }

    fn schema_names(result: &CanyonRows) -> Result<Vec<String>, IntrospectError> {
        rows(result, "schemas")?
            .iter()
            .map(|row| get(row, "schemas", "name"))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_types_from_typmod() {
        let n = native("varchar".into(), 24, "character varying(20)");
        assert_eq!(
            (n.length, n.raw.as_str()),
            (Some(Length::Chars(20)), "character varying(20)")
        );
        assert_eq!(
            native("varchar".into(), -1, "character varying").length,
            None
        );
        assert_eq!(
            native("timestamp".into(), 3, "timestamp(3) without time zone").precision,
            Some(3)
        );
        assert_eq!(
            native("timestamp".into(), -1, "timestamp without time zone").precision,
            None
        );
        assert_eq!(native("int4".into(), -1, "integer").name, "int4");
    }

    #[test]
    fn schemas_are_bound() {
        let mut w = DdlWriter::<PgDialect>::new();
        PgReader::columns_query(&mut w, &["public".into(), "s1".into()]);
        let st = w.finish();
        assert!(
            st.sql
                .contains(" WHERE n.nspname::text IN ($1, $2) AND c.relkind")
        );
        assert_eq!(st.params.len(), 2);
    }

    /// Every function call goes through `pg_catalog.`.
    #[test]
    fn functions_are_qualified() {
        for sql in [VERSION, COLUMNS, KEYS] {
            for function in [
                "current_setting(",
                "current_schema(",
                "format_type(",
                "pg_get_expr(",
                "quote_ident(",
                "unnest(",
            ] {
                for (at, _) in sql.match_indices(function) {
                    assert!(
                        sql[..at].ends_with("pg_catalog."),
                        "unqualified {function} in {sql}"
                    );
                }
            }
        }
    }
}
