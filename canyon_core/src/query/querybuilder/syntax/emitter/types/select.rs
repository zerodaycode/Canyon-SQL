use crate::query::querybuilder::syntax::ast::BaseAst;
use crate::query::querybuilder::syntax::ast::select::SelectAst;
use crate::query::querybuilder::syntax::clause::ConditionClause;
use crate::query::querybuilder::syntax::emitter::{AstProcessor, SqlEmitter};
use crate::query::querybuilder::syntax::keyword::Keyword;
use crate::query::querybuilder::syntax::tokens::{SqlTokens, ToSqlTokens};

/// Blanket implementation for all the SqlEmitter implementors that are able to generate
/// SELECT like SQL clauses
impl<'a, T> EmitSelect<'a> for T
where
    T: SqlEmitter<'a>,
{
    fn emit_select(
        &mut self,
        ast: &impl AstProcessor<'a>,
        base_ast: &mut BaseAst<'a>,
    ) -> SqlTokens<'a> {
        let mut tokens = SqlTokens::default();

        let select_ast = transient::Downcast::downcast_ref::<SelectAst>(ast.as_any()).expect(
            "[emitSelect] - Handle this propagating result and introducing custom error types",
        );

        tokens.keyword(Keyword::Select);
        __impl::emit_distinct(select_ast, &mut tokens);

        __impl::emit_columns::<T::Dialect>(select_ast, &mut tokens);
        __impl::emit_from::<T::Dialect>(base_ast, &mut tokens);
        __impl::emit_joins::<T::Dialect>(select_ast, &mut tokens);

        for condition in &base_ast.conditions {
            tokens
                .extend(<ConditionClause<'a> as ToSqlTokens<'a, T::Dialect>>::to_tokens(condition));
        }

        __impl::emit_group_by::<T::Dialect>(select_ast, &mut tokens);
        __impl::emit_having::<T::Dialect>(select_ast, &mut tokens);
        __impl::emit_order_by::<T::Dialect>(select_ast, &mut tokens);
        __impl::emit_limit(select_ast, &mut tokens);
        __impl::emit_offset(select_ast, &mut tokens);

        tokens
    }
}

/// Emits a `SELECT` query from the provided AST nodes.
///
/// This trait is implemented as a blanket implementation for any type that
/// implements [`SqlEmitter`]. It converts the high level [`SelectAst`] plus the
/// shared [`BaseAst`] information into a sequence of [`SqlTokens`].
///
/// The emission process is intentionally split into small stateless helpers
/// to keep the codegen predictable and allow the compiler to aggressively
/// inline them.
///
/// Expected clause order:
///
/// SELECT
///   -> columns
///   -> FROM
///   -> JOINs
///   -> GROUP BY
///   -> HAVING
///   -> ORDER BY
///   -> LIMIT
///   -> OFFSET
///
/// `BaseAst` contains elements shared across query kinds (for example the
/// root table), while `SelectAst` contains clauses specific to SELECT queries.
///
/// Implementors normally do not override this method and instead rely on the
/// default blanket implementation.
pub trait EmitSelect<'a>: SqlEmitter<'a> {
    fn emit_select(
        &mut self,
        ast: &impl AstProcessor<'a>,
        base_ast: &mut BaseAst<'a>,
    ) -> SqlTokens<'a>;
}

mod __impl {
    use crate::query::querybuilder::syntax::ast::BaseAst;
    use crate::query::querybuilder::syntax::ast::select::SelectAst;
    use crate::query::querybuilder::syntax::dialect::SqlDialect;
    use crate::query::querybuilder::syntax::emitter::types::helpers;
    use crate::query::querybuilder::syntax::having::HavingClause;
    use crate::query::querybuilder::syntax::join::JoinClause;
    use crate::query::querybuilder::syntax::keyword::Keyword;
    use crate::query::querybuilder::syntax::order::OrderByClause;
    use crate::query::querybuilder::syntax::symbol::Symbol;
    use crate::query::querybuilder::syntax::table_metadata::TableMetadata;
    use crate::query::querybuilder::syntax::tokens::{SqlTokens, ToSqlTokens};

    pub(crate) fn emit_columns<'a, D: SqlDialect>(ast: &SelectAst<'a>, tokens: &mut SqlTokens<'a>) {
        let is_count_query = ast.is_count_query;
        if is_count_query {
            tokens.keyword(Keyword::Count);
            tokens.symbol(Symbol::LParen);
        }
        helpers::emit_columns::<D>(&ast.columns, tokens);
        if is_count_query {
            tokens.symbol(Symbol::RParen);
        }
    }

    pub(crate) fn emit_from<'a, D: SqlDialect>(base_ast: &BaseAst<'a>, tokens: &mut SqlTokens<'a>) {
        tokens.keyword(Keyword::From);
        tokens.extend(<TableMetadata<'a> as ToSqlTokens<'a, D>>::to_tokens(
            &base_ast.table,
        ));
    }

    pub(crate) fn emit_joins<'a, D: SqlDialect>(ast: &SelectAst<'a>, tokens: &mut SqlTokens<'a>) {
        for join in &ast.joins {
            tokens.extend(<JoinClause<'a> as ToSqlTokens<'a, D>>::to_tokens(join));
        }
    }

    pub(crate) fn emit_group_by<'a, D: SqlDialect>(
        ast: &SelectAst<'a>,
        tokens: &mut SqlTokens<'a>,
    ) {
        if let Some(group_by) = &ast.group_by {
            tokens.keyword(Keyword::GroupBy);
            helpers::emit_columns::<D>(group_by, tokens);
        }
    }

    pub(crate) fn emit_having<'a, D: SqlDialect>(ast: &SelectAst<'a>, tokens: &mut SqlTokens<'a>) {
        if let Some(having) = &ast.having {
            tokens.keyword(Keyword::Having);
            tokens.extend(<HavingClause<'_> as ToSqlTokens<'_, D>>::to_tokens(having));
        }
    }

    pub(crate) fn emit_order_by<'a, D: SqlDialect>(
        ast: &SelectAst<'a>,
        tokens: &mut SqlTokens<'a>,
    ) {
        if let Some(order_by) = &ast.order_by {
            tokens.extend(<OrderByClause<'_> as ToSqlTokens<'_, D>>::to_tokens(
                order_by,
            ));
        }
    }

    pub(crate) fn emit_limit<'a>(ast: &SelectAst<'a>, tokens: &mut SqlTokens<'a>) {
        if let Some(limit) = ast.limit {
            tokens.keyword(Keyword::Limit);
            tokens.numeric(limit);
        }
    }

    pub(crate) fn emit_offset<'a>(ast: &SelectAst<'a>, tokens: &mut SqlTokens<'a>) {
        if let Some(offset) = ast.offset {
            tokens.keyword(Keyword::Offset);
            tokens.numeric(offset);
        }
    }

    pub(crate) fn emit_distinct(ast: &SelectAst, tokens: &mut SqlTokens) {
        if ast.with_distinct {
            tokens.keyword(Keyword::Distinct);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::query::{
        operators::Operator,
        querybuilder::syntax::{
            ast::BaseAst, ast::select::SelectAst, column::ColumnRef, dialect::StandardDialect,
            emitter::SqlEmitter, emitter::types::select::EmitSelect, order::OrderByClause,
            writer::TokenWriter,
        },
    };

    struct TestEmitter;
    impl<'a> SqlEmitter<'a> for TestEmitter {
        type Dialect = StandardDialect;
    }

    fn col(name: &'_ str) -> ColumnRef<'_> {
        ColumnRef::from(name)
    }

    fn render<'a>(ast: &SelectAst<'a>, base_ast: &mut BaseAst<'a>) -> String {
        let mut emitter = TestEmitter;
        let tokens = emitter.emit_select(ast, base_ast);
        TokenWriter::new().render::<TestEmitter>(tokens).unwrap()
    }

    #[test]
    fn emits_select_with_columns_and_from() {
        let mut ast = SelectAst::new();
        ast.columns = vec![col("id"), col("name")];

        let mut base_ast = BaseAst {
            table: "users".into(),
            ..Default::default()
        };

        let sql = render(&ast, &mut base_ast);
        assert_eq!(sql, "SELECT \"id\", \"name\" FROM \"users\";");
    }

    #[test]
    fn emits_select_with_order_by_limit_and_offset() {
        let mut ast = SelectAst::new();
        ast.columns = vec![col("id")];
        ast.order_by = Some(OrderByClause::new("id", true));
        ast.limit = Some(10);
        ast.offset = Some(20);

        let mut base_ast = BaseAst {
            table: "users".into(),
            ..Default::default()
        };

        let sql = render(&ast, &mut base_ast);
        assert_eq!(
            sql,
            "SELECT \"id\" FROM \"users\" ORDER BY \"id\" DESC LIMIT 10 OFFSET 20;"
        );
    }

    #[test]
    fn emits_select_without_optional_clauses() {
        let mut ast = SelectAst::new();
        ast.columns = vec![];

        let mut base_ast = BaseAst {
            table: "users".into(),
            ..Default::default()
        };

        let sql = render(&ast, &mut base_ast);
        assert_eq!(sql, "SELECT * FROM \"users\";");
    }

    #[test]
    fn emits_group_by_when_present() {
        let mut ast = SelectAst::new();
        ast.columns = vec![col("users.country")];
        ast.group_by = Some(vec![col("users.country")]);

        let mut base_ast = BaseAst {
            table: "users".into(),
            ..Default::default()
        };

        let sql = render(&ast, &mut base_ast);
        assert_eq!(
            sql,
            "SELECT \"users\".\"country\" FROM \"users\" GROUP BY \"users\".\"country\";"
        );
    }

    #[test]
    fn emits_select_with_all_join_kinds() {
        use crate::query::querybuilder::syntax::join::{JoinClause, JoinKind};

        let mut ast = SelectAst::new();
        ast.columns = vec![col("users.id"), col("profiles.bio"), col("roles.name")];

        ast.joins = vec![
            JoinClause::new(
                JoinKind::Inner,
                "profiles".into(),
                col("users.id"),
                Operator::Eq,
                col("profiles.user_id"),
            ),
            JoinClause::new(
                JoinKind::Left,
                "roles".into(),
                col("users.role_id"),
                Operator::Eq,
                col("roles.id"),
            ),
            JoinClause::new(
                JoinKind::Right,
                "teams".into(),
                col("users.team_id"),
                Operator::Eq,
                col("teams.id"),
            ),
            JoinClause::new(
                JoinKind::FullOuter,
                "permissions".into(),
                col("users.id"),
                Operator::Eq,
                col("permissions.user_id"),
            ),
        ];

        let mut base_ast = BaseAst {
            table: "users".into(),
            ..Default::default()
        };

        let sql = render(&ast, &mut base_ast);
        assert_eq!(
            sql,
            "SELECT \"users\".\"id\", \"profiles\".\"bio\", \"roles\".\"name\" FROM \"users\" INNER JOIN \"profiles\" ON \"users\".\"id\" = \"profiles\".\"user_id\" LEFT JOIN \"roles\" ON \"users\".\"role_id\" = \"roles\".\"id\" RIGHT JOIN \"teams\" ON \"users\".\"team_id\" = \"teams\".\"id\" FULL OUTER JOIN \"permissions\" ON \"users\".\"id\" = \"permissions\".\"user_id\";"
        );
    }
}
