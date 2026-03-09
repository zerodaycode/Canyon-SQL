use crate::query::operators::Comp;
use crate::query::querybuilder::syntax::column::ColumnRef;
use crate::query::querybuilder::syntax::keyword::Keyword;
use crate::query::querybuilder::syntax::table_metadata::TableMetadata;
use crate::query::querybuilder::syntax::tokens::{SqlToken, SqlTokens, ToSqlTokens};

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
    pub operator: Comp, // usually Eq
    pub right: ColumnRef<'a>, // e.g. "t2.t1_id" // TODO: this is always the base or the previous (at least, in one of the sides)
                              // so we could look in the vector for the previous clause and auto-add the join
}

impl<'a> JoinClause<'a> {
    pub const fn new(
        kind: JoinKind,
        target_table: TableMetadata<'a>,
        left: ColumnRef<'a>,
        operator: Comp,
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

impl<'a> ToSqlTokens<'a> for JoinClause<'a> {
    fn to_tokens(&self) -> impl IntoIterator<Item = SqlToken<'a>> + 'a {
        let mut out = SqlTokens::with_capacity(6);

        out.ident(self.kind.as_str()); // NOTE: dubious
        out.extend(self.target_table.to_tokens());
        out.keyword(Keyword::On);
        out.extend(self.left.to_tokens());
        out.operator(self.operator);
        out.extend(self.right.to_tokens());

        out
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
    tokens.extend(join.to_tokens());

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
