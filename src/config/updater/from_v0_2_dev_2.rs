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

//! Configuration updater from version 0.2-dev.2.

use toml_edit::Document;

use super::common;

/// Updates the configuration from version 0.2-dev.2.
pub fn update(toml_config: &mut Document, switch_scopes_to_any: bool) {
    common::update_version(toml_config);

    if switch_scopes_to_any {
        common::switch_scopes_to_any(toml_config);
    }
}
