//! MySQL catalog: `information_schema`. A schema is a database; the default one is the
//! connection's.

use super::{IntrospectError, KeyKind, RawColumn, RawKey, Reader, Session};
use super::{schema_filter, version};
use crate::dialect::DdlWriter;
use crate::dialect::mysql::is_bit_value;
use crate::model::{CatalogSql, ColumnDefault, ColumnExtra, Length, NativeType};
use canyon_core::query::querybuilder::syntax::dialect::MySql;
use canyon_core::rows::CanyonRows;
use mysql_async::Row;
use mysql_async::prelude::FromValue;

pub(super) struct MysqlReader;

const VERSION: &str = "SELECT VERSION(), DATABASE(), \
    FIND_IN_SET('NO_BACKSLASH_ESCAPES', @@SESSION.sql_mode) > 0";

const COLUMNS: &str = "SELECT c.TABLE_SCHEMA, c.TABLE_NAME, c.COLUMN_NAME, c.DATA_TYPE, \
    c.COLUMN_TYPE, c.CHARACTER_MAXIMUM_LENGTH, c.DATETIME_PRECISION, c.IS_NULLABLE, c.EXTRA, \
    c.COLUMN_DEFAULT, c.COLLATION_NAME, c.COLUMN_COMMENT \
    FROM information_schema.COLUMNS c \
    JOIN information_schema.TABLES t ON t.TABLE_SCHEMA = c.TABLE_SCHEMA AND t.TABLE_NAME = c.TABLE_NAME";
const COLUMNS_TAIL: &str =
    " AND t.TABLE_TYPE = 'BASE TABLE' ORDER BY c.TABLE_SCHEMA, c.TABLE_NAME, c.ORDINAL_POSITION";

const KEYS: &str = "SELECT k.TABLE_SCHEMA, k.TABLE_NAME, k.CONSTRAINT_NAME, tc.CONSTRAINT_TYPE, \
    k.COLUMN_NAME, k.ORDINAL_POSITION, k.REFERENCED_TABLE_SCHEMA, k.REFERENCED_TABLE_NAME, \
    k.REFERENCED_COLUMN_NAME \
    FROM information_schema.KEY_COLUMN_USAGE k \
    JOIN information_schema.TABLE_CONSTRAINTS tc ON tc.CONSTRAINT_SCHEMA = k.CONSTRAINT_SCHEMA \
      AND tc.TABLE_SCHEMA = k.TABLE_SCHEMA AND tc.TABLE_NAME = k.TABLE_NAME \
      AND tc.CONSTRAINT_NAME = k.CONSTRAINT_NAME";
// Constraints are joined by name: a UNIQUE key may share its name with a foreign key, so a
// foreign key only takes the rows that reference something
const KEYS_TAIL: &str = " AND tc.CONSTRAINT_TYPE IN ('PRIMARY KEY', 'FOREIGN KEY') \
    AND (tc.CONSTRAINT_TYPE = 'FOREIGN KEY') = (k.REFERENCED_TABLE_NAME IS NOT NULL) \
    ORDER BY k.TABLE_SCHEMA, k.TABLE_NAME, k.CONSTRAINT_NAME, k.ORDINAL_POSITION";

fn rows<'a>(rows: &'a CanyonRows, stage: &'static str) -> Result<&'a [Row], IntrospectError> {
    rows.get_mysql_rows()
        .map_err(|e| IntrospectError::decode(stage, e))
}

/// A nullable value by position; `Row::get` would panic on a value it can't convert.
fn opt<T: FromValue>(
    row: &Row,
    stage: &'static str,
    i: usize,
) -> Result<Option<T>, IntrospectError> {
    match row.get_opt::<Option<T>, _>(i) {
        Some(Ok(value)) => Ok(value),
        Some(Err(e)) => Err(IntrospectError::decode(
            stage,
            format_args!("column {i}: {e}"),
        )),
        None => Err(IntrospectError::missing(stage, &format!("column {i}"))),
    }
}

fn required<T: FromValue>(row: &Row, stage: &'static str, i: usize) -> Result<T, IntrospectError> {
    opt(row, stage, i)?.ok_or_else(|| IntrospectError::missing(stage, &format!("column {i}")))
}

/// `COLUMN_DEFAULT` is SQL only when `EXTRA` says `DEFAULT_GENERATED`, or for a `bit`
/// value (`b'1'`); otherwise it's the value itself, unquoted.
fn default_of(
    value: Option<String>,
    extra: &str,
    data_type: &str,
) -> Result<Option<ColumnDefault>, IntrospectError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let bit_value = data_type.eq_ignore_ascii_case("bit") && is_bit_value(&value);
    Ok(Some(if extra.contains("DEFAULT_GENERATED") {
        ColumnDefault::Expr {
            constraint: None,
            sql: expression(value)?,
        }
    } else if bit_value {
        ColumnDefault::Expr {
            constraint: None,
            sql: CatalogSql::new(value),
        }
    } else {
        ColumnDefault::Literal {
            constraint: None,
            value,
        }
    }))
}

/// An expression default as SQL. MySQL 8 shows its quotes backslash-escaped
/// (`concat(_utf8mb4\'a\')`): undone only when that's there, and the result has to end
/// outside any quoted text, or text from inside a string would become SQL.
fn expression(value: String) -> Result<CatalogSql, IntrospectError> {
    let sql = if value.contains("\\'") {
        let mut out = String::with_capacity(value.len());
        let mut chars = value.chars();
        while let Some(c) = chars.next() {
            out.push(if c == '\\' {
                chars.next().unwrap_or(c)
            } else {
                c
            });
        }
        out
    } else {
        value
    };
    if quotes_balanced(&sql) {
        Ok(CatalogSql::new(sql))
    } else {
        Err(IntrospectError::decode(
            "columns",
            format_args!("can't read the default `{sql}` back as SQL"),
        ))
    }
}

/// Every `'…'`, `"…"` and `` `…` `` closes (doubled quotes and backslash escapes inside).
fn quotes_balanced(sql: &str) -> bool {
    let mut open: Option<char> = None;
    let mut chars = sql.chars().peekable();
    while let Some(c) = chars.next() {
        match open {
            None if matches!(c, '\'' | '"' | '`') => open = Some(c),
            None => {}
            Some(q) if c == '\\' && q != '`' => {
                chars.next();
            }
            Some(q) if c == q => {
                if chars.peek() == Some(&q) {
                    chars.next();
                } else {
                    open = None;
                }
            }
            Some(_) => {}
        }
    }
    open.is_none()
}

/// `on update CURRENT_TIMESTAMP(6)` in `EXTRA`.
fn on_update(extra: &str) -> Option<CatalogSql> {
    const MARK: &str = "on update ";
    let at = extra.to_ascii_lowercase().find(MARK)?;
    extra[at + MARK.len()..]
        .split_whitespace()
        .next()
        .map(CatalogSql::new)
}

impl Reader for MysqlReader {
    type Dialect = MySql;

    const VERSION: &'static str = VERSION;
    const SCHEMAS: &'static str =
        "SELECT SCHEMA_NAME FROM information_schema.SCHEMATA WHERE SCHEMA_NAME IN ";

    fn session(result: &CanyonRows) -> Result<Session, IntrospectError> {
        const S: &str = "version";
        let row = rows(result, S)?
            .first()
            .ok_or_else(|| IntrospectError::missing(S, "row"))?;
        let version_text: String = required(row, S, 0)?;
        let default_schema: String = opt(row, S, 1)?
            .ok_or_else(|| IntrospectError::decode(S, "the connection has no database selected"))?;
        let server = version::parse_mysql(&version_text).ok_or_else(|| {
            IntrospectError::decode(S, format_args!("can't parse `{version_text}`"))
        })?;
        Ok(Session {
            server,
            default_schema,
            no_backslash_escapes: required::<i64>(row, S, 2)? == 1,
        })
    }

    fn columns_query(w: &mut DdlWriter<MySql>, schemas: &[String]) {
        w.raw(COLUMNS);
        schema_filter(w, "c.TABLE_SCHEMA", schemas);
        w.raw(COLUMNS_TAIL);
    }

    fn columns(result: &CanyonRows) -> Result<Vec<RawColumn>, IntrospectError> {
        const C: &str = "columns";
        rows(result, C)?
            .iter()
            .map(|row| {
                let data_type: String = required(row, C, 3)?;
                let column_type: String = required(row, C, 4)?;
                let length = opt::<u64>(row, C, 5)?
                    .and_then(|n| u32::try_from(n).ok())
                    .map(Length::Chars);
                let precision = opt::<u64>(row, C, 6)?.and_then(|n| u8::try_from(n).ok());
                let extra: String = opt(row, C, 8)?.unwrap_or_default();
                Ok(RawColumn {
                    schema: required(row, C, 0)?,
                    table: required(row, C, 1)?,
                    column: required(row, C, 2)?,
                    native: NativeType::new(&data_type, length, precision, &column_type),
                    nullable: required::<String>(row, C, 7)? == "YES",
                    identity: extra.contains("auto_increment"),
                    default: default_of(opt(row, C, 9)?, &extra, &data_type)?,
                    extra: ColumnExtra {
                        collation: opt::<String>(row, C, 10)?.map(CatalogSql::new),
                        comment: opt::<String>(row, C, 11)?.filter(|c| !c.is_empty()),
                        on_update: on_update(&extra),
                        generated: extra.contains("VIRTUAL GENERATED")
                            || extra.contains("STORED GENERATED"),
                        invisible: extra.contains("INVISIBLE"),
                    },
                })
            })
            .collect()
    }

    fn keys_query(w: &mut DdlWriter<MySql>, schemas: &[String]) {
        w.raw(KEYS);
        schema_filter(w, "k.TABLE_SCHEMA", schemas);
        w.raw(KEYS_TAIL);
    }

    fn keys(result: &CanyonRows) -> Result<Vec<RawKey>, IntrospectError> {
        const K: &str = "keys";
        rows(result, K)?
            .iter()
            .map(|row| {
                Ok(RawKey {
                    schema: required(row, K, 0)?,
                    table: required(row, K, 1)?,
                    name: required(row, K, 2)?,
                    kind: match required::<String>(row, K, 3)?.as_str() {
                        "PRIMARY KEY" => KeyKind::Primary,
                        _ => KeyKind::Foreign,
                    },
                    column: required(row, K, 4)?,
                    ordinal: u32::try_from(required::<u64>(row, K, 5)?).unwrap_or(u32::MAX),
                    ref_schema: opt(row, K, 6)?,
                    ref_table: opt(row, K, 7)?,
                    ref_column: opt(row, K, 8)?,
                })
            })
            .collect()
    }

    fn schema_names(result: &CanyonRows) -> Result<Vec<String>, IntrospectError> {
        rows(result, "schemas")?
            .iter()
            .map(|row| required(row, "schemas", 0))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default(value: &str, extra: &str, data_type: &str) -> ColumnDefault {
        default_of(Some(value.into()), extra, data_type)
            .unwrap()
            .unwrap()
    }

    fn literal(value: &str) -> ColumnDefault {
        ColumnDefault::Literal {
            constraint: None,
            value: value.into(),
        }
    }

    fn expr(sql: &str) -> ColumnDefault {
        ColumnDefault::Expr {
            constraint: None,
            sql: CatalogSql::new(sql),
        }
    }

    #[test]
    fn defaults_are_values_unless_generated() {
        assert_eq!(default("x'y", "", "varchar"), literal("x'y"));
        assert_eq!(default("1", "", "tinyint"), literal("1"));
        assert_eq!(
            default(
                "CURRENT_TIMESTAMP(6)",
                "DEFAULT_GENERATED on update CURRENT_TIMESTAMP(6)",
                "datetime"
            ),
            expr("CURRENT_TIMESTAMP(6)")
        );
        assert_eq!(default_of(None, "auto_increment", "int").unwrap(), None);
    }

    #[test]
    fn bit_values_are_sql_by_type_only() {
        assert_eq!(default("b'1'", "", "bit"), expr("b'1'"));
        assert_eq!(default("b'1'", "", "varchar"), literal("b'1'"));
        let sneaky = "b'1'); DROP TABLE x; -- ";
        assert_eq!(default(sneaky, "", "bit"), literal(sneaky));
        assert_eq!(default(sneaky, "", "varchar"), literal(sneaky));
    }

    #[test]
    fn escaped_expressions_are_unescaped() {
        // As MySQL shows `DEFAULT (concat('a','b'))` and `DEFAULT ('it''s')`
        assert_eq!(
            default(
                r"concat(_utf8mb4\'a\',_utf8mb4\'b\')",
                "DEFAULT_GENERATED",
                "varchar"
            ),
            expr("concat(_utf8mb4'a',_utf8mb4'b')")
        );
        assert_eq!(
            default(r"_utf8mb4\'it\\\'s\'", "DEFAULT_GENERATED", "varchar"),
            expr(r"_utf8mb4'it\'s'")
        );
        assert_eq!(
            default("(`n` * 2)", "DEFAULT_GENERATED", "int"),
            expr("(`n` * 2)")
        );
    }

    #[test]
    fn expressions_that_would_end_a_string_early_are_refused() {
        // Not escaped: kept as it is
        assert_eq!(
            default("concat('a','b')", "DEFAULT_GENERATED", "varchar"),
            expr("concat('a','b')")
        );
        // Unescaping these would leave a quote open, so text from inside a string would
        // become SQL: refused
        for value in [r"_utf8mb4'a\'b'", r"concat(\'a)"] {
            assert!(
                default_of(Some(value.into()), "DEFAULT_GENERATED", "varchar").is_err(),
                "{value}"
            );
        }
        assert!(quotes_balanced("'a''b' \"c\" `d`"));
        assert!(!quotes_balanced("'a"));
    }

    #[test]
    fn on_update_comes_from_extra() {
        let read = |extra: &str| on_update(extra).map(|s| s.as_str().to_owned());
        assert_eq!(
            read("DEFAULT_GENERATED on update CURRENT_TIMESTAMP(6)").as_deref(),
            Some("CURRENT_TIMESTAMP(6)")
        );
        assert_eq!(
            read("on update CURRENT_TIMESTAMP INVISIBLE").as_deref(),
            Some("CURRENT_TIMESTAMP")
        );
        assert_eq!(read("DEFAULT_GENERATED"), None);
        assert_eq!(read(""), None);
    }

    #[test]
    fn schemas_are_bound() {
        let mut w = DdlWriter::<MySql>::new();
        MysqlReader::columns_query(&mut w, &["db".into(), "db_s1".into()]);
        let st = w.finish();
        assert!(
            st.sql
                .contains(" WHERE c.TABLE_SCHEMA IN (?, ?) AND t.TABLE_TYPE")
        );
        assert_eq!(st.params.len(), 2);
    }
}
