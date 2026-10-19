//! SQL Server: one transaction per plan. `IDENTITY` can't be added to or removed from a
//! column, so identity changes rebuild the table.
//!
//! Every statement goes out as its own `sp_executesql` call: a `SET` inside one is undone
//! when it returns, so anything that needs a `SET` keeps it in the same statement.

use super::{DdlDialect, DdlWriter, LowerError, TypeSql};
use super::{actual_column, actual_type, add_foreign_key, add_primary_key, desired_type};
use super::{drop_constraint, fk_name, known, one, pk_name, renamed_constraints};
use super::{restated, rewritable};
use crate::model::{ColumnDef, ColumnDefault, Ident, Length, Logical, NativeType};
use crate::model::{QualifiedName, TableDef};
use crate::op::{Ctx, Op, Param, Statement};
use canyon_core::query::querybuilder::syntax::dialect::MsSql;
use canyon_core::types::SqlType;

type W = DdlWriter<MsSql>;

impl DdlDialect for MsSql {
    const TRANSACTIONAL_DDL: bool = true;

    fn type_sql(ty: SqlType) -> Result<TypeSql, LowerError> {
        Ok(TypeSql::new(match ty {
            SqlType::Bool => "bit".into(),
            SqlType::SmallInt => "smallint".into(),
            SqlType::Int => "int".into(),
            SqlType::BigInt => "bigint".into(),
            SqlType::Text { max_len: None } => "nvarchar(max)".into(),
            SqlType::Text { max_len: Some(n) } => format!("nvarchar({n})"),
            SqlType::Date => "date".into(),
            SqlType::Time { precision } => format!("time({precision})"),
            SqlType::Timestamp { precision } => format!("datetime2({precision})"),
            other => return Err(LowerError::UnsupportedType(other)),
        }))
    }

    fn to_logical(native: &NativeType) -> Logical {
        let precision = native.precision.unwrap_or(7); // SQL Server's default scale
        Logical::Known(match (native.name.as_str(), native.length) {
            ("bit", _) => SqlType::Bool,
            ("smallint", _) => SqlType::SmallInt,
            ("int", _) => SqlType::Int,
            ("bigint", _) => SqlType::BigInt,
            ("nvarchar", None | Some(Length::Max)) => SqlType::Text { max_len: None },
            ("nvarchar", Some(Length::Chars(n))) => SqlType::Text { max_len: Some(n) },
            ("date", _) => SqlType::Date,
            ("time", _) => SqlType::Time { precision },
            ("datetime2", _) => SqlType::Timestamp { precision },
            // varchar can't hold every String; datetime has 1/300 s steps; tinyint is unsigned
            _ => return Logical::Other(native.clone()),
        })
    }

    fn needs_rebuild(op: &Op) -> bool {
        matches!(op, Op::AddIdentity { .. } | Op::DropIdentity { .. })
    }

    // T-SQL has no backslash escapes: doubling the quote is all it takes
    fn text_literal(value: &str) -> String {
        format!("N'{}'", value.replace('\'', "''"))
    }

    fn lower(op: &Op, ctx: &Ctx<'_>) -> Result<Vec<Statement>, LowerError> {
        match op {
            Op::CreateTable(table) => one(create_table(table, &table.name, true)?),
            Op::RenameTable { from, to } => {
                let mut out = vec![sp_rename(object_name(from, None), to.as_str(), None)];
                // Canyon names constraints after the table: rename them too, or a new
                // table with the old name collides with them
                let renamed = QualifiedName {
                    schema: from.schema.clone(),
                    name: to.clone(),
                };
                let constraints = renamed_constraints(ctx.actual_table(from), from, &renamed);
                let pairs = constraints.primary_key.into_iter().chain(
                    constraints
                        .foreign_keys
                        .into_iter()
                        .map(|(_, old, new)| (old, new)),
                );
                for (old, new) in pairs {
                    let constraint = QualifiedName {
                        schema: from.schema.clone(),
                        name: old,
                    };
                    out.push(sp_rename(
                        object_name(&constraint, None),
                        new.as_str(),
                        Some("OBJECT"),
                    ));
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
                w.raw("ALTER TABLE ").qualified(table).raw(" ADD ");
                column_def(&mut w, column, false)?;
                one(w)
            }
            Op::RenameColumn { table, from, to } => Ok(vec![sp_rename(
                object_name(table, Some(from)),
                to.as_str(),
                Some("COLUMN"),
            )]),
            Op::DropColumn { table, column } => {
                let (_, actual) = actual_column(ctx, table, column)?;
                let mut out = drop_default(table, actual);
                let mut w = W::new();
                w.raw("ALTER TABLE ")
                    .qualified(table)
                    .raw(" DROP COLUMN ")
                    .ident(column);
                out.push(w.finish());
                Ok(out)
            }
            // Only the type: the column keeps the nullability it has now
            Op::AlterColumnType { table, column } => {
                let (_, actual) = actual_column(ctx, table, &column.name)?;
                rewritable(actual)?;
                let ty = desired_type::<Self>(column)?;
                let change = alter_column(
                    table,
                    &restated(actual, known(column)?),
                    &ty,
                    actual.nullable,
                );
                Ok(around_default(table, actual, change))
            }
            Op::SetNotNull { table, column } | Op::DropNotNull { table, column } => {
                let (_, actual) = actual_column(ctx, table, column)?;
                rewritable(actual)?;
                let ty = actual_type::<Self>(actual)?;
                let nullable = matches!(op, Op::DropNotNull { .. });
                Ok(around_default(
                    table,
                    actual,
                    alter_column(table, actual, &ty, nullable),
                ))
            }
            Op::AddPrimaryKey { table, key } => one(add_primary_key::<Self>(table, key)),
            Op::DropPrimaryKey { table, name } | Op::DropForeignKey { table, name } => {
                one(drop_constraint::<Self>(table, name))
            }
            Op::AddForeignKey { table, key } => {
                let name = key
                    .name
                    .clone()
                    .unwrap_or_else(|| fk_name(table, &key.columns));
                one(add_foreign_key::<Self>(table, &name, key, &key.ref_table))
            }
            Op::RebuildTable {
                from,
                to,
                column_map,
            } => rebuild(from, to, column_map),
            // `needs_rebuild`: the planner sends a RebuildTable instead
            Op::AddIdentity { .. } | Op::DropIdentity { .. } => {
                Err(LowerError::NotForThisDialect(op.kind()))
            }
            // CREATE SCHEMA must be alone in its batch, so it goes through EXEC. EXEC only
            // takes literals and variables, hence the variable; QUOTENAME quotes the bound
            // name, so no name reaches the batch text
            Op::CreateSchema { schema } => {
                let mut w = W::new();
                w.raw("DECLARE @canyon_sql nvarchar(max) = N'CREATE SCHEMA ' + QUOTENAME(")
                    .param(Param::Text(schema.to_string()))
                    .raw("); IF SCHEMA_ID(")
                    .param(Param::Text(schema.to_string()))
                    .raw(") IS NULL EXEC(@canyon_sql)");
                one(w)
            }
            // TRANSFER needs the source schema even when it's the default one
            Op::MoveTable { from, to_schema } => {
                let default = Ident::from_catalog(ctx.default_schema);
                let source = QualifiedName {
                    schema: Some(from.schema.clone().unwrap_or_else(|| default.clone())),
                    name: from.name.clone(),
                };
                let mut w = W::new();
                w.raw("ALTER SCHEMA ")
                    .ident(to_schema.as_ref().unwrap_or(&default))
                    .raw(" TRANSFER ")
                    .qualified(&source);
                one(w)
            }
        }
    }
}

fn column_def(w: &mut W, column: &ColumnDef, identity: bool) -> Result<(), LowerError> {
    let ty = desired_type::<MsSql>(column)?;
    w.ident(&column.name).raw(" ").ty(&ty);
    // Only a column read from the database has one (a rebuild's copy)
    if let Some(collation) = &column.extra.collation {
        w.raw(" COLLATE ").catalog_sql(collation);
    }
    if identity {
        w.raw(" IDENTITY(1,1)");
    }
    w.raw(if column.nullable {
        " NULL"
    } else {
        " NOT NULL"
    });
    Ok(())
}

fn create_table(table: &TableDef, name: &QualifiedName, with_pk: bool) -> Result<W, LowerError> {
    let mut w = W::new();
    w.raw("CREATE TABLE ").qualified(name).raw(" (");
    for (i, column) in table.columns.iter().enumerate() {
        if i > 0 {
            w.raw(", ");
        }
        column_def(&mut w, column, table.is_identity(&column.name))?;
    }
    if let (true, Some(pk)) = (with_pk, &table.primary_key) {
        let pk_ident = pk.name.clone().unwrap_or_else(|| pk_name(&table.name));
        w.raw(", CONSTRAINT ")
            .ident(&pk_ident)
            .raw(" PRIMARY KEY (")
            .idents(&pk.columns)
            .raw(")");
    }
    w.raw(")");
    Ok(w)
}

/// `[schema].[table]` or `[schema].[table].[column]`: `sp_rename` splits its object name
/// on dots, so every part is quoted. The new name is taken as it is.
fn object_name(table: &QualifiedName, column: Option<&Ident>) -> String {
    let mut w = W::new();
    w.qualified(table);
    if let Some(column) = column {
        w.raw(".").ident(column);
    }
    w.into_sql()
}

fn sp_rename(object: String, new_name: &str, kind: Option<&'static str>) -> Statement {
    let mut w = W::new();
    w.raw("EXEC sp_rename ")
        .param(Param::Text(object))
        .raw(", ")
        .param(Param::Text(new_name.to_owned()));
    match kind {
        Some("COLUMN") => w.raw(", 'COLUMN'"),
        Some(_) => w.raw(", 'OBJECT'"),
        None => &mut w,
    };
    w.finish()
}

// Without COLLATE a text column falls back to the database collation, so the column's
// own is written again. `column` already has the type it gets (see `restated`)
fn alter_column(
    table: &QualifiedName,
    column: &ColumnDef,
    ty: &TypeSql,
    nullable: bool,
) -> Statement {
    let mut w = W::new();
    w.raw("ALTER TABLE ")
        .qualified(table)
        .raw(" ALTER COLUMN ")
        .ident(&column.name)
        .raw(" ")
        .ty(ty);
    if let Some(collation) = &column.extra.collation {
        w.raw(" COLLATE ").catalog_sql(collation);
    }
    w.raw(if nullable { " NULL" } else { " NOT NULL" });
    w.finish()
}

fn drop_default(table: &QualifiedName, column: &ColumnDef) -> Vec<Statement> {
    let Some(name) = column.default.as_ref().and_then(ColumnDefault::constraint) else {
        return Vec::new();
    };
    vec![drop_constraint::<MsSql>(table, name).finish()]
}

fn add_default(table: &QualifiedName, column: &Ident, default: &ColumnDefault) -> Statement {
    let mut w = W::new();
    w.raw("ALTER TABLE ").qualified(table).raw(" ADD ");
    if let Some(name) = default.constraint() {
        w.raw("CONSTRAINT ").ident(name).raw(" ");
    }
    w.raw("DEFAULT ");
    MsSql::default_sql(&mut w, default);
    w.raw(" FOR ").ident(column);
    w.finish()
}

// SQL Server refuses ALTER COLUMN while a default constraint depends on the column
fn around_default(table: &QualifiedName, column: &ColumnDef, change: Statement) -> Vec<Statement> {
    let mut out = drop_default(table, column);
    out.push(change);
    if let Some(default) = &column.default
        && default.constraint().is_some()
    {
        out.push(add_default(table, &column.name, default));
    }
    out
}

fn rebuild(
    from: &TableDef,
    to: &TableDef,
    column_map: &[(Ident, Ident)],
) -> Result<Vec<Statement>, LowerError> {
    let temp = QualifiedName {
        schema: to.name.schema.clone(),
        name: Ident::from_catalog(&format!("{}__canyon_rebuild", to.name.name)),
    };
    // A computed column would come back as a plain one
    if let Some(column) = from.columns.iter().find(|c| c.extra.generated) {
        rewritable(column)?;
    }
    // The copy keeps each column's collation, like it keeps the defaults below, as far as
    // its type can hold one
    let mut target = to.clone();
    for (old, new) in column_map {
        if let (Some(old), Some(column)) = (
            from.column(old.as_str()),
            target.columns.iter_mut().find(|c| &c.name == new),
        ) && column.extra.collation.is_none()
            && let Logical::Known(ty) = column.ty
        {
            column.extra.collation = restated(old, ty).extra.collation;
        }
    }
    let mut out = vec![create_table(&target, &temp, false)?.finish()];

    // ON, the copy and OFF in one statement: a SET doesn't outlive its sp_executesql call
    let (old_cols, new_cols): (Vec<Ident>, Vec<Ident>) = column_map.iter().cloned().unzip();
    let copies_identity = new_cols.iter().any(|c| to.is_identity(c));
    let mut copy = W::new();
    if copies_identity {
        copy.raw("SET IDENTITY_INSERT ")
            .qualified(&temp)
            .raw(" ON; ");
    }
    copy.raw("INSERT INTO ")
        .qualified(&temp)
        .raw(" (")
        .idents(&new_cols)
        .raw(") SELECT ")
        .idents(&old_cols)
        .raw(" FROM ")
        .qualified(&from.name);
    if copies_identity {
        copy.raw("; SET IDENTITY_INSERT ")
            .qualified(&temp)
            .raw(" OFF");
    }
    out.push(copy.finish());

    let mut drop = W::new();
    drop.raw("DROP TABLE ").qualified(&from.name);
    out.push(drop.finish());
    out.push(sp_rename(
        object_name(&temp, None),
        to.name.name.as_str(),
        None,
    ));
    if let Some(pk) = &to.primary_key {
        out.push(add_primary_key::<MsSql>(&to.name, pk).finish());
    }
    // The defaults went with the old table; their names are free again now
    for (old, new) in column_map {
        let default = from.column(old.as_str()).and_then(|c| c.default.as_ref());
        if let Some(default) = default
            && !to.is_identity(new)
        {
            out.push(add_default(&to.name, new, default));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialect::fixtures::*;
    use crate::model::SchemaModel;

    fn actual(default: Option<ColumnDefault>) -> SchemaModel {
        SchemaModel::from_tables(vec![actual_player("int", "nvarchar(100)", default)]).unwrap()
    }

    #[test]
    fn lowering_snapshots() {
        for (name, op) in all_ops() {
            insta::assert_snapshot!(
                format!("mssql_{name}"),
                render::<MsSql>(&op, &actual(None), "dbo")
            );
        }
        insta::assert_snapshot!(
            "mssql_rebuild_table",
            render::<MsSql>(&rebuild_op(), &actual(None), "dbo")
        );
    }

    #[test]
    fn edge_case_snapshots() {
        for (name, op, actual) in edge_cases("int", "nvarchar(100)", "Latin1_General_BIN") {
            insta::assert_snapshot!(
                format!("mssql_edge_{name}"),
                render::<MsSql>(&op, &actual, "dbo")
            );
        }
    }

    #[test]
    fn defaults_are_dropped_and_restored_around_column_changes() {
        let default = Some(named(expr("((0))"), "DF_player_age"));
        for (name, op) in all_ops() {
            if matches!(
                name,
                "drop_column" | "alter_column_type" | "set_not_null" | "drop_not_null"
            ) {
                insta::assert_snapshot!(
                    format!("mssql_{name}_with_default"),
                    render::<MsSql>(&op, &actual(default.clone()), "dbo")
                );
            }
        }
    }

    #[test]
    fn identity_changes_need_a_rebuild() {
        for (name, op) in all_ops() {
            assert_eq!(
                MsSql::needs_rebuild(&op),
                matches!(name, "add_identity" | "drop_identity"),
                "{name}"
            );
        }
    }

    #[test]
    fn text_literals_double_the_quote() {
        assert_eq!(MsSql::text_literal("it's \\x"), r"N'it''s \x'");
    }

    #[test]
    fn types_both_ways() {
        use canyon_core::types::SqlType::*;
        for (ty, sql, native) in [
            (Bool, "bit", n("bit", None, None)),
            (SmallInt, "smallint", n("smallint", None, None)),
            (Int, "int", n("int", None, None)),
            (BigInt, "bigint", n("bigint", None, None)),
            (
                Text { max_len: None },
                "nvarchar(max)",
                n("nvarchar", Some(Length::Max), None),
            ),
            (
                Text { max_len: Some(100) },
                "nvarchar(100)",
                n("nvarchar", Some(Length::Chars(100)), None),
            ),
            (Date, "date", n("date", None, None)),
            (Time { precision: 6 }, "time(6)", n("time", None, Some(6))),
            (
                Timestamp { precision: 7 },
                "datetime2(7)",
                n("datetime2", None, Some(7)),
            ),
        ] {
            assert_eq!(MsSql::type_sql(ty).unwrap().as_str(), sql);
            assert_eq!(MsSql::to_logical(&native), Logical::Known(ty), "{native:?}");
        }
        for other in [
            "varchar",
            "ntext",
            "text",
            "datetime",
            "smalldatetime",
            "tinyint",
            "datetimeoffset",
            "uniqueidentifier",
        ] {
            assert!(
                matches!(MsSql::to_logical(&n(other, None, None)), Logical::Other(_)),
                "{other}"
            );
        }
    }

    fn n(name: &str, length: Option<Length>, precision: Option<u8>) -> NativeType {
        NativeType::new(name, length, precision, name)
    }
}
