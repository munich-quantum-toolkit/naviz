//! Structs that allow setting some options to initialize the app
//! with some predefined values.

use naviz_import::ImportOptions;

/// Something that is either specified by an `id` from the [Repository]
/// or manually loaded.
pub enum IdOrManual<ID, MAN> {
    Id(ID),
    Manual(MAN),
}

type IdOrManualInit<'a> = IdOrManual<&'a str, &'a [u8]>;

/// Options to start the app with.
/// Leave [None] to keep unset or set to [Some] value to initialize with the value.
/// Can be passed to [App::new_with_init][crate::App::new_with_init].
#[derive(Default)]
pub struct InitOptions<'a> {
    /// The machine to load
    pub machine: Option<IdOrManualInit<'a>>,
    /// The style to load
    pub style: Option<IdOrManualInit<'a>>,
    /// The visualization input to load.
    /// Pass [Some] [ImportOptions] if the content needs to be imported.
    pub input: Option<(Option<ImportOptions>, &'a [u8])>,
}
