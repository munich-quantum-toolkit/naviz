use std::fmt::Display;

use itertools::Itertools;

/// An [ErrorKind] for [Error].
/// Converts to [Error] using [Into::into].
///
/// Used for config-parsing.
#[derive(Debug)]
pub enum ErrorKind {
    MissingField(&'static str),
    WrongType(&'static str),
}

impl Display for ErrorKind {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingField(f) => write!(fmt, "missing field: {f}"),
            Self::WrongType(t) => write!(fmt, "wrong type: expected {t}"),
        }
    }
}

/// An error, which has a [ErrorKind] and collects the tagged path.
/// Not constructed directly, but though [ErrorKind::into][Into<Error>::into].
///
/// See [TagError] to tag errors.
#[derive(Debug)]
pub struct Error {
    /// The type of error
    kind: ErrorKind,
    /// A path of tags
    /// (usually in reverse order as closest tag to error location is tagged first)
    path: Vec<&'static str>,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.kind)?;
        Itertools::intersperse(self.path.iter().rev(), &" -> ").try_for_each(|e| write!(f, "{e}"))
    }
}

impl Error {
    /// The type of error
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    /// A path of tags.
    /// Usually in reverse order as closest tag to error location is tagged first.
    ///
    /// See [TagError] to tag errors.
    pub fn path(&self) -> &[&'static str] {
        self.path.as_slice()
    }
}

impl From<ErrorKind> for Error {
    fn from(value: ErrorKind) -> Self {
        Self {
            kind: value,
            path: Vec::new(),
        }
    }
}

/// A trait that allows tagging an [Error] using [TagError::tag].
pub trait TagError {
    fn tag(self, tag: &'static str) -> Self;
}

impl<T> TagError for Result<T, Error> {
    fn tag(mut self, tag: &'static str) -> Self {
        if let Err(e) = &mut self {
            e.path.push(tag);
        }
        self
    }
}

impl TagError for Error {
    fn tag(mut self, tag: &'static str) -> Self {
        self.path.push(tag);
        self
    }
}
