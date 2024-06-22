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

mod common;
mod from_v0_1;

use std::{fs, io, marker::PhantomData};

use thiserror::Error;
use toml_edit::Document;

use super::{
    config_file, Config, ConfigFileError, FromTomlError, CONFIG_FILE_NAME,
};

/// A configuration updater.
#[must_use]
pub struct ConfigUpdater<State> {
    /// The parsed configuration.
    parsed_config: Config,
    /// The editable TOML document.
    toml_config: Document,
    /// The state of the updater.
    _state: PhantomData<State>,
}

/// Initial state of the updater.
pub struct Init;

/// Updated state of the updater.
pub struct Updated;

/// Whether to ask and require a ticket.
#[derive(Debug, Clone, Copy)]
pub enum AskForTicket {
    /// Ask for a ticket.
    Ask {
        /// Require the ticket.
        require: bool,
    },
    /// Do not ask for a ticket.
    DontAsk,
}

/// Errors that can occur when loading the configuration.
#[derive(Debug, Error)]
pub enum LoadError {
    /// The path of the configuration cannot be resolved.
    #[error("Failed to get the configuration file path")]
    ConfigFileError(#[from] ConfigFileError),
    /// There is no configuration file.
    #[error("No configuration file")]
    NoConfigFile,
    /// An error has occured while reading the configuration file.
    #[error("Failed to read {CONFIG_FILE_NAME}")]
    ReadError(#[from] io::Error),
    /// The configuration is invalid.
    #[error("Invalid configuration in {CONFIG_FILE_NAME}")]
    InvalidConfig(#[from] FromTomlError),
    /// The configuration is not a valid TOML document.
    #[error("Failed to parse {CONFIG_FILE_NAME} into a TOML document")]
    TomlEditError(#[from] toml_edit::TomlError),
}

/// Errors that can occur when updating the configuration.
#[derive(Debug, Error)]
pub enum UpdateError {
    /// The version of the configuration is not matching the updater.
    #[error(
        "Tried to update from version {tried_from}, but the actual version is {actual}."
    )]
    IncorrectVersion {
        /// The version from which the updater knows how to update.
        tried_from: String,
        /// The actual version of the configuration.
        actual: String,
    },
}

/// Errors that can occur when saving the configuration.
#[derive(Debug, Error)]
pub enum SaveError {
    /// The path of the configuration file cannot be resolved.
    #[error("Failed to get the configuration file path")]
    ConfigFileError(#[from] ConfigFileError),
    /// Error while writing the configuration file.
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
                    parsed_config,
                    toml_config,
                    _state: PhantomData,
                })
            }

            Err(error) => match error.kind() {
                io::ErrorKind::NotFound => Err(LoadError::NoConfigFile),
                _ => Err(LoadError::ReadError(error)),
            },
        }
    }

    /// Returns the parsed configuration.
    pub fn parsed_config(&self) -> &Config {
        &self.parsed_config
    }

    /// Returns the current version of the configuration.
    pub fn config_version(&self) -> &str {
        &self.parsed_config.version
    }

    /// Updates the configuration from version 0.1.
    pub fn update_from_v0_1(
        mut self,
        switch_scopes_to_any: bool,
        ask_for_ticket: AskForTicket,
        empty_prefix_to_hash: bool,
    ) -> Result<ConfigUpdater<Updated>, UpdateError> {
        self.check_version("0.1")?;

        from_v0_1::update(
            &mut self.toml_config,
            switch_scopes_to_any,
            ask_for_ticket,
            empty_prefix_to_hash,
        );

        Ok(ConfigUpdater {
            parsed_config: self.parsed_config,
            toml_config: self.toml_config,
            _state: PhantomData,
        })
    }

    /// Checks the configuration version matches the updater.
    fn check_version(&self, updater_version: &str) -> Result<(), UpdateError> {
        let config_version = self.config_version();

        if config_version == updater_version {
            Ok(())
        } else {
            Err(UpdateError::IncorrectVersion {
                tried_from: updater_version.to_owned(),
                actual: config_version.to_owned(),
            })
        }
    }
}

impl ConfigUpdater<Updated> {
    /// Writes the updated configuration to the configuration file.
    pub fn save(self) -> Result<(), SaveError> {
        fs::write(config_file()?, self.toml_config.to_string())?;
        Ok(())
    }
}
