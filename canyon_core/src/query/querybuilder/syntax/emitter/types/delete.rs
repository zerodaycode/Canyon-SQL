use crate::query::querybuilder::syntax::clause::ConditionClause;
use crate::query::querybuilder::syntax::table_metadata::TableMetadata;
use crate::query::querybuilder::syntax::{
    ast::BaseAst,
    emitter::{AstProcessor, SqlEmitter},
    keyword::Keyword,
    tokens::{SqlTokens, ToSqlTokens},
};

impl<'a, T> EmitDelete<'a> for T
where
    T: SqlEmitter<'a>,
{
    fn emit_delete(
        &mut self,
        _ast: &impl AstProcessor<'a>,
        base_ast: &mut BaseAst<'a>,
    ) -> SqlTokens<'a> {
        let mut tokens = SqlTokens::default();

        tokens.keyword(Keyword::Delete);
        tokens.keyword(Keyword::From);
        tokens
            .extend(<TableMetadata<'_> as ToSqlTokens<'_, T::Dialect>>::to_tokens(&base_ast.table));

        if !base_ast.conditions.is_empty() {
            tokens.whitespace();
            for cond in &base_ast.conditions {
                tokens
                    .extend(<ConditionClause<'_> as ToSqlTokens<'_, T::Dialect>>::to_tokens(&cond));
            }
        }

        tokens
    }
}

pub trait EmitDelete<'a>: SqlEmitter<'a> {
    fn emit_delete(
        &mut self,
        ast: &impl AstProcessor<'a>,
        base_ast: &mut BaseAst<'a>,
    ) -> SqlTokens<'a>;
}

#[cfg(test)]
mod tests {
    use super::EmitDelete;
    use crate::query::querybuilder::syntax::dialect::MsSql;
    use crate::query::querybuilder::syntax::tokens::PlaceholderKind;
    use crate::query::querybuilder::syntax::writer::TokenWriter;
    use crate::query::querybuilder::syntax::{
        ast::BaseAst, ast::delete::DeleteAst, dialect::StandardDialect, emitter::SqlEmitter,
    };

    #[derive(Default)]
    struct TestDeleteEmitter;
    impl<'a> SqlEmitter<'a> for TestDeleteEmitter {
        type Dialect = StandardDialect;
    }

    #[derive(Default)]
    struct TestDeleteEmitterMsSql;
    impl<'a> SqlEmitter<'a> for TestDeleteEmitterMsSql {
        type Dialect = MsSql;
    }

    fn render_standard<'a>(ast: &SelectlessDeleteAst, base_ast: &mut BaseAst<'a>) -> String {
        let mut emitter = TestDeleteEmitter;
        let tokens = emitter.emit_delete(&ast.0, base_ast);
        TokenWriter::new()
            .render::<TestDeleteEmitter>(tokens)
            .unwrap()
    }

    fn render_mssql<'a>(ast: &SelectlessDeleteAst, base_ast: &mut BaseAst<'a>) -> String {
        let mut emitter = TestDeleteEmitterMsSql;
        let tokens = emitter.emit_delete(&ast.0, base_ast);
        TokenWriter::new()
            .render::<TestDeleteEmitterMsSql>(tokens)
            .unwrap()
    }

    struct SelectlessDeleteAst(DeleteAst);

    impl SelectlessDeleteAst {
        fn new(ast: DeleteAst) -> Self {
            Self(ast)
        }
    }

    #[test]
    fn emits_delete_from_table_without_conditions() {
        let ast = SelectlessDeleteAst::new(DeleteAst::default());

        let mut base_ast = BaseAst {
            table: "users".into(),
            ..Default::default()
        };

        let sql = render_standard(&ast, &mut base_ast);
        assert_eq!(sql.trim(), "DELETE FROM \"users\";");
    }

    #[test]
    fn emits_delete_from_table_without_conditions_in_mssql() {
        let ast = SelectlessDeleteAst::new(DeleteAst::default());

        let mut base_ast = BaseAst {
            table: "users".into(),
            ..Default::default()
        };

        let sql = render_mssql(&ast, &mut base_ast);
        assert_eq!(sql.trim(), "DELETE FROM [users];");
    }

    #[test]
    fn emits_delete_with_where_condition() {
        use crate::query::operators::Operator;
        use crate::query::querybuilder::syntax::clause::{ConditionClause, ConditionClauseKind};

        let ast = SelectlessDeleteAst::new(DeleteAst::default());

        let mut base_ast = BaseAst {
            table: "users".into(),
            ..Default::default()
        };

        base_ast.conditions.push(ConditionClause {
            kind: ConditionClauseKind::Where,
            column_name: "id".into(),
            operator: Operator::Eq,
            value_indexes: PlaceholderKind::Value(1),
        });

        let sql = render_standard(&ast, &mut base_ast);

        assert_eq!(sql.trim(), "DELETE FROM \"users\" WHERE \"id\" = $1;");
    }
}
