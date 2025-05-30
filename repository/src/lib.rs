// Copyright (c) 2023 - 2025 Chair for Design Automation, TUM
// Copyright (c) 2025 Munich Quantum Software Company GmbH
// All rights reserved.
//
// SPDX-License-Identifier: MIT
//
// Licensed under the MIT License

//! [Repository] for loading configs.
//! Also contains bundled configs.

use std::{
    borrow::{Borrow, Cow},
    collections::HashMap,
    fs,
    hash::Hash,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;
use error::{Error, Result};
use include_dir::{include_dir, Dir};
use naviz_parser::config::{generic::Config, machine::MachineConfig, visual::VisualConfig};

pub mod error;

static BUNDLED_MACHINES: Dir = include_dir!("$CARGO_MANIFEST_DIR/../configs/machines");
static BUNDLED_STYLES: Dir = include_dir!("$CARGO_MANIFEST_DIR/../configs/styles");

const MACHINES_SUBDIR: &str = "machines";
const STYLES_SUBDIR: &str = "styles";

/// A repository of config files.
pub struct Repository(HashMap<String, RepositoryEntry>);

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
                RepositoryEntry::new_with_id(
                    f.path()
                        .file_stem()
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

    /// Gets the path of the passed `subdir` of the user-directory
    fn user_dir(subdir: &str) -> Result<PathBuf> {
        let directory = project_dirs()?.data_dir().join(subdir);

        if !directory.exists() {
            fs::create_dir_all(&directory).map_err(Error::IoError)?;
        }

        Ok(directory)
    }

    /// Loads the configs from the passed `subdir` of the user-directory
    /// into the passed [Repository]
    fn load_user_dir(mut self, subdir: &str) -> Result<Self> {
        self.0 = insert_results(
            self.0,
            Self::user_dir(subdir)?
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
                    RepositoryEntry::new_with_id(
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
        self.load_user_dir(MACHINES_SUBDIR)
    }

    /// Loads the styles from the user-directory into the passed [Repository]
    pub fn user_dir_styles(self) -> Result<Self> {
        self.load_user_dir(STYLES_SUBDIR)
    }

    /// Imports a `file` into the passed `subdir` in the user-directory.
    /// Will validate that the config can be parsed into a valid `C`.
    fn import_to_user_dir<C>(&mut self, subdir: &str, file: &Path) -> Result<()>
    where
        Config: TryInto<C, Error = naviz_parser::config::error::Error>,
    {
        if !file.is_file() {
            return Err(Error::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "File to import is not a file.",
            )));
        }

        let id = file
            .file_stem()
            .ok_or(Error::IdError)?
            .to_string_lossy()
            .into_owned();
        let entry = RepositoryEntry::new(RepositorySource::UserDir(file.to_owned()))?;
        // Ensure the config is valid (i.e., can be parsed correctly)
        entry
            .contents_as_config()?
            .try_into()
            .map_err(Error::ConfigReadError)?;

        fs::copy(
            file,
            Self::user_dir(subdir)?.join(file.file_name().unwrap()),
        )
        .map_err(Error::IoError)?;

        self.0.insert(id, entry);

        Ok(())
    }

    /// Import a machine into the user-directory.
    /// Will validate that the config can be parsed into a valid [MachineConfig].
    pub fn import_machine_to_user_dir(&mut self, file: &Path) -> Result<()> {
        self.import_to_user_dir::<MachineConfig>(MACHINES_SUBDIR, file)
    }

    /// Import a style into the user-directory.
    /// Will validate that the config can be parsed into a valid [VisualConfig].
    pub fn import_style_to_user_dir(&mut self, file: &Path) -> Result<()> {
        self.import_to_user_dir::<VisualConfig>(STYLES_SUBDIR, file)
    }

    /// The list of entries of this repository: `(id, name)`-pairs
    pub fn list(&self) -> Vec<(&str, &str)> {
        self.0
            .iter()
            .map(|(id, entry)| (id.as_str(), entry.name()))
            .collect()
    }

    /// Checks whether the repository has an entry with `id`
    pub fn has(&self, id: &str) -> bool {
        self.0.contains_key(id)
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
            .filter_map(|(id, entry)| {
                Some((
                    id.as_str(),
                    entry.contents_as_config().ok()?.try_into().ok()?,
                ))
            })
            .next()
    }
}

/// An entry in the repository.
/// Contains a cached `name`, an `id`, and the `source`.
/// Is hashed and checked for equality only by `id`.
struct RepositoryEntry {
    /// The name as read from the config-file
    name: String,
    /// The source of the entry
    source: RepositorySource,
}

impl RepositoryEntry {
    /// Creates a new [RepositoryEntry] from the passed `source`.
    /// Will also return the `id` for usage as a [HashMap]-entry.
    /// Will extract the name from the `source`.
    fn new_with_id(id: String, source: RepositorySource) -> Result<(String, Self)> {
        Ok((id, Self::new(source)?))
    }

    /// Creates a new [RepositoryEntry] from the passed `source`.
    /// Will extract the name from the `source`.
    fn new(source: RepositorySource) -> Result<Self> {
        Ok(Self {
            name: source.name()?,
            source,
        })
    }

    /// The name of this [RepositoryEntry]
    pub fn name(&self) -> &str {
        &self.name
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

/// Insert an [Iterator] of [Result]s into the `target` [HashMap].
///
/// Returns [Ok] with the updated [HashMap] if all [Result]s were [Ok]
/// or the first [Err].
fn insert_results<K: Eq + Hash, V>(
    mut target: HashMap<K, V>,
    source: impl IntoIterator<Item = Result<(K, V)>>,
) -> Result<HashMap<K, V>> {
    for result in source.into_iter() {
        let (key, value) = result?;
        target.insert(key, value);
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
