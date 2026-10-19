//! MySQL: DDL commits implicitly, so statements run one by one, outside a transaction.
//! A schema is a database.

use super::renamed_constraints;
use super::{DdlDialect, DdlWriter, LowerError, TypeSql};
use super::{actual_column, actual_type, add_foreign_key, desired_type, fk_name, one};
use crate::model::{ColumnDef, ColumnDefault, ForeignKey, Ident, Length, Logical, NativeType};
use crate::model::{QualifiedName, TableDef};
use crate::op::{Ctx, Op, Statement};
use canyon_core::query::querybuilder::syntax::dialect::MySql;
use canyon_core::types::SqlType;

type W = DdlWriter<MySql>;

impl DdlDialect for MySql {
    const TRANSACTIONAL_DDL: bool = false;

    fn type_sql(ty: SqlType) -> Result<TypeSql, LowerError> {
        Ok(TypeSql::new(match ty {
            SqlType::Bool => "tinyint(1)".into(),
            SqlType::SmallInt => "smallint".into(),
            SqlType::Int => "int".into(),
            SqlType::BigInt => "bigint".into(),
            SqlType::Text { max_len: None } => "longtext".into(),
            SqlType::Text { max_len: Some(n) } => format!("varchar({n})"),
            SqlType::Date => "date".into(),
            SqlType::Time { precision } => format!("time({precision})"),
            SqlType::Timestamp { precision } => format!("datetime({precision})"),
            other => return Err(LowerError::UnsupportedType(other)),
        }))
    }

    fn to_logical(native: &NativeType) -> Logical {
        let raw = native.raw.as_str();
        if raw.contains("unsigned") {
            return Logical::Other(native.clone());
        }
        let precision = native.precision.unwrap_or(0); // MySQL's default fsp is 0
        Logical::Known(match (native.name.as_str(), native.length) {
            ("tinyint", _) if raw == "tinyint(1)" => SqlType::Bool,
            ("smallint", _) => SqlType::SmallInt,
            ("int", _) => SqlType::Int,
            ("bigint", _) => SqlType::BigInt,
            ("longtext", _) => SqlType::Text { max_len: None },
            ("varchar", Some(Length::Chars(n))) => SqlType::Text { max_len: Some(n) },
            ("date", _) => SqlType::Date,
            ("time", _) => SqlType::Time { precision },
            ("datetime", _) => SqlType::Timestamp { precision },
            // text/mediumtext can't hold every String; TIMESTAMP converts time zones
            _ => return Logical::Other(native.clone()),
        })
    }

    // Hex: no quote or backslash to escape, so `sql_mode` (NO_BACKSLASH_ESCAPES) can't
    // change how it reads
    fn text_literal(value: &str) -> String {
        use std::fmt::Write;
        let mut out = String::with_capacity(value.len() * 2 + 12);
        out.push_str("_utf8mb4 X'");
        for byte in value.bytes() {
            let _ = write!(out, "{byte:02X}");
        }
        out.push('\'');
        out
    }

    // MySQL wants expression defaults in parentheses
    fn default_sql(w: &mut W, default: &ColumnDefault) {
        match default {
            ColumnDefault::Expr { sql, .. } => w.raw("(").catalog_sql(sql).raw(")"),
            ColumnDefault::Literal { value, .. } => w.literal(value),
        };
    }

    fn lower(op: &Op, ctx: &Ctx<'_>) -> Result<Vec<Statement>, LowerError> {
        match op {
            Op::CreateTable(table) => one(create_table(table)?),
            Op::RenameTable { from, to } => {
                let renamed = QualifiedName {
                    schema: from.schema.clone(),
                    name: to.clone(),
                };
                let mut w = W::new();
                w.raw("RENAME TABLE ")
                    .qualified(from)
                    .raw(" TO ")
                    .qualified(&renamed);
                let mut out = vec![w.finish()];
                // The primary key is always called PRIMARY. Foreign keys can't be renamed:
                // dropped and added again under the new table's name
                let constraints = renamed_constraints(ctx.actual_table(from), from, &renamed);
                for (key, old, new) in constraints.foreign_keys {
                    let mut drop = W::new();
                    drop.raw("ALTER TABLE ")
                        .qualified(&renamed)
                        .raw(" DROP FOREIGN KEY ")
                        .ident(&old);
                    out.push(drop.finish());
                    out.push(add_fk(&renamed, &new, key, ctx).finish());
                }
                Ok(out)
            }
            Op::DropTable(table) => {
                let mut w = W::new();
                w.raw("DROP TABLE ").qualified(table);
                one(w)
            }
            Op::AddColumn { table, column } => {
                let mut w = W::new();
                w.raw("ALTER TABLE ").qualified(table).raw(" ADD COLUMN ");
                column_def(&mut w, column, false)?;
                one(w)
            }
            Op::RenameColumn { table, from, to } => {
                let mut w = W::new();
                w.raw("ALTER TABLE ")
                    .qualified(table)
                    .raw(" RENAME COLUMN ")
                    .ident(from)
                    .raw(" TO ")
                    .ident(to);
                one(w)
            }
            Op::DropColumn { table, column } => {
                let mut w = W::new();
                w.raw("ALTER TABLE ")
                    .qualified(table)
                    .raw(" DROP COLUMN ")
                    .ident(column);
                one(w)
            }
            // Only the type: the column keeps the nullability it has now
            Op::AlterColumnType { table, column } => {
                let (actual_table, actual) = actual_column(ctx, table, &column.name)?;
                let ty = desired_type::<Self>(column)?;
                let identity = actual_table.is_identity(&actual.name);
                one(modify(table, actual, &ty, actual.nullable, identity))
            }
            Op::SetNotNull { table, column } | Op::DropNotNull { table, column } => {
                let (actual_table, actual) = actual_column(ctx, table, column)?;
                let ty = actual_type::<Self>(actual)?;
                let nullable = matches!(op, Op::DropNotNull { .. });
                let identity = actual_table.is_identity(column);
                one(modify(table, actual, &ty, nullable, identity))
            }
            Op::AddIdentity { table, column } | Op::DropIdentity { table, column } => {
                let (_, actual) = actual_column(ctx, table, column)?;
                let ty = actual_type::<Self>(actual)?;
                let identity = matches!(op, Op::AddIdentity { .. });
                one(modify(table, actual, &ty, false, identity))
            }
            Op::AddPrimaryKey { table, key } => {
                let mut w = W::new();
                w.raw("ALTER TABLE ")
                    .qualified(table)
                    .raw(" ADD PRIMARY KEY (")
                    .idents(&key.columns)
                    .raw(")");
                one(w)
            }
            Op::DropPrimaryKey { table, .. } => {
                let mut w = W::new();
                w.raw("ALTER TABLE ")
                    .qualified(table)
                    .raw(" DROP PRIMARY KEY");
                one(w)
            }
            Op::AddForeignKey { table, key } => {
                let name = key
                    .name
                    .clone()
                    .unwrap_or_else(|| fk_name(table, &key.columns));
                one(add_fk(table, &name, key, ctx))
            }
            Op::DropForeignKey { table, name } => {
                let mut w = W::new();
                w.raw("ALTER TABLE ")
                    .qualified(table)
                    .raw(" DROP FOREIGN KEY ")
                    .ident(name);
                one(w)
            }
            Op::RebuildTable { .. } => Err(LowerError::NotForThisDialect(op.kind())),
            Op::CreateSchema { schema } => {
                let mut w = W::new();
                w.raw("CREATE DATABASE IF NOT EXISTS ").ident(schema);
                one(w)
            }
            // Back to the default schema means the connection's own database, by name
            Op::MoveTable { from, to_schema } => {
                let target = QualifiedName {
                    schema: Some(
                        to_schema
                            .clone()
                            .unwrap_or_else(|| Ident::from_catalog(ctx.default_schema)),
                    ),
                    name: from.name.clone(),
                };
                let mut w = W::new();
                w.raw("RENAME TABLE ")
                    .qualified(from)
                    .raw(" TO ")
                    .qualified(&target);
                one(w)
            }
        }
    }
}

// An unqualified REFERENCES points into the child table's database, not the connection's:
// a table in another schema names the default one explicitly
fn add_fk(table: &QualifiedName, name: &Ident, key: &ForeignKey, ctx: &Ctx<'_>) -> W {
    let ref_table = match (&table.schema, &key.ref_table.schema) {
        (Some(_), None) => QualifiedName {
            schema: Some(Ident::from_catalog(ctx.default_schema)),
            name: key.ref_table.name.clone(),
        },
        _ => key.ref_table.clone(),
    };
    add_foreign_key::<MySql>(table, name, key, &ref_table)
}

fn column_def(w: &mut W, column: &ColumnDef, identity: bool) -> Result<(), LowerError> {
    let ty = desired_type::<MySql>(column)?;
    w.ident(&column.name)
        .raw(" ")
        .ty(&ty)
        .raw(if column.nullable {
            " NULL"
        } else {
            " NOT NULL"
        });
    if identity {
        w.raw(" AUTO_INCREMENT");
    }
    Ok(())
}

// MODIFY restates the whole column: the default and AUTO_INCREMENT are written again (an
// AUTO_INCREMENT column can't have a default). COMMENT, ON UPDATE and COLLATE aren't read
// from the catalog yet, so MODIFY drops them
fn modify(
    table: &QualifiedName,
    column: &ColumnDef,
    ty: &TypeSql,
    nullable: bool,
    identity: bool,
) -> W {
    let mut w = W::new();
    w.raw("ALTER TABLE ")
        .qualified(table)
        .raw(" MODIFY COLUMN ")
        .ident(&column.name)
        .raw(" ")
        .ty(ty)
        .raw(if nullable { " NULL" } else { " NOT NULL" });
    if identity {
        w.raw(" AUTO_INCREMENT");
    } else if let Some(default) = &column.default {
        w.raw(" DEFAULT ");
        MySql::default_sql(&mut w, default);
    }
    w
}

fn create_table(table: &TableDef) -> Result<W, LowerError> {
    let mut w = W::new();
    w.raw("CREATE TABLE ").qualified(&table.name).raw(" (");
    for (i, column) in table.columns.iter().enumerate() {
        if i > 0 {
            w.raw(", ");
        }
        column_def(&mut w, column, table.is_identity(&column.name))?;
    }
    if let Some(pk) = &table.primary_key {
        w.raw(", PRIMARY KEY (").idents(&pk.columns).raw(")");
    }
    w.raw(")");
    Ok(w)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialect::fixtures::*;
    use crate::model::SchemaModel;

    fn actual(default: Option<ColumnDefault>) -> SchemaModel {
        SchemaModel::from_tables(vec![actual_player("int", "varchar(100)", default)]).unwrap()
    }

    #[test]
    fn lowering_snapshots() {
        for (name, op) in all_ops() {
            insta::assert_snapshot!(
                format!("mysql_{name}"),
                render::<MySql>(&op, &actual(None), "canyon")
            );
        }
        insta::assert_snapshot!(
            "mysql_rebuild_table",
            render::<MySql>(&rebuild_op(), &actual(None), "canyon")
        );
    }

    #[test]
    fn edge_case_snapshots() {
        for (name, op, actual) in edge_cases("int", "varchar(100)") {
            insta::assert_snapshot!(
                format!("mysql_edge_{name}"),
                render::<MySql>(&op, &actual, "canyon")
            );
        }
    }

    #[test]
    fn modify_restates_defaults() {
        for (label, default) in [("expr", expr("now()")), ("literal", literal("it's"))] {
            for (name, op) in all_ops() {
                if matches!(name, "alter_column_type" | "set_not_null" | "drop_not_null") {
                    insta::assert_snapshot!(
                        format!("mysql_{name}_with_{label}_default"),
                        render::<MySql>(&op, &actual(Some(default.clone())), "canyon")
                    );
                }
            }
        }
    }

    #[test]
    fn text_literals_are_hex() {
        assert_eq!(MySql::text_literal("a'\\"), "_utf8mb4 X'61275C'");
        assert_eq!(MySql::text_literal(""), "_utf8mb4 X''");
    }

    #[test]
    fn types_both_ways() {
        use canyon_core::types::SqlType::*;
        for (ty, sql, native) in [
            (Bool, "tinyint(1)", n("tinyint", "tinyint(1)", None, None)),
            (SmallInt, "smallint", n("smallint", "smallint", None, None)),
            (Int, "int", n("int", "int", None, None)),
            (BigInt, "bigint", n("bigint", "bigint", None, None)),
            (
                Text { max_len: None },
                "longtext",
                n("longtext", "longtext", None, None),
            ),
            (
                Text { max_len: Some(100) },
                "varchar(100)",
                n("varchar", "varchar(100)", Some(Length::Chars(100)), None),
            ),
            (Date, "date", n("date", "date", None, None)),
            (
                Time { precision: 6 },
                "time(6)",
                n("time", "time(6)", None, Some(6)),
            ),
            (
                Timestamp { precision: 6 },
                "datetime(6)",
                n("datetime", "datetime(6)", None, Some(6)),
            ),
        ] {
            assert_eq!(MySql::type_sql(ty).unwrap().as_str(), sql);
            assert_eq!(MySql::to_logical(&native), Logical::Known(ty), "{native:?}");
        }
        for (name, raw) in [
            ("tinyint", "tinyint"),
            ("tinyint", "tinyint(4)"),
            ("int", "int unsigned"),
            ("text", "text"),
            ("mediumtext", "mediumtext"),
            ("timestamp", "timestamp(6)"),
            ("decimal", "decimal(10,2)"),
        ] {
            assert!(
                matches!(
                    MySql::to_logical(&n(name, raw, None, None)),
                    Logical::Other(_)
                ),
                "{raw}"
            );
        }
    }

    fn n(name: &str, raw: &str, length: Option<Length>, precision: Option<u8>) -> NativeType {
        NativeType::new(name, length, precision, raw)
    }
}
