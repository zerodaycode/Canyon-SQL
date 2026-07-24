/// Blanket implementation of `SqlEmitter` for `SelectAst`.
/// This implementation provides a default plan for emitting SQL statements for
/// `SelectAst` nodes, since all the Canyon queries for now emit `SelectAst` nodes in the same way,
/// regardless of the backend dialect.
// impl<'a, T> SqlEmitter<'a, SelectAst<'a>> for T
//     where
//           T: SqlEmitter<'a, SelectAst<'a>>, {
//     type Dialect = <T as SqlEmitter<'a, SelectAst<'a>>>::Dialect;
//
//     const PLAN: &'a [EmitStep<'a, SelectAst<'a>>] = select_default_plan!(Self::Dialect);
// }

macro_rules! select_default_plan {
    ($dialect:ty) => {
        &[
            |ast, base_ast, tokens| {
                $crate::query::querybuilder::syntax::emitter::types::select::__impl::emit_select_keyword(ast, base_ast, tokens)
            },
            |ast, base_ast, tokens| {
                $crate::query::querybuilder::syntax::emitter::types::select::__impl::emit_distinct(ast, base_ast, tokens)
            },
            |ast, base_ast, tokens| {
                $crate::query::querybuilder::syntax::emitter::types::select::__impl::emit_columns::<$dialect>(ast, base_ast, tokens)
            },
            |ast, base_ast, tokens| {
                $crate::query::querybuilder::syntax::emitter::types::select::__impl::emit_from::<$dialect>(ast, base_ast, tokens)
            },
            |ast, base_ast, tokens| {
                $crate::query::querybuilder::syntax::emitter::types::select::__impl::emit_joins::<$dialect>(ast, base_ast, tokens)
            },
            |ast, base_ast, tokens| {
                $crate::query::querybuilder::syntax::emitter::types::select::__impl::emit_conditions::<$dialect>(ast, base_ast, tokens)
            },
            |ast, base_ast, tokens| {
                $crate::query::querybuilder::syntax::emitter::types::select::__impl::emit_group_by::<$dialect>(ast, base_ast, tokens)
            },
            |ast, base_ast, tokens| {
                $crate::query::querybuilder::syntax::emitter::types::select::__impl::emit_having::<$dialect>(ast, base_ast, tokens)
            },
            |ast, base_ast, tokens| {
                $crate::query::querybuilder::syntax::emitter::types::select::__impl::emit_order_by::<$dialect>(ast, base_ast, tokens)
            },
            |ast, base_ast, tokens| {
                $crate::query::querybuilder::syntax::emitter::types::select::__impl::emit_limit(ast, base_ast, tokens)
            },
            |ast, base_ast, tokens| {
                $crate::query::querybuilder::syntax::emitter::types::select::__impl::emit_offset(ast, base_ast, tokens)
            },
        ]
    };
}

pub(crate) use select_default_plan;

pub(crate) mod __impl {
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

    pub(crate) fn emit_select_keyword<'a>(
        _ast: &SelectAst<'a>,
        _base_ast: &mut BaseAst<'a>,
        tokens: &mut SqlTokens<'a>,
    ) {
        tokens.keyword(Keyword::Select);
    }

    pub(crate) fn emit_columns<'a, D: SqlDialect>(
        ast: &SelectAst<'a>,
        _base_ast: &mut BaseAst<'a>,
        tokens: &mut SqlTokens<'a>,
    ) {
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

    pub(crate) fn emit_from<'a, D: SqlDialect>(
        _ast: &SelectAst<'a>,
        base_ast: &mut BaseAst<'a>,
        tokens: &mut SqlTokens<'a>,
    ) {
        tokens.keyword(Keyword::From);
        tokens.extend(<TableMetadata<'a> as ToSqlTokens<'a, D>>::to_tokens(
            base_ast.table(),
        ));
    }

    pub(crate) fn emit_joins<'a, D: SqlDialect>(
        ast: &SelectAst<'a>,
        _base_ast: &mut BaseAst<'a>,
        tokens: &mut SqlTokens<'a>,
    ) {
        for join in &ast.joins {
            tokens.extend(<JoinClause<'a> as ToSqlTokens<'a, D>>::to_tokens(join));
        }
    }

    pub(crate) fn emit_conditions<'a, D>(
        _ast: &SelectAst<'a>,
        base_ast: &mut BaseAst<'a>,
        tokens: &mut SqlTokens<'a>,
    ) where
        D: SqlDialect,
    {
        helpers::emit_query_conditions::<D>(base_ast.conditions(), tokens);
    }

    pub(crate) fn emit_group_by<'a, D: SqlDialect>(
        ast: &SelectAst<'a>,
        _base_ast: &mut BaseAst<'a>,
        tokens: &mut SqlTokens<'a>,
    ) {
        if let Some(group_by) = &ast.group_by {
            tokens.keyword(Keyword::GroupBy);
            helpers::emit_columns::<D>(group_by, tokens);
        }
    }

    pub(crate) fn emit_having<'a, D: SqlDialect>(
        ast: &SelectAst<'a>,
        _base_ast: &mut BaseAst<'a>,
        tokens: &mut SqlTokens<'a>,
    ) {
        if let Some(having) = &ast.having {
            tokens.keyword(Keyword::Having);
            tokens.extend(<HavingClause<'_> as ToSqlTokens<'_, D>>::to_tokens(having));
        }
    }

    pub(crate) fn emit_order_by<'a, D: SqlDialect>(
        ast: &SelectAst<'a>,
        _base_ast: &mut BaseAst<'a>,
        tokens: &mut SqlTokens<'a>,
    ) {
        if let Some(order_by) = &ast.order_by {
            tokens.extend(<OrderByClause<'_> as ToSqlTokens<'_, D>>::to_tokens(
                order_by,
            ));
        }
    }

    pub(crate) fn emit_limit<'a>(
        ast: &SelectAst<'a>,
        _base_ast: &mut BaseAst<'a>,
        tokens: &mut SqlTokens<'a>,
    ) {
        if let Some(limit) = ast.limit {
            tokens.keyword(Keyword::Limit);
            tokens.numeric(limit);
        }
    }

    pub(crate) fn emit_offset<'a>(
        ast: &SelectAst<'a>,
        _base_ast: &mut BaseAst<'a>,
        tokens: &mut SqlTokens<'a>,
    ) {
        if let Some(offset) = ast.offset {
            tokens.keyword(Keyword::Offset);
            tokens.numeric(offset);
        }
    }

    pub(crate) fn emit_distinct<'a>(
        ast: &SelectAst<'a>,
        _base_ast: &mut BaseAst<'a>,
        tokens: &mut SqlTokens<'a>,
    ) {
        if ast.with_distinct {
            tokens.keyword(Keyword::Distinct);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::query::querybuilder::syntax::dialect::PgDialect;
    use crate::query::querybuilder::syntax::emitter::EmitStep;
    use crate::query::{
        operators::Operator,
        querybuilder::syntax::{
            ast::BaseAst, ast::select::SelectAst, column::ColumnRef, dialect::StandardDialect,
            emitter::SqlEmitter, order::OrderByClause, writer::TokenWriter,
        },
    };

    struct TestEmitter;
    impl<'a> SqlEmitter<'a, SelectAst<'a>> for TestEmitter {
        type Dialect = PgDialect;

        const PLAN: &'a [EmitStep<'a, SelectAst<'a>>] = select_default_plan!(Self::Dialect);
    }

    fn col(name: &'_ str) -> ColumnRef<'_> {
        ColumnRef::from(name)
    }

    fn render<'a>(ast: &SelectAst<'a>, base_ast: &mut BaseAst<'a>) -> String {
        let mut emitter = TestEmitter;
        let tokens = emitter.emit(ast, base_ast);
        TokenWriter::new().render::<PgDialect>(tokens).unwrap()
    }

    #[test]
    fn emits_select_with_columns_and_from() {
        let mut ast = SelectAst::new();
        ast.columns = vec![col("id"), col("name")];

        let mut base_ast = BaseAst::new_ast("users".into());

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

        let mut base_ast = BaseAst::new_ast("users".into());

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

        let mut base_ast = BaseAst::new_ast("users".into());

        let sql = render(&ast, &mut base_ast);
        assert_eq!(sql, "SELECT * FROM \"users\";");
    }

    #[test]
    fn emits_group_by_when_present() {
        let mut ast = SelectAst::new();
        ast.columns = vec![col("users.country")];
        ast.group_by = Some(vec![col("users.country")]);

        let mut base_ast = BaseAst::new_ast("users".into());

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

        let mut base_ast = BaseAst::new_ast("users".into());

        let sql = render(&ast, &mut base_ast);
        assert_eq!(
            sql,
            "SELECT \"users\".\"id\", \"profiles\".\"bio\", \"roles\".\"name\" FROM \"users\" INNER JOIN \"profiles\" ON \"users\".\"id\" = \"profiles\".\"user_id\" LEFT JOIN \"roles\" ON \"users\".\"role_id\" = \"roles\".\"id\" RIGHT JOIN \"teams\" ON \"users\".\"team_id\" = \"teams\".\"id\" FULL OUTER JOIN \"permissions\" ON \"users\".\"id\" = \"permissions\".\"user_id\";"
        );
    }
}
