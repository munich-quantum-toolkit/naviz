use super::Value;
use crate::common::{
    color::{Color, ParseColorError},
    lexer,
    percentage::Percentage,
};
use fraction::Fraction;
use regex::Regex;
use std::{convert::Infallible, error::Error, fmt::Display, str::ParseBoolError};

/// An error holding the possible errors of a [TryIntoValue].
#[derive(Debug)]
pub enum TryIntoValueError<
    StringError: Error,
    RegexError: Error,
    NumberError: Error,
    PercentageError: Error,
    BooleanError: Error,
    ColorError: Error,
    IdentifierError: Error,
> {
    StringError(StringError),
    RegexError(RegexError),
    NumberError(NumberError),
    PercentageError(PercentageError),
    BooleanError(BooleanError),
    ColorError(ColorError),
    IdentifierError(IdentifierError),
}

impl<
        StringError: Error,
        RegexError: Error,
        NumberError: Error,
        PercentageError: Error,
        BooleanError: Error,
        ColorError: Error,
        IdentifierError: Error,
    > Display
    for TryIntoValueError<
        StringError,
        RegexError,
        NumberError,
        PercentageError,
        BooleanError,
        ColorError,
        IdentifierError,
    >
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[allow(deprecated)]
        self.description().fmt(f)
    }
}

impl<
        StringError: Error,
        RegexError: Error,
        NumberError: Error,
        PercentageError: Error,
        BooleanError: Error,
        ColorError: Error,
        IdentifierError: Error,
    > Error
    for TryIntoValueError<
        StringError,
        RegexError,
        NumberError,
        PercentageError,
        BooleanError,
        ColorError,
        IdentifierError,
    >
{
    #[allow(deprecated)]
    fn description(&self) -> &str {
        match self {
            Self::StringError(e) => e.description(),
            Self::RegexError(e) => e.description(),
            Self::NumberError(e) => e.description(),
            Self::PercentageError(e) => e.description(),
            Self::BooleanError(e) => e.description(),
            Self::ColorError(e) => e.description(),
            Self::IdentifierError(e) => e.description(),
        }
    }
}

/// Try to parse the current object into the contents of a [Value]
pub trait TryIntoValue {
    /// Error when parsing into [String].
    type StringError: Error + Send + Sync + 'static;
    /// Error when parsing into [Regex].
    type RegexError: Error + Send + Sync + 'static;
    /// Error when parsing into [Fraction].
    type NumberError: Error + Send + Sync + 'static;
    /// Error when parsing into [Percentage].
    type PercentageError: Error + Send + Sync + 'static;
    /// Error when parsing into [bool].
    type BooleanError: Error + Send + Sync + 'static;
    /// Error when parsing into [Color].
    type ColorError: Error + Send + Sync + 'static;
    /// Error when parsing into identifier.
    type IdentifierError: Error + Send + Sync + 'static;

    /// Try to parse `self` into a [String] for [Value::String].
    fn string(self) -> Result<String, Self::StringError>;
    /// Try to parse `self` into a [Regex] for [Value::Regex].
    fn regex(self) -> Result<Regex, Self::RegexError>;
    /// Try to parse `self` into a [Fraction] for [Value::Number].
    fn number(self) -> Result<Fraction, Self::NumberError>;
    /// Try to parse `self` into a [Percentage] for [Value::Number].
    fn percentage(self) -> Result<Percentage, Self::PercentageError>;
    /// Try to parse `self` into a [bool] for [Value::Boolean].
    fn boolean(self) -> Result<bool, Self::BooleanError>;
    /// Try to parse `self` into a [Color] for [Value::Color].
    fn color(self) -> Result<Color, Self::ColorError>;
    /// Try to parse `self` into a [String] for [Value::Identifier].
    fn identifier(self) -> Result<String, Self::IdentifierError>;
}

/// Allow converting a [`lexer::Value<T>`] into a [Value] when `T` implements [TryIntoValue].
impl<T: TryIntoValue> TryFrom<lexer::Value<T>> for Value {
    type Error = TryIntoValueError<
        T::StringError,
        T::RegexError,
        T::NumberError,
        T::PercentageError,
        T::BooleanError,
        T::ColorError,
        T::IdentifierError,
    >;

    fn try_from(value: lexer::Value<T>) -> Result<Self, Self::Error> {
        Ok(match value {
            lexer::Value::String(s) => {
                Self::String(s.string().map_err(TryIntoValueError::StringError)?)
            }
            lexer::Value::Regex(r) => {
                Self::Regex(r.regex().map_err(TryIntoValueError::RegexError)?)
            }
            lexer::Value::Number(n) => {
                Self::Number(n.number().map_err(TryIntoValueError::NumberError)?)
            }
            lexer::Value::Percentage(n) => {
                Self::Percentage(n.percentage().map_err(TryIntoValueError::PercentageError)?)
            }
            lexer::Value::Boolean(b) => {
                Self::Boolean(b.boolean().map_err(TryIntoValueError::BooleanError)?)
            }
            lexer::Value::Color(c) => {
                Self::Color(c.color().map_err(TryIntoValueError::ColorError)?)
            }
        })
    }
}

/// [TryIntoValue] for [&str][str]s.
impl TryIntoValue for &str {
    type StringError = Infallible;
    type RegexError = regex::Error;
    type NumberError = fraction::error::ParseError;
    type PercentageError = fraction::error::ParseError;
    type BooleanError = ParseBoolError;
    type ColorError = ParseColorError;
    type IdentifierError = Infallible;

    fn string(self) -> Result<String, Self::StringError> {
        Ok(self.to_string())
    }
    fn regex(self) -> Result<Regex, Self::RegexError> {
        self.parse()
    }
    fn number(self) -> Result<Fraction, Self::NumberError> {
        self.parse()
    }
    fn percentage(self) -> Result<Percentage, Self::PercentageError> {
        self.parse().map(Percentage)
    }
    fn boolean(self) -> Result<bool, Self::BooleanError> {
        self.parse()
    }
    fn color(self) -> Result<Color, Self::ColorError> {
        self.parse()
    }
    fn identifier(self) -> Result<String, Self::IdentifierError> {
        Ok(self.to_string())
    }
}
