use fraction::Fraction;

/// A newtype representing a percentage
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Percentage(pub Fraction);
