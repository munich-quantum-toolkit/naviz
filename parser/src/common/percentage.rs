use super::parser::Value;
use crate::config::{
    error::{Error, ErrorKind},
    generic::ConfigItem,
};
use fraction::Fraction;

/// A newtype representing a percentage
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Percentage(pub Fraction);

impl TryFrom<ConfigItem> for Percentage {
    type Error = Error;
    fn try_from(value: ConfigItem) -> Result<Self, Self::Error> {
        match value {
            ConfigItem::Value(Value::Percentage(p)) => Ok(p),
            _ => Err(ErrorKind::WrongType("percentage").into()),
        }
    }
}

impl From<Percentage> for Fraction {
    fn from(value: Percentage) -> Self {
        value.0 / 100
    }
}
