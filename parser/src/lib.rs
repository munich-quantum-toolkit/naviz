pub mod common;
pub mod config;
pub mod input;

#[cfg(test)]
pub mod test_utils;

/// Error returned when parsing/lexing.
/// Contains Reference to the input.
pub type ParseError<I> = winnow::error::ParseError<I, ParseErrorInner>;

/// Inner error in [ParseError].
pub type ParseErrorInner = winnow::error::ContextError;
