use std::str::Utf8Error;

use naviz_parser::input::concrete::Instructions;

pub mod mqt;
pub mod separated_display;

/// The available import formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ImportFormat {
    /// [mqt::na]
    MqtNa,
}

/// List of all import-formats (all entries of [ImportFormat]).
pub static IMPORT_FORMATS: [ImportFormat; 1] = [ImportFormat::MqtNa];

/// The options for the different import formats
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ImportOptions {
    /// [mqt::na]
    MqtNa(mqt::na::convert::ConvertOptions<'static>),
}

/// An error that can occur during import
#[derive(Debug, Clone, PartialEq)]
pub enum ImportError {
    /// Something was not valid UTF-8
    InvalidUtf8(Utf8Error),
    /// An error occurred while parsing [mqt::na]
    MqtNqParse(mqt::na::format::ParseErrorInner),
    /// An error occurred while converting [mqt::na]
    MqtNqConvert(mqt::na::convert::OperationConversionError),
}

impl ImportFormat {
    /// A human-readable name of this [ImportFormat]
    pub fn name(&self) -> &'static str {
        match self {
            Self::MqtNa => "mqt na",
        }
    }

    /// A list of file-extensions commonly used by this [ImportFormat]
    pub fn file_extensions(&self) -> &'static [&'static str] {
        match self {
            Self::MqtNa => &["na"],
        }
    }
}

impl From<ImportFormat> for ImportOptions {
    fn from(value: ImportFormat) -> Self {
        match value {
            ImportFormat::MqtNa => ImportOptions::MqtNa(Default::default()),
        }
    }
}

impl From<&ImportOptions> for ImportFormat {
    fn from(value: &ImportOptions) -> Self {
        match value {
            &ImportOptions::MqtNa(_) => ImportFormat::MqtNa,
        }
    }
}

impl ImportOptions {
    /// Imports the `data` using the options in `self`
    pub fn import(self, data: &[u8]) -> Result<Instructions, ImportError> {
        match self {
            Self::MqtNa(options) => mqt::na::convert::convert(
                &mqt::na::format::parse(
                    std::str::from_utf8(data).map_err(ImportError::InvalidUtf8)?,
                )
                .map_err(|e| e.into_inner())
                .map_err(ImportError::MqtNqParse)?,
                options,
            )
            .map_err(ImportError::MqtNqConvert),
        }
    }
}
