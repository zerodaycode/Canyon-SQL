use crate::query::querybuilder::syntax::emitter::types::helpers;
use crate::query::querybuilder::syntax::table_metadata::TableMetadata;
use crate::query::querybuilder::syntax::{
    ast::BaseAst,
    emitter::{AstProcessor, SqlEmitter},
    keyword::Keyword,
    tokens::{SqlTokens, ToSqlTokens},
};

pub(crate) fn general_delete_impl<'a, T, P>(_ast: &P, base_ast: &mut BaseAst<'a>) -> SqlTokens<'a>
where
    T: SqlEmitter<'a, P>,
    P: AstProcessor<'a> + 'a,
{
    let mut tokens = SqlTokens::default();

    tokens.keyword(Keyword::Delete);
    tokens.keyword(Keyword::From);
    tokens.extend(<TableMetadata<'_> as ToSqlTokens<'_, T::Dialect>>::to_tokens(&base_ast.table));

    helpers::emit_query_conditions::<T::Dialect>(&base_ast.conditions, &mut tokens);

    tokens
}

pub trait EmitDelete<'a, T, P>
where
    T: SqlEmitter<'a, P>,
    P: AstProcessor<'a> + 'a,
{
    fn emit_delete(&mut self, ast: &P, base_ast: &mut BaseAst<'a>) -> SqlTokens<'a> {
        general_delete_impl::<T, P>(&ast, base_ast)
    }
}

#[cfg(test)]
mod tests {
    use crate::query::querybuilder::syntax::{
        ast::{BaseAst, delete::DeleteAst},
        dialect::{MsSql, StandardDialect},
        emitter::{AstProcessor, SqlEmitter, types::helpers::Range},
        writer::TokenWriter,
    };

    #[derive(Default)]
    struct TestDeleteEmitter;

    impl<'a> SqlEmitter<'a, DeleteAst> for TestDeleteEmitter {
        type Dialect = StandardDialect;

        const PLAN: &'a [crate::query::querybuilder::syntax::emitter::EmitStep<'a, DeleteAst>] =
            &[]; // TODO: Implement for DeleteAst
    }

    #[derive(Default)]
    struct TestDeleteEmitterMsSql;
    impl<'a> SqlEmitter<'a, DeleteAst> for TestDeleteEmitterMsSql {
        type Dialect = MsSql;

        const PLAN: &'a [crate::query::querybuilder::syntax::emitter::EmitStep<'a, DeleteAst>] =
            &[]; // TODO: Implement for DeleteAst
    }

    fn render_standard<'a>(ast: &DeleteAst, base_ast: &mut BaseAst<'a>) -> String {
        let mut emitter = TestDeleteEmitter;
        let tokens = emitter.emit(ast, base_ast);
        TokenWriter::new()
            .render::<StandardDialect>(tokens)
            .unwrap()
    }

    fn render_mssql<'a>(ast: &DeleteAst, base_ast: &mut BaseAst<'a>) -> String {
        let mut emitter = TestDeleteEmitterMsSql;
        let tokens = emitter.emit(ast, base_ast);
        TokenWriter::new().render::<MsSql>(tokens).unwrap()
    }

    #[test]
    fn emits_delete_from_table_without_conditions() {
        let ast = DeleteAst::default();

        let mut base_ast = BaseAst {
            table: "users".into(),
            ..Default::default()
        };

        let sql = render_standard(&ast, &mut base_ast);
        assert_eq!(sql.trim(), "DELETE FROM \"users\";");
    }

    #[test]
    fn emits_delete_from_table_without_conditions_in_mssql() {
        let ast = DeleteAst::default();

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

        let ast = DeleteAst::default();

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

        assert_eq!(sql.trim(), "DELETE FROM \"users\" WHERE \"id\" = $1;");
    }
}
