//! Structs that allow setting some options to initialize the app
//! with some predefined values.

use naviz_import::ImportOptions;

/// Options to start the app with.
/// Leave [None] to keep unset or set to [Some] value to initialize with the value.
/// Can be passed to [App::new_with_init][crate::App::new_with_init].
#[derive(Default)]
pub struct InitOptions<'a> {
    /// The machine-id to load
    pub machine: Option<&'a str>,
    /// The style-id to load
    pub style: Option<&'a str>,
    /// The visualization input to load.
    /// Pass [Some] [ImportOptions] if the content needs to be imported.
    pub input: Option<(Option<ImportOptions>, &'a [u8])>,
}
