use super::percentage::Percentage;
use super::{color::Color, lexer::Token};
use fraction::Fraction;
use regex::Regex;
use std::fmt::Debug;
use token::{
    block_close, block_open, element_separator, identifier, ignore_comments, separator,
    tuple_close, tuple_open, value_or_identifier,
};
use try_into_value::TryIntoValue;
use winnow::combinator::{alt, preceded, repeat, separated, terminated};
use winnow::prelude::*;

pub mod try_into_value;

/// A parsed value.
#[derive(Debug, Clone)]
pub enum Value {
    /// A string
    String(String),
    /// A regex
    Regex(Regex),
    /// A number
    Number(Fraction),
    /// A percentage
    Percentage(Percentage),
    /// A boolean
    Boolean(bool),
    /// A color
    Color(Color),
    /// An identifier
    Identifier(String),
    /// A tuple
    Tuple(Vec<Value>),
}

/// For tests, allow comparing [Value]s.
/// In particular, check if [Value::Regex]s were compiled from the same source string
/// (and use [PartialEq] for all other variants).
#[cfg(test)]
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Regex(a), Value::Regex(b)) => a.as_str() == b.as_str(),
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Color(a), Value::Color(b)) => a == b,
            (Value::Identifier(a), Value::Identifier(b)) => a == b,
            _ => false,
        }
    }
}

/// A [ConfigItem] represents a single item of the config.
#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub enum ConfigItem {
    // `key`, `value`
    Property(Value, Value),
    // `identifier`, `content`
    Block(String, Config),
    // `identifier`, `name`, `content`
    NamedBlock(String, Value, Config),
}

/// A [Config] is all [ConfigItem]s of a parsed config.
pub type Config = Vec<ConfigItem>;

/// Parse a full stream of [Token]s into a [Config].
pub fn parse<S: TryIntoValue + Clone + Debug + PartialEq>(
    input: &[Token<S>],
) -> Result<Config, winnow::error::ParseError<&[Token<S>], winnow::error::ContextError>> {
    config.parse(input)
}

/// Try to parse a [Config] from a stream of [Token]s.
pub fn config<S: TryIntoValue + Clone + Debug + PartialEq>(
    input: &mut &[Token<S>],
) -> PResult<Config> {
    preceded(
        ignore_comments,
        repeat(0.., terminated(config_item, ignore_comments)),
    )
    .parse_next(input)
}

/// Try to parse a single [ConfigItem] from a stream of [Token]s.
pub fn config_item<S: TryIntoValue + Clone + Debug + PartialEq>(
    input: &mut &[Token<S>],
) -> PResult<ConfigItem> {
    alt((property, block, named_block)).parse_next(input)
}

/// Try to parse a [ConfigItem::Property] from a stream of [Token]s.
pub fn property<S: TryIntoValue + Clone + Debug + PartialEq>(
    input: &mut &[Token<S>],
) -> PResult<ConfigItem> {
    (
        terminated(value_or_identifier_or_tuple, ignore_comments),
        terminated(separator, ignore_comments),
        value_or_identifier_or_tuple,
    )
        .map(|(k, _, v)| ConfigItem::Property(k, v))
        .parse_next(input)
}

/// Try to parse a [ConfigItem::Block] from a stream of [Token]s.
pub fn block<S: TryIntoValue + Clone + Debug + PartialEq>(
    input: &mut &[Token<S>],
) -> PResult<ConfigItem> {
    (
        terminated(identifier, ignore_comments),
        terminated(block_open, ignore_comments),
        terminated(config, ignore_comments),
        block_close,
    )
        .map(|(i, _, c, _)| ConfigItem::Block(i, c))
        .parse_next(input)
}

/// Try to parse a [ConfigItem::NamedBlock] from a stream of [Token]s.
pub fn named_block<S: TryIntoValue + Clone + Debug + PartialEq>(
    input: &mut &[Token<S>],
) -> PResult<ConfigItem> {
    (
        terminated(identifier, ignore_comments),
        terminated(value_or_identifier_or_tuple, ignore_comments),
        terminated(block_open, ignore_comments),
        terminated(config, ignore_comments),
        block_close,
    )
        .map(|(i, n, _, c, _)| ConfigItem::NamedBlock(i, n, c))
        .parse_next(input)
}

/// Try to parse a [Value::Tuple] from a stream of [Token]s.
pub fn tuple<S: TryIntoValue + Clone + Debug + PartialEq>(
    input: &mut &[Token<S>],
) -> PResult<Value> {
    (
        terminated(tuple_open, ignore_comments),
        terminated(
            separated(0.., value_or_identifier_or_tuple, element_separator),
            ignore_comments,
        ),
        tuple_close,
    )
        .map(|(_, t, _)| Value::Tuple(t))
        .parse_next(input)
}

/// Try to parse a single [Token::Identifier], [Token::Value], or [Value::Tuple]
/// using [value_or_identifier] and [tuple] respectively.
pub fn value_or_identifier_or_tuple<S: Clone + Debug + PartialEq + TryIntoValue>(
    input: &mut &[Token<S>],
) -> PResult<Value> {
    alt((value_or_identifier, tuple)).parse_next(input)
}

/// Parse single [Token]s into their abstract config counterparts.
pub mod token {
    use super::*;
    use winnow::token::one_of;

    /// Try to parse a single [Token::BlockOpen].
    pub fn block_open<S: Clone + Debug + PartialEq>(input: &mut &[Token<S>]) -> PResult<()> {
        one_of([Token::BlockOpen]).void().parse_next(input)
    }

    /// Try to parse a single [Token::BlockClose].
    pub fn block_close<S: Clone + Debug + PartialEq>(input: &mut &[Token<S>]) -> PResult<()> {
        one_of([Token::BlockClose]).void().parse_next(input)
    }

    /// Try to parse a single [Token::Identifier],
    /// mapping the value to a [String] using [TryIntoValue::identifier].
    pub fn identifier<S: Clone + Debug + PartialEq + TryIntoValue>(
        input: &mut &[Token<S>],
    ) -> PResult<String> {
        one_of(|t| matches!(t, Token::Identifier(_)))
            .try_map(|t: Token<S>| match t {
                Token::Identifier(i) => i.identifier(),
                _ => unreachable!(), // Parser only matches identifier
            })
            .parse_next(input)
    }

    /// Try to parse a single [Token::Separator].
    pub fn separator<S: Clone + Debug + PartialEq>(input: &mut &[Token<S>]) -> PResult<()> {
        one_of([Token::Separator]).void().parse_next(input)
    }

    /// Try to parse a single [Token::Value],
    /// mapping the value to a [Value] using [TryIntoValue].
    pub fn value<S: Clone + Debug + PartialEq + TryIntoValue>(
        input: &mut &[Token<S>],
    ) -> PResult<Value> {
        one_of(|t| matches!(t, Token::Value(_)))
            .try_map(|t| match t {
                Token::Value(v) => v.try_into(),
                _ => unreachable!(), // Parser only matches value
            })
            .parse_next(input)
    }

    /// Try to parse a single [Token::Identifier] or [Token::Value]
    /// using [identifier] and [value] respectively,
    /// mapping it to a [Value].
    pub fn value_or_identifier<S: Clone + Debug + PartialEq + TryIntoValue>(
        input: &mut &[Token<S>],
    ) -> PResult<Value> {
        alt((value, identifier.map(Value::Identifier))).parse_next(input)
    }

    /// Try to parse a single [Token::Comment].
    pub fn comment<S: Clone + Debug + PartialEq>(input: &mut &[Token<S>]) -> PResult<S> {
        one_of(|t| matches!(t, Token::Comment(_)))
            .map(|t: Token<S>| match t {
                Token::Comment(c) => c,
                _ => unreachable!(), // Parser only matches comment
            })
            .parse_next(input)
    }

    /// Ignore all comments until the next non-comment token.
    pub fn ignore_comments<S: Clone + Debug + PartialEq>(input: &mut &[Token<S>]) -> PResult<()> {
        repeat::<_, _, (), _, _>(0.., comment)
            .void()
            .parse_next(input)
    }

    /// Try to parse a single [Token::TupleOpen].
    pub fn tuple_open<S: Clone + Debug + PartialEq>(input: &mut &[Token<S>]) -> PResult<()> {
        one_of([Token::TupleOpen]).void().parse_next(input)
    }

    /// Try to parse a single [Token::TupleClose].
    pub fn tuple_close<S: Clone + Debug + PartialEq>(input: &mut &[Token<S>]) -> PResult<()> {
        one_of([Token::TupleClose]).void().parse_next(input)
    }

    /// Try to parse a single [Token::ElementSeparator].
    pub fn element_separator<S: Clone + Debug + PartialEq>(input: &mut &[Token<S>]) -> PResult<()> {
        one_of([Token::ElementSeparator]).void().parse_next(input)
    }
}

// Implement `ContainsToken` for `Token` and `Token`-slices.

impl<T: PartialEq> winnow::stream::ContainsToken<Token<T>> for Token<T> {
    #[inline(always)]
    fn contains_token(&self, token: Token<T>) -> bool {
        *self == token
    }
}

impl<T: PartialEq> winnow::stream::ContainsToken<Token<T>> for &'_ [Token<T>] {
    #[inline]
    fn contains_token(&self, token: Token<T>) -> bool {
        self.iter().any(|t| *t == token)
    }
}

impl<T: PartialEq, const LEN: usize> winnow::stream::ContainsToken<Token<T>>
    for &'_ [Token<T>; LEN]
{
    #[inline]
    fn contains_token(&self, token: Token<T>) -> bool {
        self.iter().any(|t| *t == token)
    }
}

impl<T: PartialEq, const LEN: usize> winnow::stream::ContainsToken<Token<T>> for [Token<T>; LEN] {
    #[inline]
    fn contains_token(&self, token: Token<T>) -> bool {
        self.iter().any(|t| *t == token)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::lexer;

    #[test]
    pub fn simple_example() {
        let input = [
            Token::Identifier("property1"),
            Token::Separator,
            Token::Value(lexer::Value::String("some string")),
            Token::Identifier("property2"),
            Token::Separator,
            Token::Value(lexer::Value::Number("1.2")),
            Token::Value(lexer::Value::Regex("regex")),
            Token::Comment(" comment "),
            Token::Separator,
            Token::Identifier("identifier"),
            Token::Comment(" other comment"),
            Token::Identifier("block"),
            Token::BlockOpen,
            Token::Identifier("named_block"),
            Token::Value(lexer::Value::String("name")),
            Token::BlockOpen,
            Token::Identifier("prop"),
            Token::Separator,
            Token::Value(lexer::Value::Color("c01032")),
            Token::Value(lexer::Value::Number("42")),
            Token::Separator,
            Token::Value(lexer::Value::Boolean("true")),
            Token::BlockClose,
            Token::Comment(" }"),
            Token::BlockClose,
        ];

        let expected = vec![
            ConfigItem::Property(
                Value::Identifier("property1".to_string()),
                Value::String("some string".to_string()),
            ),
            ConfigItem::Property(
                Value::Identifier("property2".to_string()),
                Value::Number(Fraction::new(12u64, 10u64)),
            ),
            ConfigItem::Property(
                Value::Regex(Regex::new("regex").unwrap()),
                Value::Identifier("identifier".to_string()),
            ),
            ConfigItem::Block(
                "block".to_string(),
                vec![ConfigItem::NamedBlock(
                    "named_block".to_string(),
                    Value::String("name".to_string()),
                    vec![
                        ConfigItem::Property(
                            Value::Identifier("prop".to_string()),
                            Value::Color(Color {
                                r: 192,
                                g: 16,
                                b: 50,
                                a: 255,
                            }),
                        ),
                        ConfigItem::Property(
                            Value::Number(Fraction::new(42u64, 1u64)),
                            Value::Boolean(true),
                        ),
                    ],
                )],
            ),
        ];

        let actual = parse(&input).expect("Failed to parse");

        assert_eq!(actual, expected);
    }
}
