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

//! Configuration updater from version 0.2-dev.1.

// NOTE: Updaters make a heavy usage of `expect` instead of proper error
// handling. This is because `ConfigUpdater::load` already validates the
// configuration by parsing it to a `Config`. Any error occuring here is a bug,
// hence should lead to a panic.
#![allow(clippy::expect_used, clippy::missing_panics_doc)]

use toml_edit::{Document, Item};

use super::common;

/// Updates the configuration from version 0.2-dev.1.
pub fn update(
    toml_config: &mut Document,
    switch_scopes_to_any: bool,
    empty_prefix_to_hash: bool,
) {
    common::update_version(toml_config);

    if switch_scopes_to_any {
        common::switch_scopes_to_any(toml_config);
    }

    if empty_prefix_to_hash {
        update_ticket_prefix(toml_config);
        update_commit_template(toml_config);
    }

    common::update_types_doc(toml_config);
    common::update_scopes_doc(toml_config);
    common::update_ticket_doc(toml_config);
    common::update_templates_doc(toml_config);
}

/// Updates the configuration for ticket prefixes.
fn update_ticket_prefix(toml_config: &mut Document) {
    if let Some(ticket) = toml_config.get_mut("ticket") {
        let prefixes = ticket
            .as_table_mut()
            .expect("The `ticket` key is not a table")
            .get_mut("prefixes")
            .expect("No `ticket.prefixes` key");

        common::empty_prefix_to_hash(prefixes);
    }
}

/// Updates the configuration for commit templates.
fn update_commit_template(toml_config: &mut Document) {
    let commit_template = toml_config
        .get_mut("templates")
        .expect("No `templates` key")
        .as_table_mut()
        .expect("The `templates` key is not a table")
        .get_mut("commit")
        .expect("No `templates.commit` key");

    let template = commit_template
        .as_str()
        .expect("The `templates.commit` key is not a string");

    let template =
        common::remove_hash_ticket_prefix_from_commit_template(template);

    *commit_template = Item::Value(template.into());
}
