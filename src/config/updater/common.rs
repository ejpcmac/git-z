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

// NOTE: Updaters make a heavy usage of `expect` instead of proper error
// handling. This is because `ConfigUpdater::load` already validates the
// configuration by parsing it to a `Config`. Any error occuring here is a bug,
// hence should lead to a panic.
#![allow(clippy::expect_used, clippy::missing_panics_doc)]

use regex::Regex;
use toml_edit::{Document, Item, Table};

use crate::config::VERSION;

/// The old documentation for `types`.
pub const OLD_TYPES_DOC: &str = "
# The available types of commits.
";

/// The new documentation for `types`.
pub const NEW_TYPES_DOC: &str = "
# The available types of commits and their description.
#
# Types are shown in the dialog in the order they appear in this configuration.
";

/// The old documentation for `scopes`.
pub const OLD_SCOPES_DOC: &str = "
# The accepted scopes.
";

/// The new documentation for `scopes`.
pub const NEW_SCOPES_DOC: &str = "
# The accepted scopes.
#
# This table is optional: if omitted, no scope will be asked for.
";

/// The documentation for `scopes.accept`.
pub const SCOPES_ACCEPT_DOC: &str = "# What kind of scope to accept.
#
# Can be one of: \"any\", \"list\". If it is \"list\", a `list` key containing a list
# of valid scopes is required.
";

/// The documentation for `ticket`.
pub const TICKET_DOC: &str = "
# The ticket / issue reference configuration.
#
# This table is optional: if omitted, no ticket will be asked for.
";

/// The documentation for `ticket.required`.
pub const TICKET_REQUIRED_DOC: &str =
    "# Set to true to require a ticket number.
# Set to false to ask for a ticket without requiring it.
";

/// The old documentation for `ticket.prefixes`.
pub const OLD_TICKET_PREFIXES_DOC: &str = "# The list of valid ticket prefixes.
";

/// The new documentation for `ticket.prefixes`.
pub const NEW_TICKET_PREFIXES_DOC: &str = "# The list of valid ticket prefixes.
#
# Can be a `#` for GitHub / GitLab issues, or a Jira key for instance.
";

/// The documentation for `templates`.
pub const TEMPLATES_DOC: &str = "
# Templates written with the Tera [1] templating engine.
#
# Each template is documented below, with its list of available variables.
# Variables marked as optional can be `None`, hence should be checked for
# presence in the template.
#
# [1] https://tera.netlify.app/
";

/// The old documentation for `templates.commit`.
pub const OLD_TEMPLATES_COMMIT_DOC: &str = "# The commit message template, written with the Tera [1] templating engine.
# [1] https://tera.netlify.app/
";

/// The new documentation for `templates.commit`.
pub const NEW_TEMPLATES_COMMIT_DOC: &str = "# The commit template.
#
# Available variables:
#
#   - type: the type of commit
#   - scope (optional): the scope of the commit
#   - description: the short description
#   - breaking_change (optional): the description of the breaking change
#   - ticket (optional): the ticket reference
";

/// Updates the version.
pub fn update_version(toml_config: &mut Document) {
    let version = toml_config.get_mut("version").expect("No `version` key");
    *version = Item::Value(VERSION.into());
}

/// Switches the accepted scopes from `list` to `any`.
pub fn switch_scopes_to_any(toml_config: &mut Document) {
    if let Some(scopes) = toml_config.get_mut("scopes") {
        let scopes = scopes
            .as_table_mut()
            .expect("The `scopes` key is not a table.");

        scopes.insert("accept", Item::Value("any".into()));
        scopes.remove("list");
    }
}

/// Replaces an empty ticket prefix by `#`.
pub fn empty_prefix_to_hash(prefixes: &mut Item) {
    let empty_prefix = prefixes
        .as_array_mut()
        .expect("The `ticket.prefixes` key is not an array")
        .iter_mut()
        .find(|item| {
            item.as_str()
                .expect("Items in `ticket.prefixes are not strings")
                .is_empty()
        });

    if let Some(value) = empty_prefix {
        *value = "#".into();
    }
}

/// Adds a condition around the usage of the `ticket` variable.
pub fn add_ticket_condition_to_commit_template(template: &str) -> String {
    // NOTE(unwrap): This regex is known to be valid.
    #[allow(clippy::unwrap_used)]
    let re = Regex::new(r"(.*\{\{ ticket \}\}.*)").unwrap();
    re.replace(template, "{% if ticket %}$1{% endif %}")
        .to_string()
}

/// Removes the `#` prefix before the `ticket` variable.
pub fn remove_hash_ticket_prefix_from_commit_template(
    template: &str,
) -> String {
    template.replace("#{{ ticket }}", "{{ ticket }}")
}

/// Updates the documentation for the `types` table.
pub fn update_types_doc(toml_config: &mut Document) {
    let decor = toml_config
        .get_mut("types")
        .expect("No `types` key")
        .as_table_mut()
        .expect("The `types` key is not a table")
        .decor_mut();

    let doc = decor
        .prefix()
        .expect("No prefix decorator for key `types`")
        .as_str()
        .expect("Improper string in the prefix decorator of the `types` key");

    decor.set_prefix(doc.replace(OLD_TYPES_DOC, NEW_TYPES_DOC));
}

/// Updates the documentation for the `scopes` table.
pub fn update_scopes_doc(toml_config: &mut Document) {
    if let Some(scopes) = toml_config.get_mut("scopes") {
        let scopes = scopes
            .as_table_mut()
            .expect("The `scopes` key is not a table");

        let decor = scopes.decor_mut();

        let doc = decor
            .prefix()
            .expect("No prefix decorator for key `scopes`")
            .as_str()
            .expect(
                "Improper string in the prefix decorator of the `scopes` key",
            );

        decor.set_prefix(doc.replace(OLD_SCOPES_DOC, NEW_SCOPES_DOC));

        update_scopes_accept_doc(scopes);
    }
}

/// Updates the documentation for `scopes.accept`.
pub fn update_scopes_accept_doc(scopes: &mut Table) {
    let decor = scopes
        .key_decor_mut("accept")
        .expect("No `scopes.accept` key");

    let doc = decor
            .prefix()
            .expect("No prefix decorator for key `scopes.accept`")
            .as_str()
            .expect(
                "Improper string in the prefix decorator of the `scopes.accept` key",
            );

    if doc.trim().is_empty() {
        decor.set_prefix(SCOPES_ACCEPT_DOC);
    }
}

/// Updates the documentation for the `ticket` table.
pub fn update_ticket_doc(toml_config: &mut Document) {
    if let Some(ticket) = toml_config.get_mut("ticket") {
        let ticket = ticket
            .as_table_mut()
            .expect("The `ticket` key is not a table");

        let decor = ticket.decor_mut();

        let doc = decor
            .prefix()
            .expect("No prefix decorator for key `ticket`")
            .as_str()
            .expect(
                "Improper string in the prefix decorator of the `ticket` key",
            );

        if doc.trim().is_empty() {
            decor.set_prefix(TICKET_DOC);
        }

        update_ticket_required_doc(ticket);
        update_ticket_prefixes_doc(ticket);
    }
}

/// Updates the documentation for `ticket.required`.
pub fn update_ticket_required_doc(ticket: &mut Table) {
    let decor = ticket
        .key_decor_mut("required")
        .expect("No `ticket.required` key");

    let doc = decor
            .prefix()
            .expect("No prefix decorator for key `ticket.required`")
            .as_str()
            .expect(
                "Improper string in the prefix decorator of the `ticket.required` key",
            );

    if doc.trim().is_empty() {
        decor.set_prefix(TICKET_REQUIRED_DOC);
    }
}

/// Updates the documentation for `ticket.prefixes`.
pub fn update_ticket_prefixes_doc(ticket: &mut Table) {
    let decor = ticket
        .key_decor_mut("prefixes")
        .expect("No `ticket.prefixes` key");

    let doc = decor
            .prefix()
            .expect("No prefix decorator for key `ticket.prefixes`")
            .as_str()
            .expect(
                "Improper string in the prefix decorator of the `ticket.prefixes` key",
            );

    decor.set_prefix(
        doc.replace(OLD_TICKET_PREFIXES_DOC, NEW_TICKET_PREFIXES_DOC),
    );
}

/// Updates the documentation for the `templates` table.
pub fn update_templates_doc(toml_config: &mut Document) {
    let templates = toml_config
        .get_mut("templates")
        .expect("No `templates` key")
        .as_table_mut()
        .expect("The `templates` key is not a table");

    let decor = templates.decor_mut();

    let doc = decor
        .prefix()
        .expect("No prefix decorator for key `templates`")
        .as_str()
        .expect(
            "Improper string in the prefix decorator of the `templates` key",
        );

    if doc.trim().is_empty() {
        decor.set_prefix(TEMPLATES_DOC);
    }

    update_templates_commit_doc(templates);
}

/// Updates the documentation for `templates.commit`.
pub fn update_templates_commit_doc(templates: &mut Table) {
    let decor = templates
        .key_decor_mut("commit")
        .expect("No `templates.commit` key");

    let doc = decor
            .prefix()
            .expect("No prefix decorator for key `templates.commit`")
            .as_str()
            .expect(
                "Improper string in the prefix decorator of the `templates.commit` key",
            );

    decor.set_prefix(
        doc.replace(OLD_TEMPLATES_COMMIT_DOC, NEW_TEMPLATES_COMMIT_DOC),
    );
}
