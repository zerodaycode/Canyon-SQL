//! Test data shared by the three dialects' snapshot tests.

use super::DdlDialect;
use crate::model::*;
use crate::op::{Ctx, Op, ServerVersion};
use canyon_core::types::SqlType;

pub fn q(s: &str) -> QualifiedName {
    QualifiedName::parse(s).unwrap()
}
pub fn id(s: &str) -> Ident {
    Ident::new(s).unwrap()
}

pub fn player() -> TableDef {
    TableDef::builder("Player", "player")
        .column("id", SqlType::Int, false)
        .primary_key(Some(Generated::Identity))
        .column("name", SqlType::Text { max_len: Some(100) }, false)
        .column("age", SqlType::Int, true)
        .column("league", SqlType::Int, true)
        .foreign_key("league", "id")
        .build()
        .unwrap()
}

/// `player` as a catalog would report it: real constraint names, native types, one default.
pub fn actual_player(raw_int: &str, raw_text: &str, default: Option<ColumnDefault>) -> TableDef {
    let mut t = player();
    t.primary_key.as_mut().unwrap().name = Some(Ident::from_catalog("player_pkey"));
    t.foreign_keys[0].name = Some(Ident::from_catalog("player_league_fkey"));
    for c in &mut t.columns {
        let raw = if c.name.as_str() == "name" {
            raw_text
        } else {
            raw_int
        };
        c.ty = Logical::Other(NativeType::named(raw, raw)); // the dialect only reads `raw`
    }
    t.columns[2].default = default; // `age`
    t
}

pub fn ctx<'a>(actual: &'a SchemaModel, default_schema: &'a str) -> Ctx<'a> {
    Ctx {
        server: ServerVersion {
            major: 99,
            minor: 0,
            patch: 0,
        },
        default_schema,
        actual,
    }
}

/// The statements of `op` as snapshot text, or the error.
pub fn render<D: DdlDialect>(op: &Op, actual: &SchemaModel, default_schema: &str) -> String {
    match D::lower(op, &ctx(actual, default_schema)) {
        Ok(statements) => statements
            .iter()
            .map(|s| {
                format!(
                    "{}\n  params: {:?}\n  transactional: {}",
                    s.sql, s.params, s.transactional
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
        Err(e) => format!("ERROR {e:?}"),
    }
}

pub fn expr(sql: &str) -> ColumnDefault {
    ColumnDefault::Expr {
        constraint: None,
        sql: CatalogSql::new(sql),
    }
}

pub fn named(default: ColumnDefault, constraint: &str) -> ColumnDefault {
    let name = Some(Ident::from_catalog(constraint));
    match default {
        ColumnDefault::Expr { sql, .. } => ColumnDefault::Expr {
            constraint: name,
            sql,
        },
        ColumnDefault::Literal { value, .. } => ColumnDefault::Literal {
            constraint: name,
            value,
        },
    }
}

pub fn literal(value: &str) -> ColumnDefault {
    ColumnDefault::Literal {
        constraint: None,
        value: value.into(),
    }
}

pub fn all_ops() -> Vec<(&'static str, Op)> {
    let t = q("player");
    let age = ColumnDef {
        name: id("age"),
        ty: Logical::Known(SqlType::BigInt),
        nullable: true,
        default: None,
    };
    let pk = PrimaryKey {
        name: None,
        columns: vec![id("id")],
        generated: None,
    };
    let fk = player().foreign_keys[0].clone();
    vec![
        ("create_table", Op::CreateTable(player())),
        (
            "create_table_in_schema",
            Op::CreateTable({
                let mut p = player();
                p.name = q("s.player");
                p.foreign_keys.clear();
                p
            }),
        ),
        (
            "rename_table",
            Op::RenameTable {
                from: t.clone(),
                to: id("players"),
            },
        ),
        ("drop_table", Op::DropTable(t.clone())),
        (
            "add_column",
            Op::AddColumn {
                table: t.clone(),
                column: age.clone(),
            },
        ),
        (
            "rename_column",
            Op::RenameColumn {
                table: t.clone(),
                from: id("name"),
                to: id("full_name"),
            },
        ),
        (
            "drop_column",
            Op::DropColumn {
                table: t.clone(),
                column: id("age"),
            },
        ),
        (
            "alter_column_type",
            Op::AlterColumnType {
                table: t.clone(),
                column: age,
            },
        ),
        (
            "set_not_null",
            Op::SetNotNull {
                table: t.clone(),
                column: id("age"),
            },
        ),
        (
            "drop_not_null",
            Op::DropNotNull {
                table: t.clone(),
                column: id("age"),
            },
        ),
        (
            "add_primary_key",
            Op::AddPrimaryKey {
                table: t.clone(),
                key: pk,
            },
        ),
        (
            "drop_primary_key",
            Op::DropPrimaryKey {
                table: t.clone(),
                name: Ident::from_catalog("player_pkey"),
            },
        ),
        (
            "add_identity",
            Op::AddIdentity {
                table: t.clone(),
                column: id("id"),
            },
        ),
        (
            "drop_identity",
            Op::DropIdentity {
                table: t.clone(),
                column: id("id"),
            },
        ),
        (
            "add_foreign_key",
            Op::AddForeignKey {
                table: t.clone(),
                key: fk,
            },
        ),
        (
            "drop_foreign_key",
            Op::DropForeignKey {
                table: t.clone(),
                name: Ident::from_catalog("player_league_fkey"),
            },
        ),
        ("create_schema", Op::CreateSchema { schema: id("s") }),
        (
            "move_table",
            Op::MoveTable {
                from: t.clone(),
                to_schema: Some(id("s")),
            },
        ),
        (
            "move_table_to_default",
            Op::MoveTable {
                from: q("s.player"),
                to_schema: None,
            },
        ),
    ]
}

/// The rebuild SQL Server uses for identity changes: `from` without identity → `player`.
pub fn rebuild_op_from(mut from: TableDef) -> Op {
    from.primary_key.as_mut().unwrap().generated = None;
    let to = player();
    let column_map = to
        .columns
        .iter()
        .map(|c| (c.name.clone(), c.name.clone()))
        .collect();
    Op::RebuildTable {
        from,
        to,
        column_map,
    }
}

pub fn rebuild_op() -> Op {
    rebuild_op_from(player())
}

/// The cases the happy-path fixtures miss: defaults where they get in the way, names
/// that need quoting, schemas. Each dialect snapshots all of them.
pub fn edge_cases(raw_int: &str, raw_text: &str) -> Vec<(&'static str, Op, SchemaModel)> {
    let model = |t: TableDef| SchemaModel::from_tables(vec![t]).unwrap();
    let with_default = |col: usize, default: ColumnDefault| {
        let mut t = actual_player(raw_int, raw_text, None);
        t.columns[col].default = Some(default);
        t
    };
    let tricky_value = "it's \\ -- ; done";
    vec![
        (
            "add_identity_over_default",
            Op::AddIdentity {
                table: q("player"),
                column: id("id"),
            },
            model(with_default(0, expr("nextval('player_id_seq'::regclass)"))),
        ),
        (
            "set_not_null_with_literal_default",
            Op::SetNotNull {
                table: q("player"),
                column: id("age"),
            },
            model(with_default(
                2,
                named(literal(tricky_value), "DF_player_age"),
            )),
        ),
        (
            "alter_type_keeps_nullability",
            Op::AlterColumnType {
                table: q("player"),
                column: ColumnDef {
                    name: id("age"),
                    ty: Logical::Known(SqlType::BigInt),
                    nullable: false, // the desired side; the column is still NULL
                    default: None,
                },
            },
            model(actual_player(raw_int, raw_text, None)),
        ),
        (
            "rebuild_keeps_defaults",
            rebuild_op_from(with_default(2, named(expr("((0))"), "DF_player_age"))),
            SchemaModel::default(),
        ),
        (
            "rename_table_with_odd_name",
            Op::RenameTable {
                from: QualifiedName {
                    schema: None,
                    name: Ident::from_catalog("we.ird]`\""),
                },
                to: id("players"),
            },
            SchemaModel::default(),
        ),
        (
            "rename_column_with_odd_names",
            Op::RenameColumn {
                table: QualifiedName {
                    schema: Some(Ident::from_catalog("my.s")),
                    name: Ident::from_catalog("we.ird"),
                },
                from: Ident::from_catalog("c]1"),
                to: id("c2"),
            },
            SchemaModel::default(),
        ),
        (
            "foreign_key_from_a_schema",
            Op::AddForeignKey {
                table: q("s.player"),
                key: player().foreign_keys[0].clone(),
            },
            SchemaModel::default(),
        ),
    ]
}
