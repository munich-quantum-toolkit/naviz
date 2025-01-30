//! Lexer for the `.naviz` format.
//! Use [lex] to lex into a stream of [Token]s.

use crate::common;
use token::token;
use winnow::{
    ascii::{multispace0, space0},
    combinator::{preceded, repeat, terminated},
    stream::{AsChar, Compare, FindSlice, SliceLen, Stream, StreamIsPartial},
    Parser,
};

// Re-export the common lexer
pub use common::lexer::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TimeSpec {
    Absolute,
    Relative { from_start: bool, positive: bool },
}

/// A token of the config format
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token<T> {
    /// An identifier
    Identifier(T),
    /// A value; see [Value]
    Value(Value<T>),
    /// A comment, either single- or multiline
    Comment(T),
    /// Opening-symbol for a tuple
    TupleOpen,
    /// Closing-symbol for a tuple
    TupleClose,
    /// Opening-symbol for a group
    GroupOpen {
        /// The timing of the subcommands can be variable
        variable: bool,
    },
    /// Closing-symbol for a group
    GroupClose,
    /// A separator between elements (e.g., in tuples)
    ElementSeparator,
    /// The symbol to denote the starting-time
    TimeSymbol(TimeSpec),
    /// A directive (`#<directive`)
    Directive(T),
    /// The separator between instructions
    Separator,
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
    preceded(multispace0, repeat(0.., terminated(token, space0)))
        .parse(input)
        .map(|mut tokens: Vec<_>| {
            // Ensure separator at end of token-stream
            match tokens.last() {
                Some(Token::Separator) => { /* Already exists */ }
                _ => tokens.push(Token::Separator),
            }
            tokens
        })
}

pub mod token {
    use super::*;
    use winnow::{
        ascii::{line_ending, multispace0},
        combinator::{alt, opt, terminated},
        stream::{AsChar, Compare, FindSlice, SliceLen, Stream, StreamIsPartial},
        token::take_till,
        PResult, Parser,
    };

    pub use common::lexer::token::*;

    /// Tries to parse a [Token::GroupOpen].
    pub fn group_open<Tok, I: Stream + StreamIsPartial + Compare<&'static str>>(
        input: &mut I,
    ) -> PResult<Tok>
    where
        Token<I::Slice>: Into<Tok>,
    {
        (opt("~"), "[")
            .map(|(variable, _)| Token::GroupOpen {
                variable: variable.is_some(),
            })
            .output_into()
            .parse_next(input)
    }

    /// Tries to parse a [Token::GroupClose].
    pub fn group_close<Tok, I: Stream + StreamIsPartial + Compare<&'static str>>(
        input: &mut I,
    ) -> PResult<Tok>
    where
        Token<I::Slice>: Into<Tok>,
    {
        "]".map(|_| Token::GroupClose)
            .output_into()
            .parse_next(input)
    }

    /// Tries to parse a single [Token::TimeSymbol].
    pub fn time_symbol<I: Stream + StreamIsPartial + Compare<&'static str>>(
        input: &mut I,
    ) -> PResult<Token<<I as Stream>::Slice>> {
        (
            "@",
            opt("=".void()).map(|o| o.is_some()),
            opt(alt(["+".value(true), "-".value(false)])),
        )
            .map(|(_, from_start, sign)| {
                Token::TimeSymbol(match sign {
                    Some(positive) => TimeSpec::Relative {
                        from_start,
                        positive,
                    },
                    None => {
                        if from_start {
                            // Handle `@=<time>`
                            TimeSpec::Relative {
                                from_start: true,
                                positive: true,
                            }
                        } else {
                            TimeSpec::Absolute
                        }
                    }
                })
            })
            .parse_next(input)
    }

    /// Tries to parse a single [Token::Directive].
    pub fn directive<I: Stream + StreamIsPartial + Compare<&'static str>>(
        input: &mut I,
    ) -> PResult<Token<<I as Stream>::Slice>>
    where
        I::Token: AsChar + Clone,
    {
        (
            "#",
            take_till(0.., |x: I::Token| x.clone().is_newline() || x.is_space()),
        )
            .map(|(_, dir)| Token::Directive(dir))
            .parse_next(input)
    }

    /// Tries to parse a single [Token::Separator].
    pub fn separator<I: Stream + StreamIsPartial + Compare<&'static str>>(
        input: &mut I,
    ) -> PResult<Token<<I as Stream>::Slice>>
    where
        I::Token: AsChar + Clone,
    {
        terminated(line_ending, multispace0)
            .map(|_| Token::Separator)
            .parse_next(input)
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
            value,
            identifier,
            comment,
            tuple_open,
            tuple_close,
            group_open,
            group_close,
            element_separator,
            time_symbol,
            directive,
            separator,
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
        #directive value
        #other_directive "string"

        instruction argument "argument" ^argument$
        @0 timed_instruction arg
        @-1 negative_timed_instruction arg
        @=2 positive_start_timed_instruction arg
        @+ [
            group_instruction_a 1
            group_instruction_b 2
        ]
        group_instruction ~[
            1
            2
        ]"#;

        let expected = vec![
            Token::Directive("directive"),
            Token::Identifier("value"),
            Token::Separator,
            Token::Directive("other_directive"),
            Token::Value(Value::String("string")),
            Token::Separator,
            Token::Identifier("instruction"),
            Token::Identifier("argument"),
            Token::Value(Value::String("argument")),
            Token::Value(Value::Regex("^argument$")),
            Token::Separator,
            Token::TimeSymbol(TimeSpec::Absolute),
            Token::Value(Value::Number("0")),
            Token::Identifier("timed_instruction"),
            Token::Identifier("arg"),
            Token::Separator,
            Token::TimeSymbol(TimeSpec::Relative {
                from_start: false,
                positive: false,
            }),
            Token::Value(Value::Number("1")),
            Token::Identifier("negative_timed_instruction"),
            Token::Identifier("arg"),
            Token::Separator,
            Token::TimeSymbol(TimeSpec::Relative {
                from_start: true,
                positive: true,
            }),
            Token::Value(Value::Number("2")),
            Token::Identifier("positive_start_timed_instruction"),
            Token::Identifier("arg"),
            Token::Separator,
            Token::TimeSymbol(TimeSpec::Relative {
                from_start: false,
                positive: true,
            }),
            Token::GroupOpen { variable: false },
            Token::Separator,
            Token::Identifier("group_instruction_a"),
            Token::Value(Value::Number("1")),
            Token::Separator,
            Token::Identifier("group_instruction_b"),
            Token::Value(Value::Number("2")),
            Token::Separator,
            Token::GroupClose,
            Token::Separator,
            Token::Identifier("group_instruction"),
            Token::GroupOpen { variable: true },
            Token::Separator,
            Token::Value(Value::Number("1")),
            Token::Separator,
            Token::Value(Value::Number("2")),
            Token::Separator,
            Token::GroupClose,
            Token::Separator,
        ];

        let actual = lex(input).expect("Failed to lex");

        assert_eq!(actual, expected);
    }
}
