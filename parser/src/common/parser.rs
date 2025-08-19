//! Common parser items
//!
//! - Primitive [Value]
//! - [Conversion from lexed Value to parsed Value][try_into_value::TryIntoValue]
//! - Helper functions to parse values

use super::{color::Color, lexer::GenericToken, percentage::Percentage};
use fraction::Fraction;
use regex::Regex;
use std::fmt::Debug;
use token::{
    element_separator, ignore_comments, set_close, set_open, tuple_close, tuple_open,
    value_or_identifier,
};
use try_into_value::TryIntoValue;
use winnow::{
    combinator::{alt, separated, terminated},
    error::ParserError,
    stream::{Stream, StreamIsPartial},
    ModalResult, Parser,
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
    /// A set of values
    Set(Vec<Value>),
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
            (Value::Set(a), Value::Set(b)) => a == b,
            (Value::Tuple(a), Value::Tuple(b)) => a == b,
            _ => false,
        }
    }
}

impl Value {
    /// Recursively flattens all [Set][Value::Set]s contained in this [Value].
    /// Will only flatten and recurse into [Set][Value::Set]s
    /// (i.e., will not flatten or recurse into e.g., [Tuple][Value::Tuple]s).
    pub fn flatten_sets(self) -> Box<dyn Iterator<Item = Self>> {
        match self {
            Self::Set(v) => Box::new(v.into_iter().flat_map(Value::flatten_sets)),
            value => Box::new(std::iter::once(value)),
        }
    }
}

/// Create a parser to a list-like [Value] (e.g., tuples or sets) using the passed parameters.
pub fn list_like<I: Stream + StreamIsPartial, E: ParserError<I>, TO, ES, TC, IG>(
    open: impl Parser<I, TO, E>,
    element_separator: impl Parser<I, ES, E>,
    close: impl Parser<I, TC, E>,
    value: impl Parser<I, Value, E>,
    ignore: impl Parser<I, IG, E> + Copy,
    output: impl Fn(Vec<Value>) -> Value + 'static,
) -> impl Parser<I, Value, E> {
    (
        terminated(open, ignore),
        terminated(separated(0.., value, element_separator), ignore),
        close,
    )
        .map(move |(_, values, _)| output(values))
}

/// Try to parse a [Value::Tuple] from a stream of tokens which are a superset of [GenericToken].
pub fn tuple<Tok: Into<Option<GenericToken<S>>> + Clone + Debug, S: TryIntoValue>(
    input: &mut &[Tok],
) -> ModalResult<Value> {
    list_like(
        tuple_open,
        element_separator,
        tuple_close,
        any_value,
        ignore_comments,
        Value::Tuple,
    )
    .parse_next(input)
}

/// Try to parse a [Value::Set] from a stream of tokens which are a superset of [GenericToken].
pub fn set<Tok: Into<Option<GenericToken<S>>> + Clone + Debug, S: TryIntoValue>(
    input: &mut &[Tok],
) -> ModalResult<Value> {
    list_like(
        set_open,
        element_separator,
        set_close,
        any_value,
        ignore_comments,
        Value::Set,
    )
    .parse_next(input)
}

/// Try to parse any [Value] from the stream.
/// Does not only parse single-token values like [value_or_identifier],
/// but also parses composite values such as sets or tuples.
pub fn any_value<Tok: Into<Option<GenericToken<S>>> + Clone + Debug, S: TryIntoValue>(
    input: &mut &[Tok],
) -> ModalResult<Value> {
    alt((value_or_identifier, tuple, set)).parse_next(input)
}

pub mod token {
    use super::*;
    use crate::input::lexer::GenericToken;
    use std::fmt::Debug;
    use try_into_value::TryIntoValue;
    use winnow::{
        combinator::{alt, repeat},
        token::one_of,
        ModalResult,
    };

    /// Try to parse a single [GenericToken::Identifier],
    /// mapping the value to a [String] using [TryIntoValue::identifier].
    pub fn identifier<Tok: Into<Option<GenericToken<S>>> + Clone + Debug, S: TryIntoValue>(
        input: &mut &[Tok],
    ) -> ModalResult<String> {
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
    ) -> ModalResult<Value> {
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
    ) -> ModalResult<Value> {
        alt((value, identifier.map(Value::Identifier))).parse_next(input)
    }

    /// Try to parse a single [GenericToken::Comment].
    pub fn comment<Tok: Into<Option<GenericToken<S>>> + Clone + Debug, S: TryIntoValue>(
        input: &mut &[Tok],
    ) -> ModalResult<S> {
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
    ) -> ModalResult<()> {
        repeat::<_, _, (), _, _>(0.., comment)
            .void()
            .parse_next(input)
    }

    /// Try to parse a single [GenericToken::TupleOpen].
    pub fn tuple_open<Tok: Into<Option<GenericToken<S>>> + Clone + Debug, S: TryIntoValue>(
        input: &mut &[Tok],
    ) -> ModalResult<()> {
        one_of(|t: Tok| matches!(t.into(), Some(GenericToken::TupleOpen)))
            .void()
            .parse_next(input)
    }

    /// Try to parse a single [GenericToken::TupleClose].
    pub fn tuple_close<Tok: Into<Option<GenericToken<S>>> + Clone + Debug, S: TryIntoValue>(
        input: &mut &[Tok],
    ) -> ModalResult<()> {
        one_of(|t: Tok| matches!(t.into(), Some(GenericToken::TupleClose)))
            .void()
            .parse_next(input)
    }

    /// Try to parse a single [GenericToken::SetOpen].
    pub fn set_open<Tok: Into<Option<GenericToken<S>>> + Clone + Debug, S: TryIntoValue>(
        input: &mut &[Tok],
    ) -> ModalResult<()> {
        one_of(|t: Tok| matches!(t.into(), Some(GenericToken::SetOpen)))
            .void()
            .parse_next(input)
    }

    /// Try to parse a single [GenericToken::SetClose].
    pub fn set_close<Tok: Into<Option<GenericToken<S>>> + Clone + Debug, S: TryIntoValue>(
        input: &mut &[Tok],
    ) -> ModalResult<()> {
        one_of(|t: Tok| matches!(t.into(), Some(GenericToken::SetClose)))
            .void()
            .parse_next(input)
    }

    /// Try to parse a single [GenericToken::ElementSeparator].
    pub fn element_separator<
        Tok: Into<Option<GenericToken<S>>> + Clone + Debug,
        S: TryIntoValue,
    >(
        input: &mut &[Tok],
    ) -> ModalResult<()> {
        one_of(|t: Tok| matches!(t.into(), Some(GenericToken::ElementSeparator)))
            .void()
            .parse_next(input)
    }
}

#[cfg(test)]
mod test {
    use super::Value;

    #[test]
    fn set_flatten() {
        let input = Value::Set(vec![
            Value::Identifier("i1".to_string()),
            Value::Set(vec![Value::Identifier("i2".to_string())]),
            Value::Set(vec![
                Value::Identifier("i3".to_string()),
                Value::Set(vec![
                    Value::Identifier("i4".to_string()),
                    Value::Set(vec![Value::Identifier("i5".to_string())]),
                ]),
            ]),
        ]);

        let expected = vec![
            Value::Identifier("i1".to_string()),
            Value::Identifier("i2".to_string()),
            Value::Identifier("i3".to_string()),
            Value::Identifier("i4".to_string()),
            Value::Identifier("i5".to_string()),
        ];

        assert_eq!(
            input.flatten_sets().collect::<Vec<_>>(),
            expected,
            "Set was incorrectly flattened"
        );
    }
}
