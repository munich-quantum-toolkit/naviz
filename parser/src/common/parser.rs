//! Common parser items
//!
//! - Primitive [Value]
//! - [Conversion from lexed Value to parsed Value][try_into_value::TryIntoValue]
//! - Helper functions to parse values

use super::{color::Color, lexer::GenericToken, percentage::Percentage};
use fraction::Fraction;
use regex::Regex;
use std::fmt::Debug;
use token::{element_separator, ignore_comments, tuple_close, tuple_open, value_or_identifier};
use try_into_value::TryIntoValue;
use winnow::{
    combinator::{alt, separated, terminated},
    error::ParserError,
    stream::{Stream, StreamIsPartial},
    PResult, Parser,
};

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

/// Create a parser to a [Value::Tuple] from the passed token-parsers.
pub fn tuple_custom<I: Stream + StreamIsPartial, E: ParserError<I>, TO, ES, TC, IG>(
    tuple_open: impl Parser<I, TO, E>,
    element_separator: impl Parser<I, ES, E>,
    tuple_close: impl Parser<I, TC, E>,
    value: impl Parser<I, Value, E>,
    ignore: impl Parser<I, IG, E> + Copy,
) -> impl Parser<I, Value, E> {
    (
        terminated(tuple_open, ignore),
        terminated(separated(0.., value, element_separator), ignore),
        tuple_close,
    )
        .map(|(_, t, _)| Value::Tuple(t))
}

/// Try to parse a [Value::Tuple] from a stream of tokens which are a superset of [GenericToken].
pub fn tuple<Tok: Into<Option<GenericToken<S>>> + Clone + Debug, S: TryIntoValue>(
    input: &mut &[Tok],
) -> PResult<Value> {
    tuple_custom(
        tuple_open,
        element_separator,
        tuple_close,
        value_or_identifier_or_tuple,
        ignore_comments,
    )
    .parse_next(input)
}

/// Try to parse a single [GenericToken::Identifier], [GenericToken::Value], or [Value::Tuple]
/// using [value_or_identifier] and [tuple()] respectively.
pub fn value_or_identifier_or_tuple<
    Tok: Into<Option<GenericToken<S>>> + Clone + Debug,
    S: TryIntoValue,
>(
    input: &mut &[Tok],
) -> PResult<Value> {
    alt((value_or_identifier, tuple)).parse_next(input)
}

pub mod token {
    use super::*;
    use crate::input::lexer::GenericToken;
    use std::fmt::Debug;
    use try_into_value::TryIntoValue;
    use winnow::{
        combinator::{alt, repeat},
        token::one_of,
        PResult,
    };

    /// Try to parse a single [GenericToken::Identifier],
    /// mapping the value to a [String] using [TryIntoValue::identifier].
    pub fn identifier<Tok: Into<Option<GenericToken<S>>> + Clone + Debug, S: TryIntoValue>(
        input: &mut &[Tok],
    ) -> PResult<String> {
        one_of(|t: Tok| matches!(t.into(), Some(GenericToken::Identifier(_))))
            .output_into()
            .map(|t| match t {
                Some(GenericToken::Identifier(i)) => i,
                _ => unreachable!(),
            })
            .try_map(TryIntoValue::identifier)
            .parse_next(input)
    }

    /// Try to parse a single [GenericToken::Value],
    /// mapping the value to a [Value] using [TryIntoValue].
    pub fn value<Tok: Into<Option<GenericToken<S>>> + Clone + Debug, S: TryIntoValue>(
        input: &mut &[Tok],
    ) -> PResult<Value> {
        one_of(|t: Tok| matches!(t.into(), Some(GenericToken::Value(_))))
            .output_into()
            .try_map(|t| match t {
                Some(GenericToken::Value(v)) => v.try_into(),
                _ => unreachable!(), // Parser only matches value
            })
            .parse_next(input)
    }

    /// Try to parse a single [GenericToken::Identifier] or [GenericToken::Value]
    /// using [identifier] and [value] respectively,
    /// mapping it to a [Value].
    pub fn value_or_identifier<
        Tok: Into<Option<GenericToken<S>>> + Clone + Debug,
        S: TryIntoValue,
    >(
        input: &mut &[Tok],
    ) -> PResult<Value> {
        alt((value, identifier.map(Value::Identifier))).parse_next(input)
    }

    /// Try to parse a single [GenericToken::Comment].
    pub fn comment<Tok: Into<Option<GenericToken<S>>> + Clone + Debug, S: TryIntoValue>(
        input: &mut &[Tok],
    ) -> PResult<S> {
        one_of(|t: Tok| matches!(t.into(), Some(GenericToken::Comment(_))))
            .output_into()
            .map(|t| match t {
                Some(GenericToken::Comment(c)) => c,
                _ => unreachable!(), // Parser only matches comment
            })
            .parse_next(input)
    }

    /// Ignore all comments until the next non-comment token.
    pub fn ignore_comments<Tok: Into<Option<GenericToken<S>>> + Clone + Debug, S: TryIntoValue>(
        input: &mut &[Tok],
    ) -> PResult<()> {
        repeat::<_, _, (), _, _>(0.., comment)
            .void()
            .parse_next(input)
    }

    /// Try to parse a single [GenericToken::TupleOpen].
    pub fn tuple_open<Tok: Into<Option<GenericToken<S>>> + Clone + Debug, S: TryIntoValue>(
        input: &mut &[Tok],
    ) -> PResult<()> {
        one_of(|t: Tok| matches!(t.into(), Some(GenericToken::TupleOpen)))
            .void()
            .parse_next(input)
    }

    /// Try to parse a single [GenericToken::TupleClose].
    pub fn tuple_close<Tok: Into<Option<GenericToken<S>>> + Clone + Debug, S: TryIntoValue>(
        input: &mut &[Tok],
    ) -> PResult<()> {
        one_of(|t: Tok| matches!(t.into(), Some(GenericToken::TupleClose)))
            .void()
            .parse_next(input)
    }

    /// Try to parse a single [GenericToken::ElementSeparator].
    pub fn element_separator<
        Tok: Into<Option<GenericToken<S>>> + Clone + Debug,
        S: TryIntoValue,
    >(
        input: &mut &[Tok],
    ) -> PResult<()> {
        one_of(|t: Tok| matches!(t.into(), Some(GenericToken::ElementSeparator)))
            .void()
            .parse_next(input)
    }
}
