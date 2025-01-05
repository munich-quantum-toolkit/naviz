//! [Repository] for loading configs.
//! Also contains bundled configs.

use std::{
    borrow::{Borrow, Cow},
    collections::HashSet,
    fs,
    hash::Hash,
    path::PathBuf,
};

use directories::ProjectDirs;
use error::{Error, Result};
use include_dir::{include_dir, Dir};
use naviz_parser::config::generic::Config;

pub mod error;

static BUNDLED_MACHINES: Dir = include_dir!("$CARGO_MANIFEST_DIR/../configs/machines");
static BUNDLED_STYLES: Dir = include_dir!("$CARGO_MANIFEST_DIR/../configs/styles");

/// A repository of config files.
pub struct Repository(HashSet<RepositoryEntry>);

/// The project directories for this application
fn project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("tv", "floeze", "naviz").ok_or(Error::IoError(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Failed to get user directory",
    )))
}

impl Repository {
    /// Creates a new empty repository
    pub fn empty() -> Self {
        Self(Default::default())
    }

    /// Loads the passed bundled config into the passed [Repository]
    fn load_bundled(mut self, bundled: &Dir<'static>) -> Result<Self> {
        self.0 = insert_results(
            self.0,
            bundled.files().map(|f| {
                RepositoryEntry::new(
                    f.path()
                        .file_name()
                        .ok_or(Error::IdError)?
                        .to_string_lossy()
                        .into_owned(),
                    RepositorySource::Bundled(f.contents()),
                )
            }),
        )?;
        Ok(self)
    }

    /// Loads the bundled machines into the passed [Repository]
    pub fn bundled_machines(self) -> Result<Self> {
        self.load_bundled(&BUNDLED_MACHINES)
    }

    /// Loads the bundles styles into the passed [Repository]
    pub fn bundled_styles(self) -> Result<Self> {
        self.load_bundled(&BUNDLED_STYLES)
    }

    /// Loads the configs from the passed `subdir` of the user-directory
    /// into the passed [Repository]
    fn load_user_dir(mut self, subdir: &str) -> Result<Self> {
        let directory = project_dirs()?.data_dir().join(subdir);

        if !directory.exists() {
            fs::create_dir_all(&directory).map_err(Error::IoError)?;
        }

        self.0 = insert_results(
            self.0,
            directory
                .read_dir()
                .map_err(Error::IoError)?
                .filter_map(|x| {
                    if let Ok(x) = x {
                        if x.path().is_file() {
                            Some(x.path())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .map(|p| {
                    RepositoryEntry::new(
                        p.file_stem()
                            .ok_or(Error::IdError)?
                            .to_string_lossy()
                            .into_owned(),
                        RepositorySource::UserDir(p),
                    )
                }),
        )?;
        Ok(self)
    }

    /// Loads the machines from the user-directory into the passed [Repository]
    pub fn user_dir_machines(self) -> Result<Self> {
        self.load_user_dir("machines")
    }

    /// Loads the styles from the user-directory into the passed [Repository]
    pub fn user_dir_styles(self) -> Result<Self> {
        self.load_user_dir("styles")
    }

    /// The list of entries of this repository: `(id, name)`-pairs
    pub fn list(&self) -> Vec<(&str, &str)> {
        self.0.iter().map(|i| (i.id(), i.name())).collect()
    }

    /// Tries to get the raw contents of the entry with the passed `id`.
    ///
    /// Returns:
    /// - `None`: No entry with the passed `id` exists
    /// - `Some(Err)`: An entry exists, but failed to load the data
    /// - `Some(Ok)`: The data of the found entry
    pub fn get_raw(&self, id: &str) -> Option<Result<Cow<[u8]>>> {
        self.0.get(id).map(|e| e.contents())
    }

    /// Tries to get the contents of the entry with the passed `id` as some [Config].
    ///
    /// Returns:
    /// - `None`: No entry with the passed `id` exists
    /// - `Some(Err)`: An entry exists, but failed to load the data or failed to convert to `C`
    /// - `Some(Ok)`: The config of the found entry
    pub fn get<C>(&self, id: &str) -> Option<Result<C>>
    where
        Config: TryInto<C, Error = naviz_parser::config::error::Error>,
    {
        self.0.get(id).map(|e| {
            e.contents_as_config()?
                .try_into()
                .map_err(Error::ConfigReadError)
        })
    }

    /// Try to get any config from this repository
    pub fn try_get_any<C>(&self) -> Option<(&str, C)>
    where
        Config: TryInto<C>,
    {
        self.0
            .iter()
            .filter_map(|e| Some((e.id(), e.contents_as_config().ok()?.try_into().ok()?)))
            .next()
    }
}

/// An entry in the repository.
/// Contains a cached `name`, an `id`, and the `source`.
/// Is hashed and checked for equality only by `id`.
struct RepositoryEntry {
    /// The name as read from the config-file
    name: String,
    /// The id of the entry
    id: String,
    /// The source of the entry
    source: RepositorySource,
}

impl RepositoryEntry {
    /// Creates a new [RepositoryEntry] with the given `id` from the passed `source`.
    /// Will extract the name from the `source`.
    fn new(id: String, source: RepositorySource) -> Result<Self> {
        Ok(Self {
            name: source.name()?,
            id,
            source,
        })
    }

    /// The name of this [RepositoryEntry]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The id of this [RepositoryEntry]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The contents of this [RepositoryEntry]
    pub fn contents(&self) -> Result<Cow<[u8]>> {
        self.source.contents()
    }

    /// The name of this [RepositoryEntry] as a [Config]
    pub fn contents_as_config(&self) -> Result<Config> {
        self.source.contents_as_config()
    }
}

impl PartialEq for RepositoryEntry {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for RepositoryEntry {}

impl Hash for RepositoryEntry {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Borrow<str> for RepositoryEntry {
    fn borrow(&self) -> &str {
        &self.id
    }
}

/// A data-source for the repository.
enum RepositorySource {
    /// Bundled in the executable
    Bundled(&'static [u8]),
    /// Stored in the user-directory
    UserDir(PathBuf),
}

impl RepositorySource {
    /// Extract the config-name from this [RepositorySource]
    pub fn name(&self) -> Result<String> {
        naviz_parser::config::generic::get_item(&mut self.contents_as_config()?, "name")
            .map_err(Error::ConfigReadError)
    }

    /// Read the contents of this [RepositorySource]
    pub fn contents(&self) -> Result<Cow<[u8]>> {
        Ok(match self {
            Self::Bundled(c) => Cow::Borrowed(c),
            Self::UserDir(p) => fs::read(p).map(Cow::Owned).map_err(Error::IoError)?,
        })
    }

    /// Get the contents of this [RepositorySource] as a [Config]
    pub fn contents_as_config(&self) -> Result<Config> {
        config_from_bytes(self.contents()?.borrow())
    }
}

/// Try to parse a [Config] from the passed `bytes`
pub fn config_from_bytes(bytes: &[u8]) -> Result<Config> {
    let config =
        naviz_parser::config::lexer::lex(std::str::from_utf8(bytes).map_err(Error::UTF8Error)?)
            .map_err(Error::lex_error)?;
    let config = naviz_parser::config::parser::parse(&config).map_err(Error::parse_error)?;
    Ok(config.into())
}

/// Insert an [Iterator] of [Result]s into the `target` [HashSet].
///
/// Returns [Ok] with the updated [HashSet] if all [Result]s were [Ok]
/// or the first [Err].
fn insert_results<T: Eq + Hash>(
    mut target: HashSet<T>,
    source: impl IntoIterator<Item = Result<T>>,
) -> Result<HashSet<T>> {
    for value in source.into_iter() {
        target.insert(value?);
    }
    Ok(target)
}

#[cfg(test)]
mod tests {
    use naviz_parser::config::{machine::MachineConfig, visual::VisualConfig};

    use super::*;

    /// Check if all bundled machines can be loaded and parsed successfully.
    #[test]
    fn bundled_machines() {
        let machines = Repository::empty()
            .bundled_machines()
            .expect("Failed to load bundled machines");

        for (id, name) in machines.list() {
            machines
                .get::<MachineConfig>(id)
                .expect("Machine exists in `list`, but `get` returned `None`")
                .unwrap_or_else(|e| panic!("Machine \"{name}\" ({id}) is invalid:\n{e:#?}"));
        }
    }

    /// Check if all bundled styles can be loaded and parsed successfully.
    #[test]
    fn bundled_styles() {
        let styles = Repository::empty()
            .bundled_styles()
            .expect("Failed to load bundled styles");

        for (id, name) in styles.list() {
            styles
                .get::<VisualConfig>(id)
                .expect("Style exists in `list`, but `get` returned `None`")
                .unwrap_or_else(|e| panic!("Style \"{name}\" ({id}) is invalid:\n{e:#?}"));
        }
    }
}
