//! A concrete format for the `.naviz`-format.
//! Collects parsed instructions and directives into an [Instructions]-object.
//! [TimedInstruction]s are collected into an [AbsoluteTimeline],
//! which in turn contains [RelativeTimeline]s.

use std::borrow::Cow;

use super::{
    lexer::TimeSpec,
    parser::{InstructionOrDirective, Value},
};
use crate::config::position::Position;
use fraction::{Fraction, Zero};
use itertools::{Either, Itertools};

/// Timeline which has multiple relative timelines starting at fixed positions.
pub type AbsoluteTimeline = Vec<(Fraction, RelativeTimeline)>;

/// Timeline which shows relative times.
/// Item format: `(from_start, offset, instruction)`.
/// Each entry is relative to the previous entry.
pub type RelativeTimeline = Vec<(bool, Fraction, InstructionGroup)>;

/// A group of instructions.
/// Single instructions can be represented as groups of size `1`
#[derive(Debug, PartialEq, Clone)]
pub struct InstructionGroup {
    /// Whether this is a variable group
    /// (i.e., the timing of the instructions is allowed to vary)
    pub variable: bool,
    /// The instructions of this group
    pub instructions: Vec<TimedInstruction>,
}

/// A single instruction which does not require a time.
/// See documentation of file format.
#[derive(Debug, PartialEq, Clone)]
pub enum SetupInstruction {
    Atom { position: Position, id: String },
}

impl SetupInstruction {
    /// Get the name of a [SetupInstruction]
    pub fn str(&self) -> &'static str {
        match self {
            Self::Atom { .. } => "atom",
        }
    }
}

/// A single instruction which requires a time.
/// See documentation of file format.
#[derive(Debug, PartialEq, Clone)]
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
        targets: Vec<String>,
    },
    Ry {
        value: Fraction,
        targets: Vec<String>,
    },
    Cz {
        targets: Vec<String>,
    },
}

impl TimedInstruction {
    /// Get the name of a [TimedInstruction]
    pub fn str(&self) -> &'static str {
        match self {
            Self::Load { .. } => "load",
            Self::Store { .. } => "store",
            Self::Move { .. } => "move",
            Self::Rz { .. } => "rz",
            Self::Ry { .. } => "ry",
            Self::Cz { .. } => "cz",
        }
    }
}

/// The parsed directives.
/// See documentation of file format.
#[derive(Default, Debug, PartialEq, Clone)]
pub struct Directives {
    pub targets: Vec<String>,
}

/// The parsed instructions, split into [Directives], [SetupInstruction]s, and [TimedInstruction]s.
#[derive(Default, Debug, PartialEq, Clone)]
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
        /// Name of instructions or directives
        name: Vec<&'static str>,
    },
    /// A [SetupInstruction] was given a time
    SuperfluousTime {
        /// Name of instructions or directives
        name: Vec<&'static str>,
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

                InstructionOrDirective::Instruction { time, name, args } => {
                    match parse_instruction(name.into(), args)? {
                        Instruction::SetupInstruction(setup) => {
                            if time.is_some() {
                                Err(ParseInstructionsError::SuperfluousTime {
                                    name: vec![setup.str()],
                                })?
                            }
                            instructions.setup.push(setup);
                        }
                        Instruction::TimedInstruction(instruction) => insert_at_time(
                            time,
                            false,
                            vec![instruction],
                            &mut prev,
                            &mut instructions.instructions,
                        )?,
                    }
                }

                InstructionOrDirective::GroupedTime {
                    time,
                    variable,
                    group,
                } => {
                    let (setup, timed): (Vec<_>, Vec<_>) = group
                        .into_iter()
                        .map(|(name, args)| parse_instruction(name.into(), args))
                        .process_results(|i| {
                            i.partition_map(|i| match i {
                                Instruction::SetupInstruction(setup) => Either::Left(setup),
                                Instruction::TimedInstruction(instruction) => {
                                    Either::Right(instruction)
                                }
                            })
                        })?;
                    if !setup.is_empty() && time.is_some() {
                        Err(ParseInstructionsError::SuperfluousTime {
                            name: setup.iter().map(SetupInstruction::str).collect(),
                        })?
                    }
                    setup.into_iter().for_each(|s| instructions.setup.push(s));
                    insert_at_time(
                        time,
                        variable,
                        timed,
                        &mut prev,
                        &mut instructions.instructions,
                    )?;
                }

                InstructionOrDirective::GroupedInstruction {
                    time,
                    variable,
                    name,
                    group,
                } => {
                    let (setup, timed): (Vec<_>, Vec<_>) = group
                        .into_iter()
                        .map(|args| parse_instruction(name.as_str().into(), args))
                        .process_results(|i| {
                            i.partition_map(|i| match i {
                                Instruction::SetupInstruction(setup) => Either::Left(setup),
                                Instruction::TimedInstruction(instruction) => {
                                    Either::Right(instruction)
                                }
                            })
                        })?;
                    if !setup.is_empty() && time.is_some() {
                        Err(ParseInstructionsError::SuperfluousTime {
                            name: setup.iter().map(SetupInstruction::str).collect(),
                        })?
                    }
                    setup.into_iter().for_each(|s| instructions.setup.push(s));
                    insert_at_time(
                        time,
                        variable,
                        timed,
                        &mut prev,
                        &mut instructions.instructions,
                    )?;
                }
            }
        }

        instructions.instructions.sort_unstable_by_key(|e| e.0);

        Ok(instructions)
    }
}

/// An instruction: Either a [TimedInstruction] or a [SetupInstruction]
enum Instruction {
    TimedInstruction(TimedInstruction),
    SetupInstruction(SetupInstruction),
}

impl From<SetupInstruction> for Instruction {
    fn from(value: SetupInstruction) -> Self {
        Self::SetupInstruction(value)
    }
}

impl From<TimedInstruction> for Instruction {
    fn from(value: TimedInstruction) -> Self {
        Self::TimedInstruction(value)
    }
}

/// Parses a single [Instruction] from the instruction name and its arguments.
/// Will return an [Err] if incompatible arguments were given.
fn parse_instruction(
    name: Cow<str>,
    args: Vec<Value>,
) -> Result<Instruction, ParseInstructionsError> {
    Ok(match &*name {
        "atom" => {
            let (position, id) = position_id(args, "atom")?;
            SetupInstruction::Atom { position, id }.into()
        }
        "load" => {
            let (position, id) = maybe_position_id(args, "load")?;
            TimedInstruction::Load { position, id }.into()
        }
        "store" => {
            let (position, id) = maybe_position_id(args, "store")?;
            TimedInstruction::Store { position, id }.into()
        }
        "move" => {
            let (position, id) = position_id(args, "move")?;
            TimedInstruction::Move { position, id }.into()
        }
        "rz" => {
            let (value, targets) = number_target(args, "rz")?;
            TimedInstruction::Rz { value, targets }.into()
        }
        "ry" => {
            let (value, targets) = number_target(args, "ry")?;
            TimedInstruction::Ry { value, targets }.into()
        }
        "cz" => {
            let targets = target(args, "cz")?;
            TimedInstruction::Cz { targets }.into()
        }
        _ => Err(ParseInstructionsError::UnknownInstruction {
            name: name.into_owned(),
        })?,
    })
}

/// Inserts a [TimedInstruction] into the [AbsoluteTimeline] at the specified `time`,
/// while keeping track of the insertion port and handling relative times.
///
/// # Arguments:
///
/// - `time`: Time to insert. Will return an [ParseInstructionsError::MissingTime] if [None]
/// - `variable`: Whether the group to insert is variable
/// - `instructions`: Instructions to insert as a group
/// - `prev`: Previous insertion-point (or [None] if nothing was previously inserted).
///   Will be updated to be the new insertion-point.
///   Value is the index in `target` (and is assumed to be valid).
/// - `target`: Target timeline to insert into
fn insert_at_time(
    time: Option<(TimeSpec, Fraction)>,
    variable: bool,
    instructions: Vec<TimedInstruction>,
    prev: &mut Option<usize>,
    target: &mut AbsoluteTimeline,
) -> Result<(), ParseInstructionsError> {
    if instructions.is_empty() {
        return Ok(());
    }
    let (spec, mut time) = time.ok_or_else(|| ParseInstructionsError::MissingTime {
        name: instructions.iter().map(TimedInstruction::str).collect(),
    })?;
    match spec {
        TimeSpec::Absolute => {
            target.push((
                time,
                vec![(
                    true,
                    Fraction::zero(),
                    InstructionGroup {
                        variable,
                        instructions,
                    },
                )],
            ));
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
                target[*idx].1.push((
                    from_start,
                    time,
                    InstructionGroup {
                        variable,
                        instructions,
                    },
                ));
                // prev stays the same
            } else {
                target.push((
                    time,
                    vec![(
                        from_start,
                        time,
                        InstructionGroup {
                            variable,
                            instructions,
                        },
                    )],
                ));
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

/// Tries to parse the arguments into a number and a target.
/// Returns a [ParseInstructionsError] if there is a wrong number of arguments
/// or they have wrong types.
fn number_target(
    args: Vec<Value>,
    name: &'static str,
) -> Result<(Fraction, Vec<String>), ParseInstructionsError> {
    let error = || ParseInstructionsError::WrongTypeOfArgument {
        name,
        expected: &[&["number", "id"]],
    };

    match n_args(args, name, &[2])? {
        [Value::Number(n), target] => Ok((n, value_to_target(target, error)?)),
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

/// Tries to parse the arguments into any target.
/// Returns a [ParseInstructionsError] if there is a wrong number of arguments
/// or they have wrong types.
fn target(args: Vec<Value>, name: &'static str) -> Result<Vec<String>, ParseInstructionsError> {
    let error = || ParseInstructionsError::WrongTypeOfArgument {
        name,
        expected: &[&["target"]],
    };

    let [target] = n_args(args, name, &[1])?;
    value_to_target(target, error)
}

/// Tries to convert a [Value] to a target (list of IDs).
/// Will return the `error` if the [Value] contains anything
/// apart from [Set][Value::Set]s and [Identifier][Value::Identifier]s.
fn value_to_target(
    target: Value,
    error: impl Fn() -> ParseInstructionsError,
) -> Result<Vec<String>, ParseInstructionsError> {
    target
        .flatten_sets()
        .map(|v| {
            if let Value::Identifier(id) = v {
                Ok(id)
            } else {
                Err(error())
            }
        })
        .collect()
}

/// Returns a slice of length `N` if the passed vector has length `N`,
/// or an error containing the original vector.
///
/// Typed wrapper around [Vec::try_into][TryInto<slice<T, N>>::try_into].
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
                        InstructionGroup {
                            variable: false,
                            instructions: vec![
                                TimedInstruction::Load {
                                    position: None,
                                    id: "atom0".to_string(),
                                },
                                TimedInstruction::Load {
                                    position: Some((
                                        Fraction::new(16u64, 1u64),
                                        Fraction::new(2u64, 1u64),
                                    )),
                                    id: "atom1".to_string(),
                                },
                            ],
                        },
                    ),
                    (
                        false,
                        Fraction::new(0u64, 1u64),
                        InstructionGroup {
                            variable: false,
                            instructions: vec![TimedInstruction::Move {
                                position: (Fraction::new(8u64, 1u64), Fraction::new(8u64, 1u64)),
                                id: "atom0".to_string(),
                            }],
                        },
                    ),
                    (
                        true,
                        Fraction::new(0u64, 1u64),
                        InstructionGroup {
                            variable: false,
                            instructions: vec![TimedInstruction::Move {
                                position: (Fraction::new(16u64, 1u64), Fraction::new(16u64, 1u64)),
                                id: "atom1".to_string(),
                            }],
                        },
                    ),
                    (
                        false,
                        Fraction::new(0u64, 1u64),
                        InstructionGroup {
                            variable: false,
                            instructions: vec![TimedInstruction::Store {
                                position: None,
                                id: "atom0".to_string(),
                            }],
                        },
                    ),
                    (
                        true,
                        Fraction::new(0u64, 1u64),
                        InstructionGroup {
                            variable: false,
                            instructions: vec![TimedInstruction::Store {
                                position: None,
                                id: "atom1".to_string(),
                            }],
                        },
                    ),
                    (
                        false,
                        Fraction::new(0u64, 1u64),
                        InstructionGroup {
                            variable: false,
                            instructions: vec![TimedInstruction::Rz {
                                value: Fraction::new(3141u64, 1000u64),
                                targets: vec!["atom0".to_string()],
                            }],
                        },
                    ),
                    (
                        false,
                        Fraction::new(0u64, 1u64),
                        InstructionGroup {
                            variable: false,
                            instructions: vec![TimedInstruction::Ry {
                                value: Fraction::new(3141u64, 1000u64),
                                targets: vec!["atom1".to_string()],
                            }],
                        },
                    ),
                    (
                        false,
                        Fraction::new(0u64, 1u64),
                        InstructionGroup {
                            variable: false,
                            instructions: vec![TimedInstruction::Cz {
                                targets: vec!["zone0".to_string()],
                            }],
                        },
                    ),
                    (
                        false,
                        Fraction::new(0u64, 1u64),
                        InstructionGroup {
                            variable: true,
                            instructions: vec![
                                TimedInstruction::Cz {
                                    targets: vec!["zone1".to_string()],
                                },
                                TimedInstruction::Ry {
                                    value: Fraction::new(3141u64, 1000u64),
                                    targets: vec!["atom0".to_string()],
                                },
                            ],
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
            InstructionOrDirective::GroupedTime {
                time: Some((TimeSpec::Absolute, Fraction::new(20u64, 1u64))),
                variable: false,
                group: vec![
                    (
                        "load".to_string(),
                        vec![Value::Identifier("atom0".to_string())],
                    ),
                    (
                        "load".to_string(),
                        vec![Value::Identifier("atom1".to_string())],
                    ),
                ],
            },
            InstructionOrDirective::GroupedInstruction {
                time: Some((
                    TimeSpec::Relative {
                        from_start: false,
                        positive: true,
                    },
                    Fraction::new(0u64, 1u64),
                )),
                variable: true,
                name: "store".to_string(),
                group: vec![
                    vec![Value::Identifier("atom0".to_string())],
                    vec![Value::Identifier("atom1".to_string())],
                ],
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
                            InstructionGroup {
                                variable: false,
                                instructions: vec![TimedInstruction::Load {
                                    position: None,
                                    id: "atom1".to_string(),
                                }],
                            },
                        ),
                        (
                            true,
                            Fraction::new(2u64, 1u64),
                            InstructionGroup {
                                variable: false,
                                instructions: vec![TimedInstruction::Store {
                                    position: None,
                                    id: "atom1".to_string(),
                                }],
                            },
                        ),
                        (
                            false,
                            Fraction::new(3u64, 1u64),
                            InstructionGroup {
                                variable: false,
                                instructions: vec![TimedInstruction::Load {
                                    position: None,
                                    id: "atom1".to_string(),
                                }],
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
                            InstructionGroup {
                                variable: false,
                                instructions: vec![TimedInstruction::Store {
                                    position: None,
                                    id: "atom1".to_string(),
                                }],
                            },
                        ),
                        (
                            true,
                            Fraction::new(2u64, 1u64),
                            InstructionGroup {
                                variable: false,
                                instructions: vec![TimedInstruction::Load {
                                    position: None,
                                    id: "atom1".to_string(),
                                }],
                            },
                        ),
                        (
                            false,
                            Fraction::new(0u64, 1u64),
                            InstructionGroup {
                                variable: false,
                                instructions: vec![TimedInstruction::Store {
                                    position: None,
                                    id: "atom1".to_string(),
                                }],
                            },
                        ),
                    ],
                ),
                (
                    Fraction::new(20u64, 1u64),
                    vec![
                        (
                            true,
                            Fraction::new(0u64, 1u64),
                            InstructionGroup {
                                variable: false,
                                instructions: vec![
                                    TimedInstruction::Load {
                                        position: None,
                                        id: "atom0".to_string(),
                                    },
                                    TimedInstruction::Load {
                                        position: None,
                                        id: "atom1".to_string(),
                                    },
                                ],
                            },
                        ),
                        (
                            false,
                            Fraction::new(0u64, 1u64),
                            InstructionGroup {
                                variable: true,
                                instructions: vec![
                                    TimedInstruction::Store {
                                        position: None,
                                        id: "atom0".to_string(),
                                    },
                                    TimedInstruction::Store {
                                        position: None,
                                        id: "atom1".to_string(),
                                    },
                                ],
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
