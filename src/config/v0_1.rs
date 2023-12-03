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

//! Configuration for git-z, version 0.1.

// NOTE: Never update the fields of the types defined in this file. Create a new
// version instead.

use serde::{Deserialize, Serialize};

/// The git-z configuration.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// The version of the configuration.
    pub version: String,
    /// The valid commit types.
    pub types: Vec<String>,
    /// The valid scopes.
    pub scopes: Vec<String>,
    /// The commit message template.
    pub template: String,
    /// The valid ticket prefixes.
    pub ticket_prefixes: Vec<String>,
}
