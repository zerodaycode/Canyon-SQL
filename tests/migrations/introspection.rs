//! Reading the schema of real databases back into the model.

use crate::support::per_backend;
use crate::support::test_db::TestDb;
use canyon_migrations::dialect::{DdlDialect, LowerError};
use canyon_migrations::introspect::{Catalog, introspect};
use canyon_migrations::model::{ColumnDef, ColumnDefault, Generated, Ident, Logical};
use canyon_migrations::model::{QualifiedName, SchemaModel, TableDef};
use canyon_migrations::op::{Ctx, Op, Statement};
use canyon_sql::CanyonResult;
use canyon_sql::connection::{DatabaseType, DbConnection};
use canyon_sql::core::SqlType;
use canyon_sql::query::QueryParameter;

pub(crate) async fn read(db: &TestDb, schemas: &[Ident]) -> Catalog {
    introspect(db.conn(), db.db_type(), schemas)
        .await
        .expect("introspection works")
}

pub(crate) async fn read_model(db: &TestDb) -> SchemaModel {
    read(db, &[]).await.model
}

pub(crate) fn lower_for(db: DatabaseType, op: &Op, ctx: &Ctx<'_>) -> Vec<Statement> {
    use canyon_sql::query::querybuilder::syntax::dialect::*;
    match db {
        #[cfg(feature = "postgres")]
        DatabaseType::PostgreSql => PgDialect::lower(op, ctx),
        #[cfg(feature = "mssql")]
        DatabaseType::SqlServer => MsSql::lower(op, ctx),
        #[cfg(feature = "mysql")]
        DatabaseType::MySQL => MySql::lower(op, ctx),
    }
    .expect("lowering works")
}

/// Lowers `op` against what the database has now and runs it.
pub(crate) async fn apply(db: &TestDb, op: &Op) -> CanyonResult<()> {
    let catalog = read(db, &[]).await;
    for st in lower_for(db.db_type(), op, &ctx(&catalog)) {
        let params: Vec<&dyn QueryParameter> =
            st.params.iter().map(|p| p.as_query_param()).collect();
        db.conn().execute(&st.sql, &params).await?;
    }
    Ok(())
}

pub(crate) fn ctx(catalog: &Catalog) -> Ctx<'_> {
    Ctx {
        server: catalog.server,
        default_schema: &catalog.default_schema,
        no_backslash_escapes: catalog.no_backslash_escapes,
        actual: &catalog.model,
    }
}

fn table<'a>(m: &'a SchemaModel, name: &str) -> &'a TableDef {
    m.tables
        .values()
        .find(|t| t.name.to_string() == name)
        .unwrap_or_else(|| panic!("no table {name} in {:?}", m.tables.keys()))
}

fn ident(name: &str) -> Ident {
    Ident::new(name).unwrap()
}

/// Composite PK, composite FK, identity, a default, every canonical type; on PostgreSQL a
/// partitioned table.
fn fixture(db: DatabaseType) -> Vec<&'static str> {
    match db {
        #[cfg(feature = "postgres")]
        DatabaseType::PostgreSql => vec![
            "CREATE TABLE league (id integer GENERATED ALWAYS AS IDENTITY, flag boolean NOT NULL, \
             s smallint, b bigint, txt text, name varchar(20) DEFAULT 'x', d date, t time(3), \
             ts timestamp(6), other xml, PRIMARY KEY (id))",
            "CREATE TABLE comp (x int NOT NULL, y int NOT NULL, PRIMARY KEY (x, y))",
            "CREATE TABLE child (id int PRIMARY KEY, lx int, ly int, l int REFERENCES league(id), \
             FOREIGN KEY (lx, ly) REFERENCES comp(x, y))",
            "CREATE TABLE canyon_migration_history (id int)",
            // A partitioned table is one table
            "CREATE TABLE part (id int, k int, PRIMARY KEY (id, k)) PARTITION BY LIST (k)",
            "CREATE TABLE part_1 PARTITION OF part FOR VALUES IN (1)",
        ],
        #[cfg(feature = "mssql")]
        DatabaseType::SqlServer => vec![
            "CREATE TABLE league (id int IDENTITY(1,1) NOT NULL, flag bit NOT NULL, s smallint, \
             b bigint, txt nvarchar(max), name nvarchar(20) CONSTRAINT DF_league_name DEFAULT 'x', \
             d date, t time(3), ts datetime2(6), other xml, CONSTRAINT league_pkey PRIMARY KEY (id))",
            "CREATE TABLE comp (x int NOT NULL, y int NOT NULL, PRIMARY KEY (x, y))",
            "CREATE TABLE child (id int PRIMARY KEY, lx int, ly int, l int REFERENCES league(id), \
             FOREIGN KEY (lx, ly) REFERENCES comp(x, y))",
            "CREATE TABLE canyon_migration_history (id int)",
        ],
        #[cfg(feature = "mysql")]
        DatabaseType::MySQL => vec![
            "CREATE TABLE league (id int NOT NULL AUTO_INCREMENT, flag tinyint(1) NOT NULL, \
             s smallint, b bigint, txt longtext, name varchar(20) DEFAULT 'x', d date, t time(3), \
             ts datetime(6), other json, PRIMARY KEY (id))",
            "CREATE TABLE comp (x int NOT NULL, y int NOT NULL, PRIMARY KEY (x, y))",
            "CREATE TABLE child (id int PRIMARY KEY, lx int, ly int, l int, \
             FOREIGN KEY (l) REFERENCES league(id), FOREIGN KEY (lx, ly) REFERENCES comp(x, y))",
            "CREATE TABLE canyon_migration_history (id int)",
        ],
    }
}

async fn reads_the_fixture(datasource: &str) -> CanyonResult<()> {
    let catalog = TestDb::with(datasource, "introspect_fixture", async |db| {
        for ddl in fixture(db.db_type()) {
            db.conn().execute(ddl, &[]).await?;
        }
        Ok((read(db, &[]).await, db.db_type()))
    })
    .await?;
    let (catalog, db_type) = catalog;

    let m = &catalog.model;
    let names: Vec<_> = m.tables.keys().map(ToString::to_string).collect();
    let mut expected = vec!["child", "comp", "league"];
    #[cfg(feature = "postgres")]
    if matches!(db_type, DatabaseType::PostgreSql) {
        expected.push("part");
    }
    let _ = db_type;
    assert_eq!(names, expected, "canyon tables and partitions are left out");
    assert!(catalog.existing_schemas.is_empty());

    let league = table(m, "league");
    let ty = |c: &str| league.column(c).unwrap().ty.clone();
    assert_eq!(ty("flag"), Logical::Known(SqlType::Bool));
    assert_eq!(ty("s"), Logical::Known(SqlType::SmallInt));
    assert_eq!(ty("id"), Logical::Known(SqlType::Int));
    assert_eq!(ty("b"), Logical::Known(SqlType::BigInt));
    assert_eq!(ty("txt"), Logical::Known(SqlType::Text { max_len: None }));
    assert_eq!(
        ty("name"),
        Logical::Known(SqlType::Text { max_len: Some(20) })
    );
    assert_eq!(ty("d"), Logical::Known(SqlType::Date));
    assert_eq!(ty("t"), Logical::Known(SqlType::Time { precision: 3 }));
    assert_eq!(
        ty("ts"),
        Logical::Known(SqlType::Timestamp { precision: 6 })
    );
    assert!(matches!(ty("other"), Logical::Other(_)));
    assert!(!league.column("flag").unwrap().nullable);
    assert!(league.column("s").unwrap().nullable);
    assert!(league.column("name").unwrap().default.is_some());
    assert!(league.column("s").unwrap().default.is_none());
    let pk = league.primary_key.as_ref().unwrap();
    assert_eq!(pk.columns, [ident("id")]);
    assert_eq!(pk.generated, Some(Generated::Identity));

    let comp_pk = table(m, "comp").primary_key.as_ref().unwrap();
    assert_eq!(comp_pk.columns, [ident("x"), ident("y")]);
    assert_eq!(comp_pk.generated, None);

    let child = table(m, "child");
    assert_eq!(child.foreign_keys.len(), 2);
    let composite = child
        .foreign_keys
        .iter()
        .find(|f| f.columns.len() == 2)
        .unwrap();
    assert_eq!(
        composite.ref_table.to_string(),
        "comp",
        "default schema reference is unqualified"
    );
    assert_eq!(composite.columns, [ident("lx"), ident("ly")]);
    assert_eq!(composite.ref_columns, [ident("x"), ident("y")]);
    assert!(composite.name.is_some());
    assert!(
        child
            .foreign_keys
            .iter()
            .any(|f| f.ref_table.to_string() == "league")
    );

    assert!(
        catalog.server.major >= 8,
        "server version read: {:?}",
        catalog.server
    );
    Ok(())
}
per_backend!(introspection_reads_the_fixture, super::reads_the_fixture);

/// Each dialect's canonical types read back as the same `SqlType`.
async fn canonical_types_read_back(datasource: &str) -> CanyonResult<()> {
    let wanted = [
        SqlType::Bool,
        SqlType::SmallInt,
        SqlType::Int,
        SqlType::BigInt,
        SqlType::Text { max_len: None },
        SqlType::Text { max_len: Some(450) },
        SqlType::Date,
        SqlType::Time { precision: 6 },
        SqlType::Timestamp { precision: 0 },
        SqlType::Timestamp { precision: 6 },
    ];
    let mut builder = TableDef::builder("Canon", "canon");
    for (i, ty) in wanted.iter().enumerate() {
        builder = builder.column(&format!("c{i}"), *ty, true);
    }
    let canon = Op::CreateTable(builder.build().unwrap());

    let model = TestDb::with(datasource, "introspect_canonical", async |db| {
        apply(db, &canon).await?;
        Ok(read_model(db).await)
    })
    .await?;

    let canon = table(&model, "canon");
    for (i, ty) in wanted.iter().enumerate() {
        let column = canon.column(&format!("c{i}")).unwrap();
        assert_eq!(column.ty, Logical::Known(*ty), "c{i}");
        assert!(column.nullable, "c{i}");
    }
    Ok(())
}
per_backend!(
    introspection_canonical_types_read_back,
    super::canonical_types_read_back
);

/// Asked schemas are read too; tables in them keep their schema, references to the
/// default one don't.
async fn reads_other_schemas(datasource: &str) -> CanyonResult<()> {
    TestDb::with(datasource, "introspect_schemas", async |db| {
        let s1 = Ident::from_catalog(&db.schema("s1"));
        let missing = Ident::from_catalog(&db.schema("nope"));
        apply(db, &Op::CreateSchema { schema: s1.clone() }).await?;
        db.conn()
            .execute("CREATE TABLE league (id int PRIMARY KEY)", &[])
            .await?;
        // MySQL resolves an unqualified REFERENCES in the child's database
        let league = match db.db_type() {
            #[cfg(feature = "mysql")]
            DatabaseType::MySQL => format!("`{}`.league", db.name()),
            #[allow(unreachable_patterns)]
            _ => "league".to_owned(),
        };
        let quoted = match db.db_type() {
            #[cfg(feature = "mysql")]
            DatabaseType::MySQL => format!("`{s1}`"),
            #[allow(unreachable_patterns)]
            _ => s1.to_string(),
        };
        let child = format!(
            "CREATE TABLE {quoted}.child (id int PRIMARY KEY, league int, \
             FOREIGN KEY (league) REFERENCES {league}(id))"
        );
        db.conn().execute(&child, &[]).await?;

        let catalog = read(db, &[s1.clone(), missing]).await;
        let child_name = QualifiedName {
            schema: Some(s1.clone()),
            name: ident("child"),
        };
        let child = catalog
            .model
            .table(&child_name)
            .unwrap_or_else(|| panic!("no {child_name} in {:?}", catalog.model.tables.keys()));
        assert_eq!(child.foreign_keys[0].ref_table.to_string(), "league");
        assert!(child.primary_key.is_some());
        assert!(table(&catalog.model, "league").name.schema.is_none());
        assert_eq!(catalog.existing_schemas, [s1]);

        let only_default = read_model(db).await;
        assert!(only_default.table(&child_name).is_none());
        assert_eq!(only_default.tables.len(), 1);
        Ok(())
    })
    .await
}
per_backend!(
    introspection_reads_other_schemas,
    super::reads_other_schemas
);

/// Collation, comment, ON UPDATE and a generated column as each backend has them.
fn extras_fixture(db: DatabaseType) -> &'static str {
    match db {
        #[cfg(feature = "postgres")]
        DatabaseType::PostgreSql => {
            "CREATE TABLE t (name varchar(20) COLLATE \"C\" NOT NULL, ts timestamp(6) DEFAULT now(), \
             n int, g int GENERATED ALWAYS AS (n * 2) STORED)"
        }
        #[cfg(feature = "mssql")]
        DatabaseType::SqlServer => {
            "CREATE TABLE t (name nvarchar(20) COLLATE Latin1_General_BIN NOT NULL, \
             ts datetime2(6) CONSTRAINT DF_t_ts DEFAULT SYSDATETIME(), \
             n int, g AS (n * 2))"
        }
        #[cfg(feature = "mysql")]
        DatabaseType::MySQL => {
            "CREATE TABLE t (name varchar(20) CHARACTER SET latin1 COLLATE latin1_bin NOT NULL \
             COMMENT 'it''s \\\\ a comment', \
             ts datetime(6) DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6), \
             n int, g int GENERATED ALWAYS AS (n * 2) STORED)"
        }
    }
}

/// Statements that write a column again keep what Canyon doesn't manage, and a generated
/// column is never written again where that would break it.
async fn restating_keeps_column_extras(datasource: &str) -> CanyonResult<()> {
    TestDb::with(datasource, "introspect_extras", async |db| {
        db.conn().execute(extras_fixture(db.db_type()), &[]).await?;
        let before = read(db, &[]).await;
        let t = QualifiedName::parse("t").unwrap();
        for op in [
            Op::DropNotNull {
                table: t.clone(),
                column: ident("name"),
            },
            Op::AlterColumnType {
                table: t.clone(),
                column: column("name", SqlType::Text { max_len: Some(40) }),
            },
            Op::SetNotNull {
                table: t.clone(),
                column: ident("ts"),
            },
        ] {
            apply(db, &op).await?;
        }
        let after = read_model(db).await;

        let (old, new) = (table(&before.model, "t"), table(&after, "t"));
        let name = new.column("name").unwrap();
        assert_eq!(name.ty, Logical::Known(SqlType::Text { max_len: Some(40) }));
        assert!(name.nullable);
        assert!(!new.column("ts").unwrap().nullable);
        for column in ["name", "ts", "g"] {
            let (was, is) = (old.column(column).unwrap(), new.column(column).unwrap());
            assert_eq!(was.extra, is.extra, "{column}");
            assert_eq!(was.default, is.default, "{column}");
        }
        let extra = &old.column("name").unwrap().extra;
        assert!(extra.collation.is_some(), "collation read");
        let g = old.column("g").unwrap();
        assert!(g.extra.generated, "generated column read");
        assert!(
            g.default.is_none(),
            "a generation expression isn't a default"
        );

        let rewrite_g = Op::SetNotNull {
            table: t.clone(),
            column: ident("g"),
        };
        let ctx = ctx(&before);
        #[allow(unused_imports)]
        use canyon_sql::query::querybuilder::syntax::dialect::*;
        match db.db_type() {
            #[cfg(feature = "mysql")]
            DatabaseType::MySQL => {
                assert_eq!(extra.comment.as_deref(), Some("it's \\ a comment"));
                assert!(old.column("ts").unwrap().extra.on_update.is_some());
                assert_eq!(
                    MySql::lower(&rewrite_g, &ctx),
                    Err(LowerError::GeneratedColumn("g".into()))
                );
            }
            #[cfg(feature = "mssql")]
            DatabaseType::SqlServer => assert_eq!(
                MsSql::lower(&rewrite_g, &ctx),
                Err(LowerError::GeneratedColumn("g".into()))
            ),
            #[allow(unreachable_patterns)]
            _ => {}
        }
        Ok(())
    })
    .await
}
per_backend!(
    introspection_restating_keeps_column_extras,
    super::restating_keeps_column_extras
);

/// Column shapes legacy MySQL tables have, written again by type and nullability changes.
#[cfg(feature = "mysql")]
#[canyon_sql::macros::canyon_tokio_test]
fn introspection_mysql_restates_legacy_columns() {
    TestDb::with(
        crate::constants::MYSQL_DS,
        "introspect_mysql_legacy",
        async |db| {
            db.conn()
                .execute(
                    "CREATE TABLE t (\
                     ts datetime(6) NULL DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6), \
                     legacy timestamp NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP, \
                     created datetime(6) NULL DEFAULT CURRENT_TIMESTAMP(6), \
                     v varchar(10) NULL DEFAULT (concat('a', 'b')), \
                     q varchar(10) NULL DEFAULT ('it''s'), \
                     w varchar(10) NULL DEFAULT (concat('a''b', 'c')), \
                     b bit(1) NULL DEFAULT b'1', \
                     f tinyint(1) NOT NULL DEFAULT 1)",
                    &[],
                )
                .await?;
            let t = QualifiedName::parse("t").unwrap();
            let not_null = |c: &str| Op::SetNotNull {
                table: t.clone(),
                column: ident(c),
            };
            for op in [
                Op::AlterColumnType {
                    table: t.clone(),
                    column: column("ts", SqlType::Timestamp { precision: 3 }),
                },
                Op::AlterColumnType {
                    table: t.clone(),
                    column: column("legacy", SqlType::Timestamp { precision: 6 }),
                },
                Op::AlterColumnType {
                    table: t.clone(),
                    column: column("created", SqlType::Text { max_len: Some(40) }),
                },
                not_null("v"),
                not_null("q"),
                not_null("w"),
                not_null("b"),
                Op::DropNotNull {
                    table: t.clone(),
                    column: ident("f"),
                },
            ] {
                apply(db, &op).await?;
            }

            let model = read_model(db).await;
            let t = table(&model, "t");
            let col = |c: &str| t.column(c).unwrap();
            let on_update = |c: &str| col(c).extra.on_update.as_ref().map(|s| s.as_str().to_owned());
            let default = |c: &str| match &col(c).default {
                Some(ColumnDefault::Expr { sql, .. }) => sql.as_str().to_owned(),
                Some(ColumnDefault::Literal { value, .. }) => value.clone(),
                other => panic!("{c}: {other:?}"),
            };
            assert_eq!(col("ts").ty, Logical::Known(SqlType::Timestamp { precision: 3 }));
            assert_eq!(default("ts"), "CURRENT_TIMESTAMP(3)");
            assert_eq!(on_update("ts").as_deref(), Some("CURRENT_TIMESTAMP(3)"));
            assert_eq!(col("legacy").ty, Logical::Known(SqlType::Timestamp { precision: 6 }));
            assert_eq!(default("legacy"), "CURRENT_TIMESTAMP(6)");
            assert_eq!(on_update("legacy").as_deref(), Some("CURRENT_TIMESTAMP(6)"));
            assert_eq!(col("created").ty, Logical::Known(SqlType::Text { max_len: Some(40) }));
            assert!(col("created").default.is_some(), "the default stays");
            assert_eq!(default("v"), "concat(_utf8mb4'a',_utf8mb4'b')");
            assert_eq!(default("q"), "_utf8mb4'it\\'s'");
            assert_eq!(default("b"), "b'1'");
            assert_eq!(default("f"), "1");
            for c in ["v", "q", "w", "b"] {
                assert!(!col(c).nullable, "{c}");
            }
            assert!(col("f").nullable);

            // The defaults still give the same values
            db.conn().execute("INSERT INTO t () VALUES ()", &[]).await?;
            let row: Vec<(String, String, String, i64)> = db
                .conn()
                .query_rows("SELECT v, q, w, CAST(b AS UNSIGNED) FROM t", &[])
                .await?
                .get_mysql_rows()?
                .iter()
                .map(|r| {
                    (
                        r.get(0).unwrap(),
                        r.get(1).unwrap(),
                        r.get(2).unwrap(),
                        r.get(3).unwrap(),
                    )
                })
                .collect();
            assert_eq!(
                row,
                [("ab".to_owned(), "it's".to_owned(), "a'bc".to_owned(), 1)]
            );
            Ok(())
        },
    )
    .await?;
}

/// A SQL Server rebuild keeps the collation of text columns, drops it from a column that
/// stops being text, and copies the rows.
#[cfg(feature = "mssql")]
#[canyon_sql::macros::canyon_tokio_test]
fn introspection_mssql_rebuild_keeps_collations() {
    TestDb::with(
        crate::constants::SQL_SERVER_DS,
        "introspect_mssql_rebuild",
        async |db| {
            db.conn()
                .execute(
                    "CREATE TABLE r (id int NOT NULL CONSTRAINT r_pkey PRIMARY KEY, \
                     name nvarchar(20) COLLATE Latin1_General_BIN NULL, \
                     code nvarchar(20) COLLATE Latin1_General_BIN NULL)",
                    &[],
                )
                .await?;
            db.conn()
                .execute("INSERT INTO r VALUES (5, N'Ab', N'7')", &[])
                .await?;
            let from = table(&read_model(db).await, "r").clone();
            let mut to = from.clone();
            to.primary_key.as_mut().unwrap().generated = Some(Generated::Identity);
            let code = to
                .columns
                .iter_mut()
                .find(|c| c.name.as_str() == "code")
                .unwrap();
            code.ty = Logical::Known(SqlType::Int);
            code.extra.collation = None;
            let column_map = from
                .columns
                .iter()
                .map(|c| (c.name.clone(), c.name.clone()))
                .collect();
            apply(
                db,
                &Op::RebuildTable {
                    from,
                    to,
                    column_map,
                },
            )
            .await?;

            let model = read_model(db).await;
            let r = table(&model, "r");
            assert_eq!(
                r.primary_key.as_ref().unwrap().generated,
                Some(Generated::Identity)
            );
            let name = r.column("name").unwrap();
            assert_eq!(
                name.extra.collation.as_ref().map(|c| c.as_str()),
                Some("Latin1_General_BIN")
            );
            let code = r.column("code").unwrap();
            assert_eq!(code.ty, Logical::Known(SqlType::Int));
            assert_eq!(code.extra.collation, None);
            let id: i32 = db
                .conn()
                .query_one_for("SELECT MAX(id) FROM r", &[])
                .await?;
            assert_eq!(id, 5, "rows copied");
            Ok(())
        },
    )
    .await?;
}

fn column(name: &str, ty: SqlType) -> ColumnDef {
    TableDef::builder("T", "t")
        .column(name, ty, true)
        .build()
        .unwrap()
        .columns
        .remove(0)
}
