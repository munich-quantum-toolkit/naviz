use super::{
    error::{Error, ErrorKind},
    generic::ConfigItem,
    parser::Value,
};
use fraction::Fraction;

/// A newtype representing a percentage
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
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
