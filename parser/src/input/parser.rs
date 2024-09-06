//! Parser for the `.naviz` format.
//! Takes tokens lexed by the [lexer][super::lexer].

use super::lexer::{TimeSpec, Token};
use crate::common::{self, parser::try_into_value::TryIntoValue};
use fraction::Fraction;
use std::fmt::Debug;
use token::{identifier, ignore_comments, number, separator, time_symbol};
use winnow::{
    combinator::{alt, opt, preceded, repeat, terminated},
    PResult, Parser,
};

// Re-export the common parser
pub use common::parser::*;

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub enum InstructionOrDirective {
    Instruction {
        time: Option<(TimeSpec, Fraction)>,
        name: String,
        args: Vec<Value>,
    },
    Directive {
        name: String,
        args: Vec<Value>,
    },
}

/// Parse a full stream of [Token]s into a [Vec] of [InstructionOrDirective]s.
pub fn parse<S: TryIntoValue + Clone + Debug + PartialEq>(
    input: &[Token<S>],
) -> Result<
    Vec<InstructionOrDirective>,
    winnow::error::ParseError<&[Token<S>], winnow::error::ContextError>,
> {
    instruction_or_directives.parse(input)
}

/// Parse all [Instruction][InstructionOrDirective::Instruction]s
/// and [Directive][InstructionOrDirective::Directive]s
pub fn instruction_or_directives<S: TryIntoValue + Clone + Debug + PartialEq>(
    input: &mut &[Token<S>],
) -> PResult<Vec<InstructionOrDirective>> {
    preceded(
        ignore_comments,
        repeat(
            0..,
            terminated(alt((instruction, directive)), ignore_comments),
        ),
    )
    .parse_next(input)
}

/// Try to parse an [Instruction][InstructionOrDirective::Instruction] from a stream of [Token]s.
pub fn instruction<S: TryIntoValue + Clone + Debug + PartialEq>(
    input: &mut &[Token<S>],
) -> PResult<InstructionOrDirective> {
    (
        terminated(opt(time), ignore_comments),
        terminated(identifier, ignore_comments),
        repeat(
            0..,
            terminated(value_or_identifier_or_tuple, ignore_comments),
        ),
        separator,
    )
        .map(|(time, name, args, _)| InstructionOrDirective::Instruction { time, name, args })
        .parse_next(input)
}

/// Try to parse an [Directive][InstructionOrDirective::Directive] from a stream of [Token]s.
pub fn directive<S: TryIntoValue + Clone + Debug + PartialEq>(
    input: &mut &[Token<S>],
) -> PResult<InstructionOrDirective> {
    (
        terminated(token::directive, ignore_comments),
        repeat(
            0..,
            terminated(value_or_identifier_or_tuple, ignore_comments),
        ),
        separator,
    )
        .map(|(name, args, _)| InstructionOrDirective::Directive { name, args })
        .parse_next(input)
}

/// Try to parse a time ([TimeSpec] and accompanying number) from a stream of [Token]s.
pub fn time<S: TryIntoValue + Clone + Debug>(
    input: &mut &[Token<S>],
) -> PResult<(TimeSpec, Fraction)> {
    (time_symbol, number).parse_next(input)
}

pub mod token {
    use super::*;
    use crate::input::lexer::{self, TimeSpec};
    use winnow::{token::one_of, Parser};

    // Re-export the common token-parsers
    pub use common::parser::token::*;

    /// Try to parse a single [Token::TimeSymbol].
    pub fn time_symbol<S: Clone + Debug>(input: &mut &[Token<S>]) -> PResult<TimeSpec> {
        one_of(|t| matches!(t, Token::TimeSymbol(_)))
            .map(|t| match t {
                Token::TimeSymbol(t) => t,
                _ => unreachable!(),
            })
            .parse_next(input)
    }

    /// Try to parse a single [Token::Separator].
    pub fn separator<S: Clone + Debug + PartialEq>(input: &mut &[Token<S>]) -> PResult<()> {
        one_of([Token::Separator]).void().parse_next(input)
    }

    /// Try to parse a single [Token::Value] where the value is a [Value::Number].
    pub fn number<S: TryIntoValue + Clone + Debug>(input: &mut &[Token<S>]) -> PResult<Fraction> {
        one_of(|t| matches!(t, Token::Value(lexer::Value::Number(_))))
            .map(|t| match t {
                Token::Value(lexer::Value::Number(n)) => n,
                _ => unreachable!(),
            })
            .try_map(TryIntoValue::number)
            .parse_next(input)
    }

    /// Try to parse a single [Token::Directive].
    pub fn directive<S: TryIntoValue + Clone + Debug>(input: &mut &[Token<S>]) -> PResult<String> {
        one_of(|t| matches!(t, Token::Directive(_)))
            .map(|t| match t {
                Token::Directive(d) => d,
                _ => unreachable!(),
            })
            .try_map(TryIntoValue::identifier)
            .parse_next(input)
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
    use crate::common::lexer;
    use fraction::Fraction;
    use regex::Regex;

    #[test]
    pub fn simple_example() {
        let input = vec![
            Token::Directive("directive"),
            Token::Identifier("value"),
            Token::Separator,
            Token::Directive("other_directive"),
            Token::Value(lexer::Value::String("string")),
            Token::Separator,
            Token::Identifier("instruction"),
            Token::Identifier("argument"),
            Token::Value(lexer::Value::String("argument")),
            Token::Value(lexer::Value::Regex("argument")),
            Token::Separator,
            Token::TimeSymbol(TimeSpec::Absolute),
            Token::Value(lexer::Value::Number("0")),
            Token::Identifier("timed_instruction"),
            Token::Identifier("arg"),
            Token::Separator,
            Token::TimeSymbol(TimeSpec::Relative {
                from_start: false,
                positive: false,
            }),
            Token::Value(lexer::Value::Number("1")),
            Token::Identifier("negative_timed_instruction"),
            Token::Identifier("arg"),
            Token::Separator,
            Token::TimeSymbol(TimeSpec::Relative {
                from_start: true,
                positive: true,
            }),
            Token::Value(lexer::Value::Number("2")),
            Token::Identifier("positive_start_timed_instruction"),
            Token::Identifier("arg"),
            Token::Separator,
        ];

        let expected = vec![
            InstructionOrDirective::Directive {
                name: "directive".to_string(),
                args: vec![Value::Identifier("value".to_string())],
            },
            InstructionOrDirective::Directive {
                name: "other_directive".to_string(),
                args: vec![Value::String("string".to_string())],
            },
            InstructionOrDirective::Instruction {
                time: None,
                name: "instruction".to_string(),
                args: vec![
                    Value::Identifier("argument".to_string()),
                    Value::String("argument".to_string()),
                    Value::Regex(Regex::new("argument").unwrap()),
                ],
            },
            InstructionOrDirective::Instruction {
                time: Some((TimeSpec::Absolute, Fraction::new(0u64, 1u64))),
                name: "timed_instruction".to_string(),
                args: vec![Value::Identifier("arg".to_string())],
            },
            InstructionOrDirective::Instruction {
                time: Some((
                    TimeSpec::Relative {
                        from_start: false,
                        positive: false,
                    },
                    Fraction::new(1u64, 1u64),
                )),
                name: "negative_timed_instruction".to_string(),
                args: vec![Value::Identifier("arg".to_string())],
            },
            InstructionOrDirective::Instruction {
                time: Some((
                    TimeSpec::Relative {
                        from_start: true,
                        positive: true,
                    },
                    Fraction::new(2u64, 1u64),
                )),
                name: "positive_start_timed_instruction".to_string(),
                args: vec![Value::Identifier("arg".to_string())],
            },
        ];

        let actual = parse(&input).expect("Failed to parse");

        assert_eq!(actual, expected);
    }
}
