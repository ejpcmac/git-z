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
// configuration by parsing it to a `Config`. Any error occurring here is a bug,
// hence should lead to a panic.
#![allow(clippy::expect_used, clippy::missing_panics_doc)]

use toml_edit::{DocumentMut, Item};

use crate::config::VERSION;

/// The new documentation for `types`.
pub const TYPES_DOC: &str = "
# The available types of commits and their description.
#
# Types are shown in the dialog in the order they appear in this configuration.
";

/// The new documentation for `scopes`.
pub const SCOPES_DOC: &str = "
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

/// The new documentation for `ticket.prefixes`.
pub const TICKET_PREFIXES_DOC: &str = "# The list of valid ticket prefixes.
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

/// The new documentation for `templates.commit`.
pub const TEMPLATES_COMMIT_DOC: &str = "# The commit template.
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
pub fn update_version(toml_config: &mut DocumentMut) {
    let version = toml_config.get_mut("version").expect("No `version` key");
    *version = Item::Value(VERSION.into());
}
