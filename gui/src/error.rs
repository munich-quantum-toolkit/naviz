use std::str::Utf8Error;

use naviz_import::ImportError;
use naviz_parser::{config, input::concrete::ParseInstructionsError, ParseErrorInner};

/// A [Result][std::result::Result] pre-filled with [Error]
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// An error that can occur in the gui
#[derive(Debug)]
pub enum Error {
    /// Error while opening a file.
    /// For errors when importing, see [Error::Import].
    FileOpen(InputType),
    /// Error while importing from another format.
    Import(ImportError),
    /// Error when interacting with the repository.
    Repository(RepositoryError, ConfigFormat),
}

/// An Error occurred while opening one of the input-types
#[derive(Debug)]
pub enum InputType {
    Instruction(InputError),
    Config(ConfigFormat, ConfigError),
}

/// An error to do with the instruction input
#[derive(Debug)]
pub enum InputError {
    UTF8(Utf8Error),
    Lex(ParseErrorInner, Option<ErrorLocation>),
    Parse(ParseErrorInner, Option<ErrorLocation>),
    Convert(ParseInstructionsError),
}

/// A config-format
#[derive(Debug)]
pub enum ConfigFormat {
    Machine,
    Style,
}

/// An error to do with the config files
#[derive(Debug)]
pub enum ConfigError {
    UTF8(Utf8Error),
    Lex(ParseErrorInner, Option<ErrorLocation>),
    Parse(ParseErrorInner, Option<ErrorLocation>),
    Convert(config::error::Error),
}
/// An error to do with the [Repository][naviz_repository::Repository]
#[derive(Debug)]
pub enum RepositoryError {
    Load(RepositoryLoadSource, naviz_repository::error::Error),
    Open(naviz_repository::error::Error),
    Import(naviz_repository::error::Error),
    Remove(naviz_repository::error::Error),
    Search,
}

/// The source where the [Repository][naviz_repository::Repository] was loaded from
#[derive(Debug)]
pub enum RepositoryLoadSource {
    Bundled,
    UserDir,
}

impl Error {
    /// Short message of this [Error].
    /// Can be displayed in the title-bar.
    #[rustfmt::skip]
    pub fn title(&self) -> &'static str {
        match self {
            Self::FileOpen(InputType::Config(ConfigFormat::Machine, _)) => "Invalid machine definition",
            Self::FileOpen(InputType::Config(ConfigFormat::Style, _)) => "Invalid style definition",
            Self::FileOpen(InputType::Instruction(_)) => "Invalid instructions",
            Self::Import(_) => "Failed to import",
            Self::Repository(RepositoryError::Search, ConfigFormat::Machine) => "Machine not found",
            Self::Repository(RepositoryError::Search, ConfigFormat::Style) => "Style not found",
            Self::Repository(RepositoryError::Open(_), ConfigFormat::Machine) => "Invalid machine in repository",
            Self::Repository(RepositoryError::Open(_), ConfigFormat::Style) => "Invalid style in repository",
            Self::Repository(RepositoryError::Load(RepositoryLoadSource::Bundled, _), ConfigFormat::Machine) => "Failed to load bundled machines",
            Self::Repository(RepositoryError::Load(RepositoryLoadSource::Bundled, _), ConfigFormat::Style) => "Failed to load bundled styles",
            Self::Repository(RepositoryError::Load(RepositoryLoadSource::UserDir, _), ConfigFormat::Machine) => "Failed to load machines from user-dir",
            Self::Repository(RepositoryError::Load(RepositoryLoadSource::UserDir, _), ConfigFormat::Style) => "Failed to load styles from user-dir",
            Self::Repository(RepositoryError::Import(_), ConfigFormat::Machine) => "Failed to import machine to user-dir",
            Self::Repository(RepositoryError::Import(_), ConfigFormat::Style) => "Failed to import machine to user-dir",
            Self::Repository(RepositoryError::Remove(_), ConfigFormat::Machine) => "Failed to remove machine from user-dir",
            Self::Repository(RepositoryError::Remove(_), ConfigFormat::Style) => "Failed to remove style from user-dir",
        }
    }

    /// A longer text representation of this [Error].
    /// Contains details and location information when available.
    pub fn body(&self) -> String {
        match self {
            Self::FileOpen(InputType::Config(format, config_error)) => {
                format_config_error(format, config_error)
            }
            Self::FileOpen(InputType::Instruction(input_error)) => {
                format_input_error(input_error)
            }
            Self::Import(import_error) => {
                format_import_error(import_error)
            }
            Self::Repository(repo_error, config_format) => {
                format_repository_error(repo_error, config_format)
            }
        }
    }
}

/// Format a config error with location information
fn format_config_error(format: &ConfigFormat, error: &ConfigError) -> String {
    let file_type = match format {
        ConfigFormat::Machine => "machine configuration",
        ConfigFormat::Style => "style configuration",
    };

    match error {
        ConfigError::UTF8(utf8_error) => {
            format!(
                "The {} file contains invalid UTF-8 encoding.\n\n\
                Error details: {}\n\n\
                Please ensure the file is saved with proper UTF-8 encoding.",
                file_type, utf8_error
            )
        }
        ConfigError::Lex(parse_error, location) => {
            let location_info = location
                .as_ref()
                .map(|loc| format!(" at line {}, column {}", loc.line, loc.column))
                .unwrap_or_default();
            format!(
                "Failed to parse {} file due to invalid syntax{}.\n\n\
                Lexical analysis error: {}\n\n\
                This typically means:\n\
                • Unexpected characters or symbols\n\
                • Invalid token sequences\n\
                • Malformed identifiers or literals\n\n\
                Please check the file syntax and ensure it follows the expected format.",
                file_type, location_info, format_parse_error_context(parse_error)
            )
        }
        ConfigError::Parse(parse_error, location) => {
            let location_info = location
                .as_ref()
                .map(|loc| format!(" at line {}, column {}", loc.line, loc.column))
                .unwrap_or_default();
            format!(
                "Failed to parse {} file structure{}.\n\n\
                Parsing error: {}\n\n\
                This typically means:\n\
                • Brackets, braces, or parentheses are not properly matched\n\
                • Missing or extra punctuation\n\
                • Field names are misspelled or invalid\n\
                • Values are in an unexpected format\n\n\
                Please verify the file structure and syntax.",
                file_type, location_info, format_parse_error_context(parse_error)
            )
        }
        ConfigError::Convert(config_error) => {
            format!(
                "Failed to process {} file content.\n\n\
                {}\n\n\
                The file syntax is correct, but the content validation failed.\n\
                Please check that all required fields are present and have valid values.",
                file_type, config_error
            )
        }
    }
}

/// Format an input error with location information
fn format_input_error(error: &InputError) -> String {
    match error {
        InputError::UTF8(utf8_error) => {
            format!(
                "The instruction file contains invalid UTF-8 encoding.\n\n\
                Error details: {}\n\n\
                Please ensure the file is saved with proper UTF-8 encoding.",
                utf8_error
            )
        }
        InputError::Lex(parse_error, location) => {
            let location_info = location
                .as_ref()
                .map(|loc| format!(" at line {}, column {}", loc.line, loc.column))
                .unwrap_or_default();
            format!(
                "Failed to parse instruction file due to invalid syntax{}.\n\n\
                Lexical analysis error: {}\n\n\
                Common issues:\n\
                • Unrecognized characters or symbols\n\
                • Incomplete string literals or comments\n\
                • Invalid number formats\n\
                • Malformed identifiers\n\n\
                Please check the file syntax.",
                location_info, format_parse_error_context(parse_error)
            )
        }
        InputError::Parse(parse_error, location) => {
            let location_info = location
                .as_ref()
                .map(|loc| format!(" at line {}, column {}", loc.line, loc.column))
                .unwrap_or_default();
            format!(
                "Failed to parse instruction file structure{}.\n\n\
                Parsing error: {}\n\n\
                Common issues:\n\
                • Instructions are not properly formatted\n\
                • Parentheses, brackets, or braces are unbalanced\n\
                • Missing semicolons or separators\n\
                • Invalid gate names or parameters\n\n\
                Please verify the instruction syntax.",
                location_info, format_parse_error_context(parse_error)
            )
        }
        InputError::Convert(convert_error) => {
            format!(
                "Failed to process instruction file content.\n\n\
                Conversion error: {:?}\n\n\
                The file syntax is correct, but the instructions could not be processed.\n\
                This usually indicates:\n\
                • Invalid gate parameters or arguments\n\
                • Undefined or inconsistent quantum registers\n\
                • Unsupported instruction types\n\
                • Qubit index out of range",
                convert_error
            )
        }
    }
}

/// Format parse error context information in a user-friendly way
fn format_parse_error_context(error: &ParseErrorInner) -> String {
    // Extract context information from winnow's ContextError
    let context_info: Vec<String> = error.context().map(|ctx| ctx.to_string()).collect();

    if !context_info.is_empty() {
        let contexts = context_info.join(", ");
        format!("Expected: {}", contexts)
    } else {
        // If no context is available, provide more helpful generic information
        format!("Syntax error encountered while parsing")
    }
}

/// Format an import error with helpful context
fn format_import_error(error: &ImportError) -> String {
    match error {
        ImportError::InvalidUtf8(utf8_error) => {
            format!(
                "The imported file contains invalid UTF-8 encoding.\n\n\
                Error details: {}\n\n\
                Please ensure the file is saved with proper UTF-8 encoding.",
                utf8_error
            )
        }
        ImportError::MqtNqParse(parse_error) => {
            format!(
                "Failed to parse the MQT-NA format file.\n\n\
                Parsing error: {}\n\n\
                This typically means:\n\
                • Invalid MQT-NA syntax or format\n\
                • Unrecognized quantum operation names\n\
                • Malformed operation parameters or arguments\n\
                • Missing or incorrect punctuation (semicolons, parentheses)\n\
                • Invalid qubit register declarations\n\n\
                Please verify that:\n\
                • The file follows the MQT-NA format specification\n\
                • All quantum operations are properly formatted\n\
                • Register declarations are valid",
                format_parse_error_context(parse_error)
            )
        }
        ImportError::MqtNqConvert(convert_error) => {
            format!(
                "Failed to convert MQT-NA operations.\n\n\
                Conversion error: {:?}\n\n\
                The file was parsed successfully, but some operations could not be converted.\n\
                This may indicate:\n\
                • Unsupported gate types or operations\n\
                • Invalid operation parameters or qubit indices\n\
                • Incompatible quantum circuit structure\n\
                • Operations not supported by the target format",
                convert_error
            )
        }
    }
}

/// Format a repository error with context
fn format_repository_error(error: &RepositoryError, config_format: &ConfigFormat) -> String {
    let item_type = match config_format {
        ConfigFormat::Machine => "machine",
        ConfigFormat::Style => "style",
    };

    match error {
        RepositoryError::Load(source, repo_error) => {
            let source_desc = match source {
                RepositoryLoadSource::Bundled => "bundled",
                RepositoryLoadSource::UserDir => "user directory",
            };
            format!(
                "Failed to load {} definitions from {}.\n\n\
                Error: {:?}\n\n\
                This may indicate:\n\
                • Corrupted files in the {} repository\n\
                • Missing or inaccessible files\n\
                • Permission issues",
                item_type, source_desc, repo_error, source_desc
            )
        }
        RepositoryError::Open(repo_error) => {
            format!(
                "Failed to open {} from repository.\n\n\
                Error: {:?}\n\n\
                The {} file may be corrupted or in an incompatible format.",
                item_type, repo_error, item_type
            )
        }
        RepositoryError::Import(repo_error) => {
            format!(
                "Failed to import {} to user directory.\n\n\
                Error: {:?}\n\n\
                This may be due to:\n\
                • Insufficient permissions to write to user directory\n\
                • Disk space limitations\n\
                • File system errors",
                item_type, repo_error
            )
        }
        RepositoryError::Remove(repo_error) => {
            format!(
                "Failed to remove {} from user directory.\n\n\
                Error: {:?}\n\n\
                This may be due to:\n\
                • Insufficient permissions\n\
                • File is currently in use\n\
                • File system errors",
                item_type, repo_error
            )
        }
        RepositoryError::Search => {
            format!(
                "Could not find the requested {} in the repository.\n\n\
                Please verify:\n\
                • The {} name is spelled correctly\n\
                • The {} exists in the repository\n\
                • The repository was loaded successfully",
                item_type, item_type, item_type
            )
        }
    }
}

/// Location information for parsing errors
#[derive(Debug, Clone)]
pub struct ErrorLocation {
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
    /// Byte offset in the original text
    pub offset: usize,
}

impl ErrorLocation {
    /// Create ErrorLocation from byte offset and original text
    pub fn from_offset(text: &str, offset: usize) -> Self {
        let (line, column) = byte_offset_to_line_column(text, offset);
        Self { line, column, offset }
    }
}

/// Convert byte offset to line and column numbers (1-based)
fn byte_offset_to_line_column(text: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut column = 1;

    for (i, ch) in text.char_indices() {
        if i >= offset {
            break;
        }

        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }

    (line, column)
}
