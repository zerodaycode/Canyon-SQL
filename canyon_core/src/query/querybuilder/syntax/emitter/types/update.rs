use crate::query::querybuilder::syntax::ast::BaseAst;
use crate::query::querybuilder::syntax::ast::update::UpdateAst;
use crate::query::querybuilder::syntax::emitter::types::helpers;
use crate::query::querybuilder::syntax::emitter::{AstProcessor, SqlEmitter};
use crate::query::querybuilder::syntax::keyword::Keyword;
use crate::query::querybuilder::syntax::table_metadata::TableMetadata;
use crate::query::querybuilder::syntax::tokens::{SqlTokens, ToSqlTokens};

impl<'a, T> EmitUpdate<'a> for T
where
    T: SqlEmitter<'a>,
{
    fn emit_update(
        &mut self,
        ast: &impl AstProcessor<'a>,
        base_ast: &mut BaseAst<'a>,
    ) -> SqlTokens<'a> {
        let mut tokens = SqlTokens::default();
        let ast = transient::Downcast::downcast_ref::<UpdateAst>(ast.as_any()).expect(
            "[emitUpdate] - Handle this propagating result and introducing custom error types",
        );

        tokens.keyword(Keyword::Update);
        tokens
            .extend(<TableMetadata<'_> as ToSqlTokens<'_, T::Dialect>>::to_tokens(&base_ast.table));


        tokens.keyword(Keyword::Set);
        __impl::emit_set_clause::<T::Dialect>(&ast.columns, base_ast, &mut tokens);


        helpers::add_clause_conditions::<T::Dialect>(base_ast, &mut tokens);

        tokens
    }
}

pub trait EmitUpdate<'a>: SqlEmitter<'a> {
    fn emit_update(
        &mut self,
        ast: &impl AstProcessor<'a>,
        base_ast: &mut BaseAst<'a>,
    ) -> SqlTokens<'a>;
}

pub(crate) mod __impl {
    use crate::query::querybuilder::syntax::ast::BaseAst;
    use crate::query::querybuilder::syntax::column::ColumnRef;
    use crate::query::querybuilder::syntax::dialect::SqlDialect;
    use crate::query::querybuilder::syntax::tokens::{SqlTokens, Symbol, ToSqlTokens};

    pub(crate) fn emit_set_clause<'a, D: SqlDialect>(
        columns: &[ColumnRef<'a>],
        base_ast: &mut BaseAst<'a>,
        tokens: &mut SqlTokens<'a>,
    ) {
        for (i, col) in columns.iter().enumerate() {
            if i > 0 {
                tokens.symbol(Symbol::Comma);

            }
            tokens.extend(<ColumnRef<'_> as ToSqlTokens<'_, D>>::to_tokens(col));

            tokens.symbol(Symbol::Equals);

            tokens.placeholder();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EmitUpdate;
    use crate::query::operators::Operator;
    use crate::query::querybuilder::syntax::clause::{ConditionClause, ConditionClauseKind};
    use crate::query::querybuilder::syntax::dialect::MsSql;
    use crate::query::querybuilder::syntax::emitter::types::helpers::Range;
    use crate::query::querybuilder::syntax::writer::TokenWriter;
    use crate::query::querybuilder::syntax::{
        ast::BaseAst, ast::update::UpdateAst, column::ColumnRef, dialect::StandardDialect,
        emitter::SqlEmitter,
    };

    #[derive(Default)]
    struct TestUpdateEmitter;
    impl<'a> SqlEmitter<'a> for TestUpdateEmitter {
        type Dialect = StandardDialect;
    }

    #[derive(Default)]
    struct TestUpdateEmitterMsSql;
    impl<'a> SqlEmitter<'a> for TestUpdateEmitterMsSql {
        type Dialect = MsSql;
    }

    fn col(name: &'_ str) -> ColumnRef<'_> {
        ColumnRef::from(name)
    }

    fn render_standard<'a>(ast: &SelectlessUpdateAst<'a>, base_ast: &mut BaseAst<'a>) -> String {
        let mut emitter = TestUpdateEmitter;
        let tokens = emitter.emit_update(&ast.0, base_ast);
        TokenWriter::new()
            .render::<TestUpdateEmitter>(tokens)
            .unwrap()
    }

    fn render_mssql<'a>(ast: &SelectlessUpdateAst<'a>, base_ast: &mut BaseAst<'a>) -> String {
        let mut emitter = TestUpdateEmitterMsSql;
        let tokens = emitter.emit_update(&ast.0, base_ast);
        TokenWriter::new()
            .render::<TestUpdateEmitterMsSql>(tokens)
            .unwrap()
    }

    struct SelectlessUpdateAst<'a>(UpdateAst<'a>);
    impl<'a> SelectlessUpdateAst<'a> {
        fn new(ast: UpdateAst<'a>) -> Self {
            Self(ast)
        }
    }

    #[test]
    fn emits_update_with_single_set_column() {
        let ast = SelectlessUpdateAst::new(UpdateAst {
            columns: vec![col("name")],
        });

        let mut base_ast = BaseAst {
            table: "users".into(),
            ..Default::default()
        };

        let sql = render_standard(&ast, &mut base_ast);

        assert_eq!(sql, "UPDATE \"users\" SET \"name\" = $1;");
    }

    #[test]
    fn emits_update_with_multiple_set_columns() {
        let ast = SelectlessUpdateAst::new(UpdateAst {
            columns: vec![col("name"), col("email"), col("updated_at")],
        });

        let mut base_ast = BaseAst {
            table: "users".into(),
            ..Default::default()
        };

        let sql = render_standard(&ast, &mut base_ast);

        assert_eq!(
            sql,
            "UPDATE \"users\" SET \"name\" = $1, \"email\" = $2, \"updated_at\" = $3;"
        );
    }

    #[test]
    fn emits_update_with_where_conditions() {
        let ast = SelectlessUpdateAst::new(UpdateAst {
            columns: vec![col("name"), col("email")],
        });

        let mut base_ast = BaseAst {
            table: "users".into(),
            ..Default::default()
        };

        base_ast.conditions.push(ConditionClause {
            kind: ConditionClauseKind::Where,
            column_name: "id".into(),
            operator: Operator::Eq,
            value_indexes: Some(Range::new_unbounded(3)),
        });

        let sql = render_standard(&ast, &mut base_ast);

        assert_eq!(
            sql,
            "UPDATE \"users\" SET \"name\" = $1, \"email\" = $2 WHERE \"id\" = $3;"
        );
    }

    #[test]
    fn emits_update_in_mssql_with_dialect_specific_identifiers_and_placeholders() {
        let ast = SelectlessUpdateAst::new(UpdateAst {
            columns: vec![col("name"), col("email")],
        });

        let mut base_ast = BaseAst {
            table: "users".into(),
            ..Default::default()
        };

        let sql = render_mssql(&ast, &mut base_ast);

        assert_eq!(sql, "UPDATE [users] SET [name] = @P1, [email] = @P2;");
    }

    #[test]
    fn preserves_placeholder_sequence_between_set_and_where() {
        let ast = SelectlessUpdateAst::new(UpdateAst {
            columns: vec![col("name"), col("email")],
        });

        let mut base_ast = BaseAst {
            table: "users".into(),
            ..Default::default()
        };

        base_ast.conditions.push(ConditionClause {
            kind: ConditionClauseKind::Where,
            column_name: "id".into(),
            operator: Operator::Eq,
            value_indexes: Some(Range::new_unbounded(3)),
        });

        let sql = render_standard(&ast, &mut base_ast);

        assert_eq!(
            sql,
            "UPDATE \"users\" SET \"name\" = $1, \"email\" = $2 WHERE \"id\" = $3;"
        );
    }
}
