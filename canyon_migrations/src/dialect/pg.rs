//! PostgreSQL: one transaction per plan, identity with `GENERATED ALWAYS`.

use super::{DdlDialect, DdlWriter, LowerError, TypeSql};
use super::{add_foreign_key, add_primary_key, desired_type, drop_constraint, fk_name, one};
use super::{known, pk_name, renamed_constraints, restated};
use crate::model::{ColumnDef, Ident, Length, Logical, NativeType, QualifiedName, TableDef};
use crate::op::{Ctx, Op, Param, Statement};
use canyon_core::query::querybuilder::syntax::dialect::PgDialect;
use canyon_core::types::SqlType;

type W = DdlWriter<PgDialect>;

impl DdlDialect for PgDialect {
    const TRANSACTIONAL_DDL: bool = true;

    fn type_sql(ty: SqlType) -> Result<TypeSql, LowerError> {
        Ok(TypeSql::new(match ty {
            SqlType::Bool => "boolean".into(),
            SqlType::SmallInt => "smallint".into(),
            SqlType::Int => "integer".into(),
            SqlType::BigInt => "bigint".into(),
            SqlType::Text { max_len: None } => "text".into(),
            SqlType::Text { max_len: Some(n) } => format!("varchar({n})"),
            SqlType::Date => "date".into(),
            SqlType::Time { precision } => format!("time({precision})"),
            SqlType::Timestamp { precision } => format!("timestamp({precision})"),
            other => return Err(LowerError::UnsupportedType(other)),
        }))
    }

    fn to_logical(native: &NativeType) -> Logical {
        let precision = native.precision.unwrap_or(SqlType::DEFAULT_PRECISION);
        Logical::Known(match (native.name.as_str(), native.length) {
            ("bool", _) => SqlType::Bool,
            ("int2", _) => SqlType::SmallInt,
            ("int4", _) => SqlType::Int,
            ("int8", _) => SqlType::BigInt,
            ("text", _) | ("varchar", None | Some(Length::Max)) => SqlType::Text { max_len: None },
            ("varchar", Some(Length::Chars(n))) => SqlType::Text { max_len: Some(n) },
            ("date", _) => SqlType::Date,
            ("time", _) => SqlType::Time { precision },
            ("timestamp", _) => SqlType::Timestamp { precision },
            _ => return Logical::Other(native.clone()),
        })
    }

    // E'…' reads backslashes the same whatever `standard_conforming_strings` says
    fn text_literal(value: &str) -> String {
        format!("E'{}'", value.replace('\\', "\\\\").replace('\'', "''"))
    }

    fn lower(op: &Op, ctx: &Ctx<'_>) -> Result<Vec<Statement>, LowerError> {
        match op {
            Op::CreateTable(table) => one(create_table(table)?),
            Op::RenameTable { from, to } => {
                let renamed = QualifiedName {
                    schema: from.schema.clone(),
                    name: to.clone(),
                };
                let mut table = W::new();
                table
                    .raw("ALTER TABLE ")
                    .qualified(from)
                    .raw(" RENAME TO ")
                    .ident(to);
                let mut out = vec![table.finish()];
                // Canyon names constraints after the table: rename them too, or a new
                // table with the old name collides with them
                let constraints = renamed_constraints(ctx.actual_table(from), from, &renamed);
                let pairs = constraints.primary_key.into_iter().chain(
                    constraints
                        .foreign_keys
                        .into_iter()
                        .map(|(_, old, new)| (old, new)),
                );
                for (old, new) in pairs {
                    let mut w = W::new();
                    w.raw("ALTER TABLE ")
                        .qualified(&renamed)
                        .raw(" RENAME CONSTRAINT ")
                        .ident(&old)
                        .raw(" TO ")
                        .ident(&new);
                    out.push(w.finish());
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
            // Only the type: nullability is SetNotNull / DropNotNull. Without COLLATE the
            // column gets the new type's default collation
            Op::AlterColumnType { table, column } => {
                let ty = desired_type::<Self>(column)?;
                let mut w = W::new();
                w.raw("ALTER TABLE ")
                    .qualified(table)
                    .raw(" ALTER COLUMN ")
                    .ident(&column.name)
                    .raw(" TYPE ")
                    .ty(&ty);
                if let Some(actual) = ctx.actual_column(table, &column.name)
                    && let Some(collation) = restated(actual, known(column)?).extra.collation
                {
                    w.raw(" COLLATE ").catalog_sql(&collation);
                }
                w.raw(" USING ").ident(&column.name).raw("::").ty(&ty);
                one(w)
            }
            Op::SetNotNull { table, column } | Op::DropNotNull { table, column } => {
                let mut w = W::new();
                w.raw("ALTER TABLE ")
                    .qualified(table)
                    .raw(" ALTER COLUMN ")
                    .ident(column)
                    .raw(if matches!(op, Op::SetNotNull { .. }) {
                        " SET NOT NULL"
                    } else {
                        " DROP NOT NULL"
                    });
                one(w)
            }
            Op::AddPrimaryKey { table, key } => one(add_primary_key::<Self>(table, key)),
            Op::DropPrimaryKey { table, name } | Op::DropForeignKey { table, name } => {
                one(drop_constraint::<Self>(table, name))
            }
            Op::AddIdentity { table, column } => {
                let mut out = Vec::new();
                // An identity can't be added over a default (a hand-made `serial`)
                if ctx
                    .actual_column(table, column)
                    .is_some_and(|c| c.default.is_some())
                {
                    let mut w = W::new();
                    w.raw("ALTER TABLE ")
                        .qualified(table)
                        .raw(" ALTER COLUMN ")
                        .ident(column)
                        .raw(" DROP DEFAULT");
                    out.push(w.finish());
                }
                let mut add = W::new();
                add.raw("ALTER TABLE ")
                    .qualified(table)
                    .raw(" ALTER COLUMN ")
                    .ident(column)
                    .raw(" ADD GENERATED ALWAYS AS IDENTITY");
                out.push(add.finish());
                // The sequence starts at 1: move it past the rows already there. Found by its
                // internal dependency: pg_get_serial_sequence can return an old `serial`
                // sequence the column still owns
                let mut quoted_table = W::new();
                quoted_table.qualified(table);
                let mut resync = W::new();
                resync
                    .raw("SELECT setval(d.objid::regclass, COALESCE((SELECT MAX(")
                    .ident(column)
                    .raw(") FROM ")
                    .qualified(table)
                    .raw("), 0) + 1, false) FROM pg_depend d JOIN pg_attribute a")
                    .raw(" ON a.attrelid = d.refobjid AND a.attnum = d.refobjsubid")
                    .raw(" WHERE d.classid = 'pg_class'::regclass AND d.refobjid = ")
                    .param(Param::Text(quoted_table.into_sql()))
                    .raw("::text::regclass AND a.attname::text = ")
                    .param(Param::Text(column.to_string()))
                    .raw("::text AND d.deptype = 'i'");
                out.push(resync.finish());
                Ok(out)
            }
            Op::DropIdentity { table, column } => {
                let mut w = W::new();
                w.raw("ALTER TABLE ")
                    .qualified(table)
                    .raw(" ALTER COLUMN ")
                    .ident(column)
                    .raw(" DROP IDENTITY");
                one(w)
            }
            Op::AddForeignKey { table, key } => {
                let name = key
                    .name
                    .clone()
                    .unwrap_or_else(|| fk_name(table, &key.columns));
                one(add_foreign_key::<Self>(table, &name, key, &key.ref_table))
            }
            Op::RebuildTable { .. } => Err(LowerError::NotForThisDialect(op.kind())),
            Op::CreateSchema { schema } => {
                let mut w = W::new();
                w.raw("CREATE SCHEMA IF NOT EXISTS ").ident(schema);
                one(w)
            }
            Op::MoveTable { from, to_schema } => {
                let target = to_schema
                    .clone()
                    .unwrap_or_else(|| Ident::from_catalog(ctx.default_schema));
                let mut w = W::new();
                w.raw("ALTER TABLE ")
                    .qualified(from)
                    .raw(" SET SCHEMA ")
                    .ident(&target);
                one(w)
            }
        }
    }
}

fn column_def(w: &mut W, column: &ColumnDef, identity: bool) -> Result<(), LowerError> {
    let ty = desired_type::<PgDialect>(column)?;
    w.ident(&column.name).raw(" ").ty(&ty);
    if identity {
        w.raw(" GENERATED ALWAYS AS IDENTITY");
    }
    w.raw(if column.nullable {
        " NULL"
    } else {
        " NOT NULL"
    });
    Ok(())
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
        let name = pk.name.clone().unwrap_or_else(|| pk_name(&table.name));
        w.raw(", CONSTRAINT ")
            .ident(&name)
            .raw(" PRIMARY KEY (")
            .idents(&pk.columns)
            .raw(")");
    }
    w.raw(")");
    Ok(w)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialect::fixtures::*;
    use crate::model::SchemaModel;

    fn actual() -> SchemaModel {
        SchemaModel::from_tables(vec![actual_player(
            "integer",
            "character varying(100)",
            None,
        )])
        .unwrap()
    }

    #[test]
    fn lowering_snapshots() {
        for (name, op) in all_ops() {
            insta::assert_snapshot!(
                format!("pg_{name}"),
                render::<PgDialect>(&op, &actual(), "public")
            );
        }
        insta::assert_snapshot!(
            "pg_rebuild_table",
            render::<PgDialect>(&rebuild_op(), &actual(), "public")
        );
    }

    #[test]
    fn edge_case_snapshots() {
        for (name, op, actual) in
            edge_cases("integer", "character varying(100)", r#"pg_catalog."C""#)
        {
            insta::assert_snapshot!(
                format!("pg_edge_{name}"),
                render::<PgDialect>(&op, &actual, "public")
            );
        }
    }

    #[test]
    fn text_literals_escape_quotes_and_backslashes() {
        assert_eq!(PgDialect::text_literal("it's \\x"), r"E'it''s \\x'");
    }

    #[test]
    fn types_both_ways() {
        use canyon_core::types::SqlType::*;
        for (ty, sql, native) in [
            (Bool, "boolean", native("bool", None, None)),
            (SmallInt, "smallint", native("int2", None, None)),
            (Int, "integer", native("int4", None, None)),
            (BigInt, "bigint", native("int8", None, None)),
            (Text { max_len: None }, "text", native("text", None, None)),
            (
                Text { max_len: Some(100) },
                "varchar(100)",
                native("varchar", Some(Length::Chars(100)), None),
            ),
            (Date, "date", native("date", None, None)),
            (
                Time { precision: 6 },
                "time(6)",
                native("time", None, Some(6)),
            ),
            (
                Timestamp { precision: 3 },
                "timestamp(3)",
                native("timestamp", None, Some(3)),
            ),
        ] {
            assert_eq!(PgDialect::type_sql(ty).unwrap().as_str(), sql);
            assert_eq!(
                PgDialect::to_logical(&native),
                Logical::Known(ty),
                "{native:?}"
            );
        }
        // no precision in the catalog = PostgreSQL's default, 6
        assert_eq!(
            PgDialect::to_logical(&native("timestamp", None, None)),
            Logical::Known(Timestamp { precision: 6 })
        );
        for other in [
            "timestamptz",
            "bpchar",
            "numeric",
            "float8",
            "uuid",
            "jsonb",
        ] {
            assert!(
                matches!(
                    PgDialect::to_logical(&native(other, None, None)),
                    Logical::Other(_)
                ),
                "{other}"
            );
        }
    }

    fn native(name: &str, length: Option<Length>, precision: Option<u8>) -> NativeType {
        NativeType::new(name, length, precision, name)
    }
}
