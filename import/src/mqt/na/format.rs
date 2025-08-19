//! Parsing and Serialization for the [`mqt` na file-format][super]

use std::{fmt::Display, sync::Arc};

use fraction::{Decimal, Fraction};
use operation::operation;
use winnow::{
    ascii::{multispace0, Caseless},
    combinator::{preceded, repeat, terminated},
    stream::{AsBStr, AsChar, Compare, ParseSlice, Stream, StreamIsPartial},
    Parser,
};

use crate::separated_display::SeparatedDisplay;

/// An operation: `<name><arguments>;`
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Operation<I> {
    /// `<name>`
    pub name: I,
    /// `<arguments>`
    pub args: OperationArgs,
}

impl<I: Display> Display for Operation<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{};", &self.name, &self.args)
    }
}

/// Arguments for the different types of instructions
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum OperationArgs {
    /// `<name> at <positions>`
    Init(PositionList),
    /// `<name> <positions> to <positions>`
    Shuttle {
        from: PositionList,
        to: PositionList,
    },
    /// `<name>(<number>) at <positions>` / `<name> at <positions>`
    Local {
        argument: Option<Number>,
        targets: PositionList,
    },
    /// `<name>(<number>)` / `<name>`
    Global(Option<Number>),
}

impl Display for OperationArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Init(positions) => write!(f, " at {}", SeparatedDisplay::comma(positions)),
            Self::Shuttle { from, to } => {
                write!(
                    f,
                    " {} to {}",
                    SeparatedDisplay::comma(from),
                    SeparatedDisplay::comma(to)
                )
            }
            Self::Local { argument, targets } => {
                if let Some(argument) = argument {
                    write!(f, "({})", Decimal::from_fraction(*argument))?;
                }
                write!(f, " at {}", SeparatedDisplay::comma(targets))
            }
            Self::Global(argument) => {
                if let Some(argument) = argument {
                    write!(f, "({})", Decimal::from_fraction(*argument))?;
                }
                Ok(())
            }
        }
    }
}

/// The parsed file; a list of [Operation]s.
pub type OperationList<I> = Arc<[Operation<I>]>;

/// A position with an `x`- and `y`-coordinate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Position {
    pub x: Number,
    pub y: Number,
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}, {})",
            Decimal::from_fraction(self.x),
            Decimal::from_fraction(self.y)
        )
    }
}

/// A list of multiple positions
pub type PositionList = Arc<[Position]>;

/// A number from the parsed format
pub type Number = Fraction;

/// Error returned when parsing.
/// Contains Reference to the input.
pub type ParseError<I> = winnow::error::ParseError<I, ParseErrorInner>;

/// Inner error in [ParseError].
pub type ParseErrorInner = winnow::error::ContextError;

/// Parses an `na`-file into an [OperationList]
pub fn parse<
    I: Stream
        + StreamIsPartial
        + Compare<Caseless<&'static str>>
        + Compare<char>
        + Compare<&'static str>
        + AsBStr,
>(
    input: I,
) -> Result<OperationList<I::Slice>, ParseError<I>>
where
    I::Slice: ParseSlice<Number>,
    I::Token: AsChar + Clone,
    I::IterOffsets: Clone,
{
    preceded(
        multispace0,
        repeat::<_, _, Vec<_>, _, _>(.., terminated(operation, multispace0)),
    )
    .output_into()
    .parse(input)
}

/// Parsers for the individual parts of the format
pub mod parts {
    use winnow::{
        ascii::{float, multispace0, Caseless},
        combinator::{delimited, separated, terminated},
        stream::{AsBStr, AsChar, Compare, ParseSlice, Stream, StreamIsPartial},
        ModalResult, Parser,
    };

    use super::{Number, Position, PositionList};

    /// Tries to parse a [Number]
    pub fn number<
        I: Stream + StreamIsPartial + Compare<Caseless<&'static str>> + Compare<char> + AsBStr,
    >(
        input: &mut I,
    ) -> ModalResult<Number>
    where
        I::Slice: ParseSlice<Number>,
        I::Token: AsChar + Clone,
        I::IterOffsets: Clone,
    {
        float.parse_next(input)
    }

    /// Tries to parse a [Position]
    pub fn position<
        I: Stream
            + StreamIsPartial
            + Compare<Caseless<&'static str>>
            + Compare<char>
            + Compare<&'static str>
            + AsBStr,
    >(
        input: &mut I,
    ) -> ModalResult<Position>
    where
        I::Slice: ParseSlice<Number>,
        I::Token: AsChar + Clone,
        I::IterOffsets: Clone,
    {
        (
            terminated("(", multispace0),
            terminated(number, multispace0),
            terminated(",", multispace0),
            terminated(number, multispace0),
            terminated(")", multispace0),
        )
            .map(|(_, x, _, y, _)| Position { x, y })
            .parse_next(input)
    }

    /// Tries to parse a [PositionList]
    pub fn position_list<
        I: Stream
            + StreamIsPartial
            + Compare<Caseless<&'static str>>
            + Compare<char>
            + Compare<&'static str>
            + AsBStr,
    >(
        input: &mut I,
    ) -> ModalResult<PositionList>
    where
        I::Slice: ParseSlice<Number>,
        I::Token: AsChar + Clone,
        I::IterOffsets: Clone,
    {
        separated::<_, _, Vec<_>, _, _, _, _>(
            ..,
            position,
            delimited(multispace0, ",", multispace0),
        )
        .output_into()
        .parse_next(input)
    }
}

/// Parsers for whole [Operation]s of the format
pub mod operation {
    use winnow::{
        ascii::{alphanumeric1, multispace0, Caseless},
        combinator::{alt, delimited, opt, terminated},
        stream::{AsBStr, AsChar, Compare, ParseSlice, Stream, StreamIsPartial},
        ModalResult, Parser,
    };

    use super::{
        parts::{number, position_list},
        Number, Operation, OperationArgs,
    };

    /// Tries to parse an [Operation] of type [OperationArgs::Init]
    pub fn init<
        I: Stream
            + StreamIsPartial
            + Compare<Caseless<&'static str>>
            + Compare<char>
            + Compare<&'static str>
            + AsBStr,
    >(
        input: &mut I,
    ) -> ModalResult<Operation<I::Slice>>
    where
        I::Slice: ParseSlice<Number>,
        I::Token: AsChar + Clone,
        I::IterOffsets: Clone,
    {
        (
            terminated("init", multispace0),
            terminated("at", multispace0),
            terminated(position_list, multispace0),
            ";",
        )
            .map(|(name, _, p, _)| Operation {
                name,
                args: OperationArgs::Init(p),
            })
            .parse_next(input)
    }

    /// Tries to parse an [Operation] of type [OperationArgs::Shuttle]
    pub fn shuttle<
        I: Stream
            + StreamIsPartial
            + Compare<Caseless<&'static str>>
            + Compare<char>
            + Compare<&'static str>
            + AsBStr,
    >(
        input: &mut I,
    ) -> ModalResult<Operation<I::Slice>>
    where
        I::Slice: ParseSlice<Number>,
        I::Token: AsChar + Clone,
        I::IterOffsets: Clone,
    {
        (
            terminated(alphanumeric1, multispace0),
            terminated(position_list, multispace0),
            terminated("to", multispace0),
            terminated(position_list, multispace0),
            ";",
        )
            .map(|(name, from, _, to, _)| Operation {
                name,
                args: OperationArgs::Shuttle { from, to },
            })
            .parse_next(input)
    }

    /// Tries to parse an [Operation] of type [OperationArgs::Local]
    pub fn local<
        I: Stream
            + StreamIsPartial
            + Compare<Caseless<&'static str>>
            + Compare<char>
            + Compare<&'static str>
            + AsBStr,
    >(
        input: &mut I,
    ) -> ModalResult<Operation<I::Slice>>
    where
        I::Slice: ParseSlice<Number>,
        I::Token: AsChar + Clone,
        I::IterOffsets: Clone,
    {
        (
            alphanumeric1,
            opt(terminated(
                delimited(("(", multispace0), number, (multispace0, ")")),
                multispace0,
            )),
            terminated("at", multispace0),
            terminated(position_list, multispace0),
            ";",
        )
            .map(|(name, argument, _, targets, _)| Operation {
                name,
                args: OperationArgs::Local { argument, targets },
            })
            .parse_next(input)
    }

    /// Tries to parse an [Operation] of type [OperationArgs::Global]
    pub fn global<
        I: Stream
            + StreamIsPartial
            + Compare<Caseless<&'static str>>
            + Compare<char>
            + Compare<&'static str>
            + AsBStr,
    >(
        input: &mut I,
    ) -> ModalResult<Operation<I::Slice>>
    where
        I::Slice: ParseSlice<Number>,
        I::Token: AsChar + Clone,
        I::IterOffsets: Clone,
    {
        (
            alphanumeric1,
            opt(terminated(
                delimited(("(", multispace0), number, (multispace0, ")")),
                multispace0,
            )),
            ";",
        )
            .map(|(name, argument, _)| Operation {
                name,
                args: OperationArgs::Global(argument),
            })
            .parse_next(input)
    }

    /// Tries to parse any [Operation]
    pub fn operation<
        I: Stream
            + StreamIsPartial
            + Compare<Caseless<&'static str>>
            + Compare<char>
            + Compare<&'static str>
            + AsBStr,
    >(
        input: &mut I,
    ) -> ModalResult<Operation<I::Slice>>
    where
        I::Slice: ParseSlice<Number>,
        I::Token: AsChar + Clone,
        I::IterOffsets: Clone,
    {
        alt((init, shuttle, local, global)).parse_next(input)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        mqt::na::format::{Operation, OperationArgs, Position},
        separated_display::SeparatedDisplay,
    };

    use super::parse;

    /// Round trip of the example-file
    #[test]
    fn example_round_trip() {
        // Example-file from https://mqt.readthedocs.io/projects/qmap/en/latest/NAStatePrep.html#codecell11
        let input = include_str!("../../../rsc/test/example.na").trim();

        let parsed = parse(input).expect("Failed to parse!");

        let stringified = SeparatedDisplay::newline(&parsed).to_string();

        assert_eq!(
            input, &stringified,
            "Round-Trip failed to produce same serialized result file"
        );

        let parsed_again = parse(stringified.as_str()).expect("Failed to parse!");

        assert_eq!(
            parsed, parsed_again,
            "Round-Trip failed to produce same deserialized result"
        );
    }

    #[test]
    fn instruction_position_list_parse() {
        let input = "ry(42) at (0, 0), (1, 1), (2, 2), (3, 3);";

        let parsed = parse(input).expect("Failed to parse!");

        let expected = [Operation {
            name: "ry",
            args: OperationArgs::Local {
                argument: Some(42.into()),
                targets: [
                    Position {
                        x: 0.into(),
                        y: 0.into(),
                    },
                    Position {
                        x: 1.into(),
                        y: 1.into(),
                    },
                    Position {
                        x: 2.into(),
                        y: 2.into(),
                    },
                    Position {
                        x: 3.into(),
                        y: 3.into(),
                    },
                ]
                .into(),
            },
        }]
        .into();

        assert_eq!(
            parsed, expected,
            "Wrongly parsed instruction with position list"
        );
    }

    #[test]
    fn instruction_position_list_stringify() {
        let input = Operation {
            name: "ry",
            args: OperationArgs::Local {
                argument: Some(1337.into()),
                targets: [
                    Position {
                        x: 4.into(),
                        y: 4.into(),
                    },
                    Position {
                        x: 3.into(),
                        y: 3.into(),
                    },
                    Position {
                        x: 2.into(),
                        y: 2.into(),
                    },
                    Position {
                        x: 1.into(),
                        y: 1.into(),
                    },
                ]
                .into(),
            },
        };

        let exported = input.to_string();

        let expected = "ry(1337) at (4, 4), (3, 3), (2, 2), (1, 1);";

        assert_eq!(
            exported, expected,
            "Wrongly stringified instruction with position list"
        );
    }
}
