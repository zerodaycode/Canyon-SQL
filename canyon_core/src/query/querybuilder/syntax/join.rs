use crate::query::operators::Operator;
use crate::query::querybuilder::syntax::column::ColumnRef;
use crate::query::querybuilder::syntax::dialect::SqlDialect;
use crate::query::querybuilder::syntax::keyword::Keyword;
use crate::query::querybuilder::syntax::table_metadata::TableMetadata;
use crate::query::querybuilder::syntax::tokens::{SqlToken, SqlTokens, ToSqlTokens};

#[derive(Debug, Clone, Copy)]
pub enum JoinKind {
    Inner,
    Left,
    Right,
    Full,
    FullOuter,
}

impl From<JoinKind> for Keyword {
    fn from(join_kind: JoinKind) -> Self {
        match join_kind {
            JoinKind::Inner => Keyword::Inner,
            JoinKind::Left => Keyword::Left,
            JoinKind::Right => Keyword::Right,
            JoinKind::Full => Keyword::Full,
            JoinKind::FullOuter => Keyword::FullOuter,
        }
    }
}

pub struct JoinClause<'a> {
    pub kind: JoinKind,
    pub target_table: TableMetadata<'a>,
    pub left: ColumnRef<'a>,
    pub operator: Operator, // usually Eq
    pub right: ColumnRef<'a>, // e.g. "t2.t1_id" // TODO: this is always the base or the previous (at least, in one of the sides)
                              // so we could look in the vector for the previous clause and auto-add the join
}

impl<'a> JoinClause<'a> {
    pub const fn new(
        kind: JoinKind,
        target_table: TableMetadata<'a>,
        left: ColumnRef<'a>,
        operator: Operator,
        right: ColumnRef<'a>,
    ) -> Self {
        Self {
            kind,
            target_table,
            left,
            operator,
            right,
        }
    }
}

impl<'a, D: SqlDialect> ToSqlTokens<'a, D> for JoinClause<'a> {
    fn to_tokens(&self) -> impl IntoIterator<Item = SqlToken<'a>> + 'a {
        let mut out = SqlTokens::with_capacity(6);

        out.keyword(self.kind.into());
        out.keyword(Keyword::Join);
        out.extend(<TableMetadata<'a> as ToSqlTokens<'a, D>>::to_tokens(
            &self.target_table,
        ));

        out.keyword(Keyword::On);
        out.extend(<ColumnRef<'a> as ToSqlTokens<'a, D>>::to_tokens(&self.left));

        out.operator(self.operator);

        out.extend(<ColumnRef<'a> as ToSqlTokens<'a, D>>::to_tokens(
            &self.right,
        ));

        out
    }
}
#[test]
fn test_join_clause_basic() {
    use crate::query::operators::Operator;
    use crate::query::querybuilder::syntax::dialect::StandardDialect;
    use crate::query::querybuilder::syntax::tokens::{SqlToken, Symbol};

    let join = JoinClause::new(
        JoinKind::Inner,
        TableMetadata::new("users"),
        ColumnRef::from("t.id"),
        Operator::Eq,
        "users.team_id".into(),
    );

    let mut tokens = SqlTokens::default();
    tokens.extend(<JoinClause<'_> as ToSqlTokens<'_, StandardDialect>>::to_tokens(&join));

    let expected = vec![
        SqlToken::Keyword(Keyword::Inner),
        SqlToken::Keyword(Keyword::Join),
        SqlToken::Symbol(Symbol::DoubleQuote),
        SqlToken::Ident("users".into()),
        SqlToken::Symbol(Symbol::DoubleQuote),
        SqlToken::Keyword(Keyword::On),
        SqlToken::Symbol(Symbol::DoubleQuote),
        SqlToken::Ident("t".into()),
        SqlToken::Symbol(Symbol::DoubleQuote),
        SqlToken::Symbol(Symbol::Dot),
        SqlToken::Symbol(Symbol::DoubleQuote),
        SqlToken::Ident("id".into()),
        SqlToken::Symbol(Symbol::DoubleQuote),
        SqlToken::Operator(Operator::Eq),
        SqlToken::Symbol(Symbol::DoubleQuote),
        SqlToken::Ident("users".into()),
        SqlToken::Symbol(Symbol::DoubleQuote),
        SqlToken::Symbol(Symbol::Dot),
        SqlToken::Symbol(Symbol::DoubleQuote),
        SqlToken::Ident("team_id".into()),
        SqlToken::Symbol(Symbol::DoubleQuote),
    ];

    assert_eq!(tokens.inner(), expected);
}
