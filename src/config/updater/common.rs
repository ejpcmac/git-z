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

//! Common helper functions between updaters.

use regex::Regex;
use toml_edit::{Document, Item};

use crate::config::VERSION;

// NOTE: Updaters make a heavy usage of `expect` instead of proper error
// handling. This is because `ConfigUpdater::load` already validates the
// configuration by parsing it to a `Config`. Any error occuring here is a bug,
// hence should lead to a panic.

/// Updates the version.
pub fn update_version(toml_config: &mut Document) {
    let version = toml_config.get_mut("version").expect("No `version` key");
    *version = Item::Value(VERSION.into());
}

/// Replaces an empty ticket prefix by `#`.
pub fn empty_prefix_to_hash(prefixes: &mut Item) {
    let empty_prefix = prefixes
        .as_array_mut()
        .expect("The `ticket.prefixes` key is not an array")
        .iter_mut()
        .find(|i| {
            i.as_str()
                .expect("Items in `ticket.prefixes are not strings")
                .is_empty()
        });

    if let Some(value) = empty_prefix {
        *value = "#".into();
    }
}

/// Adds a condition around the usage of the `ticket` variable.
pub fn add_ticket_condition_to_commit_template(template: &str) -> String {
    let re = Regex::new(r"(.*\{\{ ticket \}\}.*)").expect("Invalid regex");
    re.replace(template, "{% if ticket %}$1{% endif %}")
        .to_string()
}

/// Removes the `#` prefix before the `ticket` variable.
pub fn remove_hash_ticket_prefix_from_commit_template(
    template: &str,
) -> String {
    template.replace("#{{ ticket }}", "{{ ticket }}")
}
