use crate::query::operators::{LikeKind, Operator};
use crate::query::querybuilder::syntax::column::ColumnRef;
use crate::query::querybuilder::syntax::dialect::SqlDialect;
use crate::query::querybuilder::syntax::emitter::types::helpers::Range;
use crate::query::querybuilder::syntax::keyword::Keyword;
use crate::query::querybuilder::syntax::symbol::Symbol;
use crate::query::querybuilder::syntax::tokens::{SqlToken, SqlTokens, ToSqlTokens};

pub struct ConditionClause<'a> {
    pub(crate) kind: ConditionClauseKind,
    pub(crate) column_name: ColumnRef<'a>,
    pub(crate) operator: Operator,
    pub(crate) value_indexes: Option<Range>,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum ConditionClauseKind {
    Where,
    And,
    In,
    Or,
    AndValuesIn,
    OrValuesIn,
}

impl From<ConditionClauseKind> for Keyword {
    fn from(keyword: ConditionClauseKind) -> Self {
        match keyword {
            ConditionClauseKind::Where => Keyword::Where,
            ConditionClauseKind::And | ConditionClauseKind::AndValuesIn => Keyword::And,
            ConditionClauseKind::Or | ConditionClauseKind::OrValuesIn => Keyword::Or,
            ConditionClauseKind::In => Keyword::In,
        }
    }
}

impl<'a, D: SqlDialect> ToSqlTokens<'a, D> for ConditionClause<'a> {
    fn to_tokens(&self) -> impl IntoIterator<Item = SqlToken<'a>> + 'a {
        let mut out = SqlTokens::with_capacity(4);

        // Clause keyword
        out.keyword(self.kind.into());

        // Column
        out.extend(<ColumnRef<'_> as ToSqlTokens<'_, D>>::to_tokens(
            &self.column_name,
        ));



        // Operator
        out.operator(self.operator);

        match self.operator {
            Operator::Like(kind) | Operator::NotLike(kind) => {
                let like_tokens = <LikeKind as ToSqlTokens<'_, D>>::to_tokens(&kind);
                out.extend(like_tokens);
            }
            _ => {
                if let Some(ref range) = self.value_indexes
                    && range.is_range()
                {
                    __impl::output_range_of_placeholders::<D>(range, &mut out);
                } else {

                    out.placeholder();
                }
            }
        }



        out
    }
}

mod __impl {
    use crate::query::operators::Operator;
    use crate::query::querybuilder::syntax::dialect::SqlDialect;
    use crate::query::querybuilder::syntax::emitter::types::helpers::Range;
    use crate::query::querybuilder::syntax::symbol::Symbol;
    use crate::query::querybuilder::syntax::tokens::{SqlTokens, ToSqlTokens};

    pub(crate) fn output_range_of_placeholders<D: SqlDialect>(
        range: &Range,
        out: &mut SqlTokens<'_>,
    ) {

        out.symbol(Symbol::LParen);
        let mut indexes = range.into_iter().peekable();
        while indexes.next().is_some() {
            out.placeholder();
            if indexes.peek().is_some() {
                out.symbol(Symbol::Comma);

            }
        }
        out.symbol(Symbol::RParen);
    }
}
