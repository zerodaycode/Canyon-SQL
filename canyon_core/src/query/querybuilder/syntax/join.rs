use crate::query::operators::Comp;
use crate::query::querybuilder::syntax::column::ColumnRef;
use crate::query::querybuilder::syntax::keyword::Keyword;
use crate::query::querybuilder::syntax::table_metadata::TableMetadata;
use crate::query::querybuilder::syntax::tokens::{SqlTokens, ToSqlTokens};

#[derive(Debug, Clone, Copy)]
pub enum JoinKind {
    Inner,
    Left,
    Right,
    Full,
}

impl JoinKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            JoinKind::Inner => "INNER JOIN",
            JoinKind::Left => "LEFT JOIN",
            JoinKind::Right => "RIGHT JOIN",
            JoinKind::Full => "FULL JOIN",
        }
    }
}

pub struct JoinClause<'a> {
    pub kind: JoinKind,
    pub target_table: TableMetadata<'a>,
    pub left: ColumnRef<'a>,
    pub operator: Comp,      // usually Eq
    pub right: ColumnRef<'a>, // e.g. "t2.t1_id" // TODO: this is always the base or the previous (at least, in one of the sides)
                              // so we could look in the vector for the previous clause and auto-add the join
}

impl<'a> JoinClause<'a> {
    pub fn new<I: Into<ColumnRef<'a>>>(
        kind: JoinKind,
        target_table: TableMetadata<'a>,
        left: I,
        operator: Comp,
        right: I,
    ) -> Self {
        Self {
            kind,
            target_table,
            left: left.into(),
            operator,
            right: right.into(),
        }
    }
}

impl<'a> ToSqlTokens<'a> for JoinClause<'a> {
    fn to_tokens(&self, out: &mut SqlTokens<'a>) {
        out.ident(self.kind.as_str()); // NOTE: dubious
        self.target_table.to_tokens(out);
        out.keyword(Keyword::On);
        self.left.to_tokens(out);
        out.operator(self.operator);
        self.right.to_tokens(out);
    }
}
#[test]
fn test_join_clause_basic() {
    use crate::query::operators::Comp;
    use crate::query::querybuilder::syntax::tokens::{SqlToken, Symbol};

    let mock_value = 99;

    let join = JoinClause {
        kind: JoinKind::Inner,
        target_table: TableMetadata::new("users"),
        left: ColumnRef::from("t.id"),
        operator: Comp::Eq,
        right: "users.team_id".into(),
    };

    let mut tokens = SqlTokens::default();
    join.to_tokens(&mut tokens);

    let expected = vec![
        SqlToken::Keyword(Keyword::Inner),
        SqlToken::Keyword(Keyword::Join),
        SqlToken::Ident("users".into()),
        SqlToken::Keyword(Keyword::On),

        SqlToken::Ident("t".into()),
        SqlToken::Symbol(Symbol::Dot),
        SqlToken::Ident("id".into()),

        SqlToken::Symbol(Symbol::Equals),

        SqlToken::Ident("users".into()),
        SqlToken::Symbol(Symbol::Dot),
        SqlToken::Ident("team_id".into()),
    ];

    assert_eq!(tokens.inner(), expected);
}
