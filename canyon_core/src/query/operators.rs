use crate::query::querybuilder::syntax::{
    dialect::SqlDialect,
    keyword::Keyword,
    tokens::{PlaceholderKind, SqlToken, SqlTokens, Symbol, ToSqlTokens},
};
use std::borrow::Cow;
use std::fmt::Display;
use crate::query::querybuilder::syntax::dialect::PlaceholderDatatype;

/// Enumerated type for represent the available operators
/// in SQL sentences
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Operator {
    /// Operator "=" equals
    Eq,
    /// Operator "!=" not equals
    Neq,
    /// Operator ">" greater than value
    Gt,
    /// Operator ">=" greater or equals than value
    GtEq,
    /// Operator "<" less than value
    Lt,
    /// Operator "=<" less or equals than value
    LtEq,
    /// A "LIKE" comp operator
    Like(LikeKind),
    /// A "NOT LIKE" comp operator
    NotLike(LikeKind),
    /// Operator "IN" for value in (value1, value2, ...)
    In,
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let op = match *self {
            Self::Eq => "=",
            Self::Neq => "<>",
            Self::Gt => ">",
            Self::GtEq => ">=",
            Self::Lt => "<",
            Self::LtEq => "<=",
            Self::Like(ref __kind) => "LIKE",
            Self::NotLike(ref __kind) => "NOT LIKE",
            Self::In => "IN",
        };
        write!(f, "{}", op)
    }
}

impl<'a, D: SqlDialect> ToSqlTokens<'a, D> for Operator {
    fn to_tokens(&self) -> impl IntoIterator<Item = SqlToken<'a>> + 'a {
        let mut out = SqlTokens::with_capacity(16);

        match *self {
            Self::Eq => out.symbol(Symbol::Equals),
            Self::Neq => {
                out.symbol(Symbol::Not);
                out.symbol(Symbol::Equals);
            }
            Self::Gt => out.symbol(Symbol::RAngle),
            Self::GtEq => {
                out.symbol(Symbol::RAngle);
                out.symbol(Symbol::Equals);
            }
            Self::Lt => out.symbol(Symbol::LAngle),
            Self::LtEq => {
                out.symbol(Symbol::LAngle);
                out.symbol(Symbol::Equals);
            }
            Self::Like(kind) => out.extend(<LikeKind as ToSqlTokens<'_, D>>::to_tokens(&kind)),
            Self::NotLike(kind) => {
                out.keyword(Keyword::Not);
                out.whitespace();
                out.extend(<LikeKind as ToSqlTokens<'_, D>>::to_tokens(&kind));
            }
            Self::In => out.keyword(Keyword::In),
        }

        out
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum LikeKind {
    /// Operator `LIKE` as `%pattern%`.
    Full,
    /// Operator `LIKE` as `%pattern`.
    Left,
    /// Operator `LIKE` as `pattern%`.
    Right,
}

impl LikeKind {
    #[inline]
    fn push_casted_placeholder<D: SqlDialect>(out: &mut SqlTokens) {
        out.keyword(Keyword::Cast);
        out.symbol(Symbol::LParen);
        out.placeholder(PlaceholderKind::Value(1));
        out.whitespace();
        out.keyword(Keyword::As);
        out.whitespace();
        out.ident(Cow::from(<PlaceholderDatatype as Into<&str>>::into(D::PLACEHOLDER_DATA_TYPE.into())));
        out.symbol(Symbol::RParen);
    }

    #[inline]
    fn push_percent_literal(out: &mut SqlTokens) {
        out.symbol(Symbol::Quote);
        out.symbol(Symbol::PercentSign);
        out.symbol(Symbol::Quote);
    }

    #[inline]
    fn push_comma_sep(out: &mut SqlTokens) {
        out.symbol(Symbol::Comma);
        out.whitespace();
    }
}

impl<'a, D: SqlDialect> ToSqlTokens<'a, D> for LikeKind {
    fn to_tokens(&self) -> impl IntoIterator<Item = SqlToken<'a>> + 'a {
        let mut out = SqlTokens::with_capacity(19);

        out.keyword(Keyword::Like);
        out.whitespace();
        out.keyword(Keyword::Concat);
        out.symbol(Symbol::LParen);

        match *self {
            Self::Full => {
                Self::push_percent_literal(&mut out);
                Self::push_comma_sep(&mut out);
                Self::push_casted_placeholder::<D>(&mut out);
                Self::push_comma_sep(&mut out);
                Self::push_percent_literal(&mut out);
            }
            Self::Left => {
                Self::push_percent_literal(&mut out);
                Self::push_comma_sep(&mut out);
                Self::push_casted_placeholder::<D>(&mut out);
            }
            Self::Right => {
                Self::push_casted_placeholder::<D>(&mut out);
                Self::push_comma_sep(&mut out);
                Self::push_percent_literal(&mut out);
            }
        }

        out.symbol(Symbol::RParen);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "postgres")]
    use crate::query::querybuilder::syntax::dialect::PgDialect;
    use crate::query::querybuilder::syntax::dialect::PlaceholderDatatype;

    fn tokens<D, T>(value: T) -> Vec<SqlToken<'static>>
    where
        D: SqlDialect,
        T: ToSqlTokens<'static, D>,
    {
        value.to_tokens().into_iter().collect()
    }

    fn full_like_tokens<D: SqlDialect>() -> Vec<SqlToken<'static>> {
        vec![
            SqlToken::Keyword(Keyword::Like),
            SqlToken::WhiteSpace,
            SqlToken::Keyword(Keyword::Concat),
            SqlToken::Symbol(Symbol::LParen),
            SqlToken::Symbol(Symbol::Quote),
            SqlToken::Symbol(Symbol::PercentSign),
            SqlToken::Symbol(Symbol::Quote),
            SqlToken::Symbol(Symbol::Comma),
            SqlToken::WhiteSpace,
            SqlToken::Keyword(Keyword::Cast),
            SqlToken::Symbol(Symbol::LParen),
            SqlToken::Placeholder(PlaceholderKind::Value(1)),
            SqlToken::WhiteSpace,
            SqlToken::Keyword(Keyword::As),
            SqlToken::WhiteSpace,
            SqlToken::Ident(Cow::from(<PlaceholderDatatype as Into<&str>>::into(D::PLACEHOLDER_DATA_TYPE.into()))),
            SqlToken::Symbol(Symbol::RParen),
            SqlToken::Symbol(Symbol::Comma),
            SqlToken::WhiteSpace,
            SqlToken::Symbol(Symbol::Quote),
            SqlToken::Symbol(Symbol::PercentSign),
            SqlToken::Symbol(Symbol::Quote),
            SqlToken::Symbol(Symbol::RParen),
        ]
    }

    fn left_like_tokens<D: SqlDialect>() -> Vec<SqlToken<'static>> {
        vec![
            SqlToken::Keyword(Keyword::Like),
            SqlToken::WhiteSpace,
            SqlToken::Keyword(Keyword::Concat),
            SqlToken::Symbol(Symbol::LParen),
            SqlToken::Symbol(Symbol::Quote),
            SqlToken::Symbol(Symbol::PercentSign),
            SqlToken::Symbol(Symbol::Quote),
            SqlToken::Symbol(Symbol::Comma),
            SqlToken::WhiteSpace,
            SqlToken::Keyword(Keyword::Cast),
            SqlToken::Symbol(Symbol::LParen),
            SqlToken::Placeholder(PlaceholderKind::Value(1)),
            SqlToken::WhiteSpace,
            SqlToken::Keyword(Keyword::As),
            SqlToken::WhiteSpace,
            SqlToken::Ident(Cow::from(<PlaceholderDatatype as Into<&str>>::into(D::PLACEHOLDER_DATA_TYPE.into()))),
            SqlToken::Symbol(Symbol::RParen),
            SqlToken::Symbol(Symbol::RParen),
        ]
    }

    fn right_like_tokens<D: SqlDialect>() -> Vec<SqlToken<'static>> {
        vec![
            SqlToken::Keyword(Keyword::Like),
            SqlToken::WhiteSpace,
            SqlToken::Keyword(Keyword::Concat),
            SqlToken::Symbol(Symbol::LParen),
            SqlToken::Keyword(Keyword::Cast),
            SqlToken::Symbol(Symbol::LParen),
            SqlToken::Placeholder(PlaceholderKind::Value(1)),
            SqlToken::WhiteSpace,
            SqlToken::Keyword(Keyword::As),
            SqlToken::WhiteSpace,
            SqlToken::Ident(Cow::from(<PlaceholderDatatype as Into<&str>>::into(D::PLACEHOLDER_DATA_TYPE.into()))),
            SqlToken::Symbol(Symbol::RParen),
            SqlToken::Symbol(Symbol::Comma),
            SqlToken::WhiteSpace,
            SqlToken::Symbol(Symbol::Quote),
            SqlToken::Symbol(Symbol::PercentSign),
            SqlToken::Symbol(Symbol::Quote),
            SqlToken::Symbol(Symbol::RParen),
        ]
    }

    #[cfg(feature = "postgres")]
    #[test]
    fn full_like_kind_emits_like_concat_wrapping_placeholder_on_both_sides() {
        assert_eq!(
            tokens::<PgDialect, _>(LikeKind::Full),
            full_like_tokens::<PgDialect>(),
        );
    }

    #[cfg(feature = "postgres")]
    #[test]
    fn left_like_kind_emits_like_concat_with_leading_percent() {
        assert_eq!(
            tokens::<PgDialect, _>(LikeKind::Left),
            left_like_tokens::<PgDialect>(),
        );
    }

    #[cfg(feature = "postgres")]
    #[test]
    fn right_like_kind_emits_like_concat_with_trailing_percent() {
        assert_eq!(
            tokens::<PgDialect, _>(LikeKind::Right),
            right_like_tokens::<PgDialect>(),
        );
    }

    #[cfg(feature = "postgres")]
    #[test]
    fn like_operator_delegates_to_like_kind() {
        assert_eq!(
            tokens::<PgDialect, _>(Operator::Like(LikeKind::Full)),
            full_like_tokens::<PgDialect>(),
        );
    }

    #[cfg(feature = "postgres")]
    #[test]
    fn not_like_operator_emits_not_like_instead_of_not_equals() {
        let mut expected = vec![
            SqlToken::Keyword(Keyword::Not),
            SqlToken::WhiteSpace,
        ];
        expected.extend(full_like_tokens::<PgDialect>());
        assert_eq!(
            tokens::<PgDialect, _>(Operator::NotLike(LikeKind::Full)),
            expected,
        );
    }
}