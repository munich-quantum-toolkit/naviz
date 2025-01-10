//! Import definitions/handling for the naviz-gui

use std::str::Utf8Error;

use egui::{TextEdit, Ui};
use naviz_import::mqt;
use naviz_parser::input::concrete::Instructions;

use crate::menu::FileFilter;

/// The available import formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImportFormat {
    /// [mqt::na]
    MqtNa,
}

/// All import-formats that are to be shown in the GUI
pub static IMPORT_FORMATS: [ImportFormat; 1] = [ImportFormat::MqtNa];

/// The options for the different import formats
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

impl FileFilter for ImportFormat {
    fn name(&self) -> &'static str {
        match self {
            Self::MqtNa => "mqt na",
        }
    }

    fn extensions(&self) -> &'static [&'static str] {
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
    /// Draws a settings-ui for these [ImportOptions].
    /// Edits from the ui will be reflected inside `self`.
    pub fn draw(&mut self, ui: &mut Ui) {
        match self {
            Self::MqtNa(options) => {
                ui.horizontal(|ui| {
                    ui.label("Atom prefix");
                    ui.add(
                        TextEdit::singleline(&mut options.atom_prefix).desired_width(f32::INFINITY),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("CZ global zone");
                    ui.add(
                        TextEdit::singleline(&mut options.global_zones.cz)
                            .desired_width(f32::INFINITY),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("RY global zone");
                    ui.add(
                        TextEdit::singleline(&mut options.global_zones.ry)
                            .desired_width(f32::INFINITY),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("RZ global zone");
                    ui.add(
                        TextEdit::singleline(&mut options.global_zones.rz)
                            .desired_width(f32::INFINITY),
                    );
                });
            }
        }
    }

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
