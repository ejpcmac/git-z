// git-z - A Git extension to go beyond.
// Copyright (C) 2023 Jean-Philippe Cugnet <jean-philippe@cugnet.eu>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3 of the License.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Configuration updater.

mod from_v0_1;

use std::{fs, io, marker::PhantomData};

use thiserror::Error;
use toml_edit::Document;

use super::{
    config_file, Config, ConfigFileError, FromTomlError, CONFIG_FILE_NAME,
};

/// A configuration updater.
pub struct ConfigUpdater<State> {
    toml_config: Document,
    parsed_config: Config,
    _state: PhantomData<State>,
}

/// Initial state of the updater.
pub struct Init;

/// Updated state of the updater.
pub struct Updated;

/// An error that can occur when loading the configuration.
#[derive(Debug, Error)]
pub enum LoadError {
    #[error("Failed to get the configuration file path")]
    ConfigFileError(#[from] ConfigFileError),
    #[error("No configuration file")]
    NoConfigFile,
    #[error("Failed to read {CONFIG_FILE_NAME}")]
    ReadError(#[from] io::Error),
    #[error("Invalid configuration in {CONFIG_FILE_NAME}")]
    InvalidConfig(#[from] FromTomlError),
    #[error("Failed to parse {CONFIG_FILE_NAME} into a TOML document")]
    TomlEditError(#[from] toml_edit::TomlError),
}

/// An error that can occur when updating the configuration.
#[derive(Debug, Error)]
pub enum UpdateError {
    #[error(
        "Tried to update from version {tried_from}, but the actual version is {actual}."
    )]
    IncorrectVersion { tried_from: String, actual: String },
}

/// An error that can occur when saving the configuration.
#[derive(Debug, Error)]
pub enum SaveError {
    #[error("Failed to get the configuration file path")]
    ConfigFileError(#[from] ConfigFileError),
    #[error("Failed to write {CONFIG_FILE_NAME}")]
    WriteError(#[from] io::Error),
}

impl ConfigUpdater<Init> {
    /// Loads the configuration into the updater.
    pub fn load() -> Result<Self, LoadError> {
        match fs::read_to_string(config_file()?) {
            Ok(toml) => {
                // Parse the configuration first to ensure it is valid.
                let parsed_config = Config::from_toml(&toml)?;
                let toml_config = toml.parse()?;

                Ok(Self {
                    toml_config,
                    parsed_config,
                    _state: PhantomData,
                })
            }

            Err(error) => match error.kind() {
                io::ErrorKind::NotFound => Err(LoadError::NoConfigFile),
                _ => Err(LoadError::ReadError(error)),
            },
        }
    }

    /// Returns the current version of the configuration.
    pub fn config_version(&self) -> &str {
        &self.parsed_config.version
    }

    /// Updates the configuration from version 0.1.
    pub fn update_from_v0_1(
        mut self,
    ) -> Result<ConfigUpdater<Updated>, UpdateError> {
        let version = self.config_version();
        if version != "0.1" {
            return Err(UpdateError::IncorrectVersion {
                tried_from: "0.1".to_owned(),
                actual: version.to_owned(),
            });
        }

        from_v0_1::update(&mut self.toml_config);

        Ok(ConfigUpdater {
            toml_config: self.toml_config,
            parsed_config: self.parsed_config,
            _state: PhantomData,
        })
    }
}

impl ConfigUpdater<Updated> {
    /// Writes the updated configuration to the configuration file.
    pub fn save(self) -> Result<(), SaveError> {
        fs::write(config_file()?, self.toml_config.to_string())?;
        Ok(())
    }
}
