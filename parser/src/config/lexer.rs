//! Lexer for the config.
//!
//! Use [lex] to lex some input.

use crate::common;
use token::*;
use winnow::{
    ascii::multispace0,
    combinator::{delimited, repeat},
    prelude::*,
    stream::{AsChar, Compare, FindSlice, SliceLen, Stream, StreamIsPartial},
};

// Re-export the common lexer
pub use common::lexer::*;

/// A token of the config format
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token<T> {
    /// Opening-symbol for a block
    BlockOpen,
    /// Closing-symbol for a block
    BlockClose,
    /// An identifier
    Identifier(T),
    /// A property-separator
    Separator,
    /// A value; see [Value]
    Value(Value<T>),
    /// A comment, either single- or multiline
    Comment(T),
    /// Opening-symbol for a tuple
    TupleOpen,
    /// Closing-symbol for a tuple
    TupleClose,
    /// A separator between elements (e.g., in tuples)
    ElementSeparator,
}

impl<T> From<GenericToken<T>> for Token<T> {
    fn from(value: GenericToken<T>) -> Self {
        match value {
            GenericToken::Identifier(i) => Self::Identifier(i),
            GenericToken::Value(v) => Self::Value(v),
            GenericToken::Comment(c) => Self::Comment(c),
            GenericToken::TupleOpen => Self::TupleOpen,
            GenericToken::TupleClose => Self::TupleClose,
            GenericToken::ElementSeparator => Self::ElementSeparator,
        }
    }
}

impl<T> From<Token<T>> for Option<GenericToken<T>> {
    fn from(value: Token<T>) -> Self {
        match value {
            Token::Identifier(i) => Some(GenericToken::Identifier(i)),
            Token::Value(v) => Some(GenericToken::Value(v)),
            Token::Comment(c) => Some(GenericToken::Comment(c)),
            Token::TupleOpen => Some(GenericToken::TupleOpen),
            Token::TupleClose => Some(GenericToken::TupleClose),
            Token::ElementSeparator => Some(GenericToken::ElementSeparator),
            _ => None,
        }
    }
}

/// Lexes a [str] into a [Vec] of [Token]s,
/// or returns an [Err] if lexing failed.
pub fn lex<
    I: Stream
        + StreamIsPartial
        + Compare<&'static str>
        + FindSlice<&'static str>
        + FindSlice<(char, char)>
        + Copy,
>(
    input: I,
) -> Result<
    Vec<Token<<I as Stream>::Slice>>,
    winnow::error::ParseError<I, winnow::error::ContextError>,
>
where
    <I as Stream>::Token: AsChar + Clone,
    I::Slice: SliceLen,
{
    repeat(0.., delimited(multispace0, token, multispace0)).parse(input)
}

/// Lexers to lex individual [Token]s.
///
/// All lexers assume their token starts immediately (i.e., no preceding whitespace)
/// and will not consume any subsequent whitespace,
/// except when that whitespace is used as a delimiter,
/// in which case it will be noted.
pub mod token {
    use super::*;
    use winnow::{
        combinator::alt,
        stream::{AsChar, Compare, FindSlice, SliceLen},
    };

    pub use common::lexer::token::*;

    /// Tries to parse a [Token::BlockOpen].
    pub fn block_open<I: Stream + StreamIsPartial + Compare<&'static str>>(
        input: &mut I,
    ) -> PResult<Token<<I as Stream>::Slice>> {
        "{".map(|_| Token::BlockOpen).parse_next(input)
    }

    /// Tries to parse a [Token::BlockClose].
    pub fn block_close<I: Stream + StreamIsPartial + Compare<&'static str>>(
        input: &mut I,
    ) -> PResult<Token<<I as Stream>::Slice>> {
        "}".map(|_| Token::BlockClose).parse_next(input)
    }

    /// Tries to parse a [Token::Separator].
    pub fn separator<I: Stream + StreamIsPartial + Compare<&'static str>>(
        input: &mut I,
    ) -> PResult<Token<<I as Stream>::Slice>> {
        ":".map(|_| Token::Separator).parse_next(input)
    }

    /// Tries to parse any [Token].
    pub fn token<
        I: Stream
            + StreamIsPartial
            + Compare<&'static str>
            + FindSlice<&'static str>
            + FindSlice<(char, char)>
            + Copy,
    >(
        input: &mut I,
    ) -> PResult<Token<<I as Stream>::Slice>>
    where
        <I as Stream>::Token: AsChar + Clone,
        I::Slice: SliceLen,
    {
        alt((
            block_open,
            block_close,
            separator,
            tuple_open,
            tuple_close,
            element_separator,
            comment,
            value,
            identifier,
        ))
        .parse_next(input)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn simple_example() {
        let input = r#"
        property1: "some string"
        property2: 1.2
        ^regex$ /* comment */ : identifier // other comment
        block {
            named_block "name" {
                prop: #c01032
                42: true
            }
        // }
        }
        "#;

        let expected = vec![
            Token::Identifier("property1"),
            Token::Separator,
            Token::Value(Value::String("some string")),
            Token::Identifier("property2"),
            Token::Separator,
            Token::Value(Value::Number("1.2")),
            Token::Value(Value::Regex("regex")),
            Token::Comment(" comment "),
            Token::Separator,
            Token::Identifier("identifier"),
            Token::Comment(" other comment"),
            Token::Identifier("block"),
            Token::BlockOpen,
            Token::Identifier("named_block"),
            Token::Value(Value::String("name")),
            Token::BlockOpen,
            Token::Identifier("prop"),
            Token::Separator,
            Token::Value(Value::Color("c01032")),
            Token::Value(Value::Number("42")),
            Token::Separator,
            Token::Value(Value::Boolean("true")),
            Token::BlockClose,
            Token::Comment(" }"),
            Token::BlockClose,
        ];

        let lexed = lex(input).expect("Failed to lex");

        assert_eq!(lexed, expected);
    }
}
