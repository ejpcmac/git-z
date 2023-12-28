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

//! Configuration for git-z, version 0.2.

// NOTE: Never update the fields of the types defined in this file. Create a new
// version instead.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// The git-z configuration.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// The version of the configuration.
    pub version: String,
    /// The valid commit types.
    pub types: IndexMap<String, String>,
    /// The accepted scopes.
    pub scopes: Option<Scopes>,
    /// The ticket reference configuration.
    pub ticket: Option<Ticket>,
    /// The templates.
    pub templates: Templates,
}

/// Types of accepted scopes.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "accept", rename_all = "snake_case")]
pub enum Scopes {
    /// Accepts any arbitrary scope.
    Any,
    /// Accepts a list of scopes.
    List { list: Vec<String> },
}

/// Ticket reference configuration.
#[derive(Debug, Serialize, Deserialize)]
pub struct Ticket {
    /// Whether the ticket reference is required.
    pub required: bool,
    /// The valid ticket prefixes.
    pub prefixes: Vec<String>,
}

/// Templates.
#[derive(Debug, Serialize, Deserialize)]
pub struct Templates {
    /// The commit message template.
    pub commit: String,
}
