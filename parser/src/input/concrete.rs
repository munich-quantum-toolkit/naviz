//! A concrete format for the `.naviz`-format.
//! Collects parsed instructions and directives into an [Instructions]-object.
//! [TimedInstruction]s are collected into an [AbsoluteTimeline],
//! which in turn contains [RelativeTimeline]s.

use super::{
    lexer::TimeSpec,
    parser::{InstructionOrDirective, Value},
};
use crate::config::position::Position;
use fraction::{Fraction, Zero};

/// Timeline which has multiple relative timelines starting at fixed positions.
pub type AbsoluteTimeline = Vec<(Fraction, RelativeTimeline)>;

/// Timeline which shows relative times.
/// Item format: `(from_start, offset, instruction)`.
/// Each entry is relative to the previous entry.
pub type RelativeTimeline = Vec<(bool, Fraction, TimedInstruction)>;

/// A single instruction which does not require a time.
/// See documentation of file format.
#[derive(Debug, PartialEq)]
pub enum SetupInstruction {
    Atom { position: Position, id: String },
}

/// A single instruction which requires a time.
/// See documentation of file format.
#[derive(Debug, PartialEq)]
pub enum TimedInstruction {
    Load {
        position: Option<Position>,
        id: String,
    },
    Store {
        position: Option<Position>,
        id: String,
    },
    Move {
        position: Position,
        id: String,
    },
    Rz {
        value: Fraction,
        id: String,
    },
    Ry {
        value: Fraction,
        id: String,
    },
    Cz {
        id: String,
    },
}

/// The parsed directives.
/// See documentation of file format.
#[derive(Default, Debug, PartialEq)]
pub struct Directives {
    pub targets: Vec<String>,
}

/// The parsed instructions, split into [Directives], [SetupInstruction]s, and [TimedInstruction]s.
#[derive(Default, Debug, PartialEq)]
pub struct Instructions {
    pub directives: Directives,
    pub setup: Vec<SetupInstruction>,
    pub instructions: AbsoluteTimeline,
}

/// Error during the parsing of instructions in [Instructions::new].
#[derive(Debug)]
pub enum ParseInstructionsError {
    /// Encountered an unknown instruction
    UnknownInstruction {
        /// Name of instruction
        name: String,
    },
    /// Encountered an unknown directive
    UnknownDirective {
        /// Name of directive
        name: String,
    },
    /// Instruction or directive has a wrong number of arguments
    WrongNumberOfArguments {
        /// Name of instruction or directive
        name: &'static str,
        /// Expected number of arguments to be one of these
        expected: &'static [usize],
        /// Actually got this many arguments
        actual: usize,
    },
    /// Instruction or directive was called with wrong type of argument
    WrongTypeOfArgument {
        /// Name of instruction or directive
        name: &'static str,
        /// Expected one of these types of arguments
        /// (First array is options; second is types for single option)
        expected: &'static [&'static [&'static str]],
    },
    /// A [TimedInstruction] is missing a time
    MissingTime {
        /// Name of instruction or directive
        name: &'static str,
    },
    /// A [SetupInstruction] was given a time
    SuperfluousTime {
        /// Name of instruction or directive
        name: &'static str,
    },
}

impl Instructions {
    /// Try to parse [Instructions] from a [Vec] of [InstructionOrDirective]s.
    pub fn new(input: Vec<InstructionOrDirective>) -> Result<Self, ParseInstructionsError> {
        let mut instructions = Instructions::default();

        let mut prev = None;

        for i in input {
            match i {
                InstructionOrDirective::Directive { name, args } => match name.as_str() {
                    "target" => {
                        let id = id(args, "#target")?;
                        instructions.directives.targets.push(id);
                    }
                    _ => return Err(ParseInstructionsError::UnknownDirective { name }),
                },

                InstructionOrDirective::Instruction { time, name, args } => match name.as_str() {
                    "atom" => {
                        if time.is_some() {
                            return Err(ParseInstructionsError::SuperfluousTime { name: "atom" });
                        }

                        let (position, id) = position_id(args, "atom")?;
                        instructions
                            .setup
                            .push(SetupInstruction::Atom { position, id });
                    }
                    "load" => {
                        let (position, id) = maybe_position_id(args, "load")?;
                        insert_at_time(
                            time,
                            "load",
                            TimedInstruction::Load { position, id },
                            &mut prev,
                            &mut instructions.instructions,
                        )?;
                    }
                    "store" => {
                        let (position, id) = maybe_position_id(args, "store")?;
                        insert_at_time(
                            time,
                            "store",
                            TimedInstruction::Store { position, id },
                            &mut prev,
                            &mut instructions.instructions,
                        )?;
                    }
                    "move" => {
                        let (position, id) = position_id(args, "move")?;
                        insert_at_time(
                            time,
                            "move",
                            TimedInstruction::Move { position, id },
                            &mut prev,
                            &mut instructions.instructions,
                        )?;
                    }
                    "rz" => {
                        let (value, id) = number_id(args, "rz")?;
                        insert_at_time(
                            time,
                            "rz",
                            TimedInstruction::Rz { value, id },
                            &mut prev,
                            &mut instructions.instructions,
                        )?;
                    }
                    "ry" => {
                        let (value, id) = number_id(args, "ry")?;
                        insert_at_time(
                            time,
                            "ry",
                            TimedInstruction::Ry { value, id },
                            &mut prev,
                            &mut instructions.instructions,
                        )?;
                    }
                    "cz" => {
                        let id = id(args, "cz")?;
                        insert_at_time(
                            time,
                            "cz",
                            TimedInstruction::Cz { id },
                            &mut prev,
                            &mut instructions.instructions,
                        )?;
                    }
                    _ => return Err(ParseInstructionsError::UnknownInstruction { name }),
                },
            }
        }

        instructions.instructions.sort_unstable_by_key(|e| e.0);

        Ok(instructions)
    }
}

/// Inserts a [TimedInstruction] into the [AbsoluteTimeline] at the specified `time`,
/// while keeping track of the insertion port and handling relative times.
///
/// # Arguments:
///
/// - `time`: Time to insert. Will return an [ParseInstructionsError::MissingTime] if [None]
/// - `name`: Name of the instruction; used for [ParseInstructionsError::MissingTime::name]
/// - `instruction`: Instruction to insert
/// - `prev`: Previous insertion-point (or [None] if nothing was previously inserted).
///   Will be updated to be the new insertion-point.
///   Value is the index in `target` (and is assumed to be valid).
/// - `target`: Target timeline to insert into
fn insert_at_time(
    time: Option<(TimeSpec, Fraction)>,
    name: &'static str,
    instruction: TimedInstruction,
    prev: &mut Option<usize>,
    target: &mut AbsoluteTimeline,
) -> Result<(), ParseInstructionsError> {
    let (spec, mut time) = time.ok_or(ParseInstructionsError::MissingTime { name })?;
    match spec {
        TimeSpec::Absolute => {
            target.push((time, vec![(true, Fraction::zero(), instruction)]));
            *prev = Some(target.len() - 1);
        }
        TimeSpec::Relative {
            from_start,
            positive,
        } => {
            if !positive {
                time *= -1;
            }
            if let Some(idx) = prev {
                target[*idx].1.push((from_start, time, instruction));
                // prev stays the same
            } else {
                target.push((time, vec![(from_start, time, instruction)]));
                *prev = Some(target.len() - 1);
            }
        }
    }
    Ok(())
}

/// Tries to parse the arguments into a position and an id.
/// Returns a [ParseInstructionsError] if there is a wrong number of arguments
/// or they have wrong types.
fn position_id(
    args: Vec<Value>,
    name: &'static str,
) -> Result<(Position, String), ParseInstructionsError> {
    let error = || ParseInstructionsError::WrongTypeOfArgument {
        name,
        expected: &[&["position", "id"]],
    };

    match n_args(args, name, &[2])? {
        [Value::Tuple(t), Value::Identifier(id)] => match maybe_get_n(t).map_err(|_| error())? {
            [Value::Number(x), Value::Number(y)] => Ok(((x, y), id)),
            _ => Err(error()),
        },
        _ => Err(error()),
    }
}

/// Tries to parse the arguments into a position and an id or into just an id.
/// Returns a [ParseInstructionsError] if there is a wrong number of arguments
/// or they have wrong types.
fn maybe_position_id(
    args: Vec<Value>,
    name: &'static str,
) -> Result<(Option<Position>, String), ParseInstructionsError> {
    let error = || ParseInstructionsError::WrongTypeOfArgument {
        name,
        expected: &[&["position", "id"], &["id"]],
    };

    match maybe_get_n(args) {
        Ok(args) => match args {
            [Value::Tuple(t), Value::Identifier(id)] => {
                match maybe_get_n(t).map_err(|_| error())? {
                    [Value::Number(x), Value::Number(y)] => Ok((Some((x, y)), id)),
                    _ => Err(error()),
                }
            }
            _ => Err(error()),
        },
        Err(args) => match n_args(args, name, &[1, 2])? {
            [Value::Identifier(id)] => Ok((None, id)),
            _ => Err(error()),
        },
    }
}

/// Tries to parse the arguments into a number and an id.
/// Returns a [ParseInstructionsError] if there is a wrong number of arguments
/// or they have wrong types.
fn number_id(
    args: Vec<Value>,
    name: &'static str,
) -> Result<(Fraction, String), ParseInstructionsError> {
    let error = || ParseInstructionsError::WrongTypeOfArgument {
        name,
        expected: &[&["number", "id"]],
    };

    match n_args(args, name, &[2])? {
        [Value::Number(n), Value::Identifier(id)] => Ok((n, id)),
        _ => Err(error()),
    }
}

/// Tries to parse the arguments into just an id.
/// Returns a [ParseInstructionsError] if there is a wrong number of arguments
/// or they have wrong types.
fn id(args: Vec<Value>, name: &'static str) -> Result<String, ParseInstructionsError> {
    let error = || ParseInstructionsError::WrongTypeOfArgument {
        name,
        expected: &[&["id"]],
    };

    match n_args(args, name, &[1])? {
        [Value::Identifier(id)] => Ok(id),
        _ => Err(error()),
    }
}

/// Returns a slice of length `N` if the passed vector has length `N`,
/// or an error containing the original vector.
///
/// Typed wrapper around [Vec::try_into].
pub fn maybe_get_n<T, const N: usize>(vec: Vec<T>) -> Result<[T; N], Vec<T>> {
    vec.try_into()
}

/// Returns a slice of length `N` if the passed vector has length `N`,
/// or [ParseInstructionsError::WrongNumberOfArguments].
pub fn n_args<T, const N: usize>(
    vec: Vec<T>,
    name: &'static str,
    expected: &'static [usize],
) -> Result<[T; N], ParseInstructionsError> {
    maybe_get_n(vec).map_err(|e| ParseInstructionsError::WrongNumberOfArguments {
        name,
        expected,
        actual: e.len(),
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::input::{lexer, parser};

    #[test]
    pub fn example() {
        let input = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/rsc/test/example.naviz"
        ));

        let expected = Instructions {
            directives: Directives {
                targets: vec!["example".to_string()],
            },
            setup: vec![
                SetupInstruction::Atom {
                    position: (Fraction::new(0u64, 1u64), Fraction::new(0u64, 1u64)),
                    id: "atom0".to_string(),
                },
                SetupInstruction::Atom {
                    position: (Fraction::new(16u64, 1u64), Fraction::new(0u64, 1u64)),
                    id: "atom1".to_string(),
                },
                SetupInstruction::Atom {
                    position: (Fraction::new(32u64, 1u64), Fraction::new(0u64, 1u64)),
                    id: "atom2".to_string(),
                },
            ],
            instructions: vec![(
                Fraction::new(0u64, 1u64),
                vec![
                    (
                        false,
                        Fraction::new(0u64, 1u64),
                        TimedInstruction::Load {
                            position: None,
                            id: "atom0".to_string(),
                        },
                    ),
                    (
                        true,
                        Fraction::new(0u64, 1u64),
                        TimedInstruction::Load {
                            position: Some((Fraction::new(16u64, 1u64), Fraction::new(2u64, 1u64))),
                            id: "atom1".to_string(),
                        },
                    ),
                    (
                        false,
                        Fraction::new(0u64, 1u64),
                        TimedInstruction::Move {
                            position: (Fraction::new(8u64, 1u64), Fraction::new(8u64, 1u64)),
                            id: "atom0".to_string(),
                        },
                    ),
                    (
                        true,
                        Fraction::new(0u64, 1u64),
                        TimedInstruction::Move {
                            position: (Fraction::new(16u64, 1u64), Fraction::new(16u64, 1u64)),
                            id: "atom1".to_string(),
                        },
                    ),
                    (
                        false,
                        Fraction::new(0u64, 1u64),
                        TimedInstruction::Store {
                            position: None,
                            id: "atom0".to_string(),
                        },
                    ),
                    (
                        true,
                        Fraction::new(0u64, 1u64),
                        TimedInstruction::Store {
                            position: None,
                            id: "atom1".to_string(),
                        },
                    ),
                    (
                        false,
                        Fraction::new(0u64, 1u64),
                        TimedInstruction::Rz {
                            value: Fraction::new(3141u64, 1000u64),
                            id: "atom0".to_string(),
                        },
                    ),
                    (
                        false,
                        Fraction::new(0u64, 1u64),
                        TimedInstruction::Ry {
                            value: Fraction::new(3141u64, 1000u64),
                            id: "atom1".to_string(),
                        },
                    ),
                    (
                        false,
                        Fraction::new(0u64, 1u64),
                        TimedInstruction::Cz {
                            id: "zone0".to_string(),
                        },
                    ),
                ],
            )],
        };

        let lexed = lexer::lex(input).expect("Failed to lex");
        let parsed = parser::parse(&lexed).expect("Failed to parse");
        let concrete =
            Instructions::new(parsed).expect("Failed to parse into concrete instructions");

        assert_eq!(concrete, expected);
    }

    #[test]
    pub fn simple_example() {
        let input = vec![
            InstructionOrDirective::Directive {
                name: "target".to_string(),
                args: vec![Value::Identifier("machine_a".to_string())],
            },
            InstructionOrDirective::Directive {
                name: "target".to_string(),
                args: vec![Value::Identifier("machine_b".to_string())],
            },
            InstructionOrDirective::Instruction {
                time: None,
                name: "atom".to_string(),
                args: vec![
                    Value::Tuple(vec![
                        Value::Number(Fraction::new(0u64, 1u64)),
                        Value::Number(Fraction::new(0u64, 1u64)),
                    ]),
                    Value::Identifier("atom1".to_string()),
                ],
            },
            InstructionOrDirective::Instruction {
                time: Some((
                    TimeSpec::Relative {
                        from_start: false,
                        positive: true,
                    },
                    Fraction::new(0u64, 1u64),
                )),
                name: "load".to_string(),
                args: vec![Value::Identifier("atom1".to_string())],
            },
            InstructionOrDirective::Instruction {
                time: Some((
                    TimeSpec::Relative {
                        from_start: true,
                        positive: true,
                    },
                    Fraction::new(2u64, 1u64),
                )),
                name: "store".to_string(),
                args: vec![Value::Identifier("atom1".to_string())],
            },
            InstructionOrDirective::Instruction {
                time: Some((
                    TimeSpec::Relative {
                        from_start: false,
                        positive: true,
                    },
                    Fraction::new(3u64, 1u64),
                )),
                name: "load".to_string(),
                args: vec![Value::Identifier("atom1".to_string())],
            },
            InstructionOrDirective::Instruction {
                time: Some((TimeSpec::Absolute, Fraction::new(20u64, 1u64))),
                name: "store".to_string(),
                args: vec![Value::Identifier("atom1".to_string())],
            },
            InstructionOrDirective::Instruction {
                time: Some((
                    TimeSpec::Relative {
                        from_start: true,
                        positive: true,
                    },
                    Fraction::new(2u64, 1u64),
                )),
                name: "load".to_string(),
                args: vec![Value::Identifier("atom1".to_string())],
            },
            InstructionOrDirective::Instruction {
                time: Some((
                    TimeSpec::Relative {
                        from_start: false,
                        positive: true,
                    },
                    Fraction::new(0u64, 1u64),
                )),
                name: "store".to_string(),
                args: vec![Value::Identifier("atom1".to_string())],
            },
        ];

        let expected = Instructions {
            directives: Directives {
                targets: vec!["machine_a".to_string(), "machine_b".to_string()],
            },
            setup: vec![SetupInstruction::Atom {
                position: (Fraction::new(0u64, 1u64), Fraction::new(0u64, 1u64)),
                id: "atom1".to_string(),
            }],
            instructions: vec![
                (
                    Fraction::new(0u64, 1u64),
                    vec![
                        (
                            false,
                            Fraction::zero(),
                            TimedInstruction::Load {
                                position: None,
                                id: "atom1".to_string(),
                            },
                        ),
                        (
                            true,
                            Fraction::new(2u64, 1u64),
                            TimedInstruction::Store {
                                position: None,
                                id: "atom1".to_string(),
                            },
                        ),
                        (
                            false,
                            Fraction::new(3u64, 1u64),
                            TimedInstruction::Load {
                                position: None,
                                id: "atom1".to_string(),
                            },
                        ),
                    ],
                ),
                (
                    Fraction::new(20u64, 1u64),
                    vec![
                        (
                            true,
                            Fraction::zero(),
                            TimedInstruction::Store {
                                position: None,
                                id: "atom1".to_string(),
                            },
                        ),
                        (
                            true,
                            Fraction::new(2u64, 1u64),
                            TimedInstruction::Load {
                                position: None,
                                id: "atom1".to_string(),
                            },
                        ),
                        (
                            false,
                            Fraction::new(0u64, 1u64),
                            TimedInstruction::Store {
                                position: None,
                                id: "atom1".to_string(),
                            },
                        ),
                    ],
                ),
            ],
        };

        let actual = Instructions::new(input).expect("Failed to parse into tree");

        assert_eq!(actual, expected);
    }
}
