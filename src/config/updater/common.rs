// SPDX-FileCopyrightText: 2023-2024 Jean-Philippe Cugnet <jean-philippe@cugnet.eu>
// SPDX-License-Identifier: GPL-3.0-only

//! Common helper functions between updaters.

#![expect(
    clippy::expect_used,
    clippy::missing_panics_doc,
    reason = "Updaters make a heavy usage of `expect` instead of proper error \
        handling. This is because `ConfigUpdater::load` already validates the \
        configuration by parsing it to a `Config`. Any error occurring here is \
        a bug, hence should lead to a panic."
)]

use indoc::indoc;
use toml_edit::{DocumentMut, Item};

use crate::config::VERSION;

/// The new documentation for `types`.
pub const TYPES_DOC: &str = indoc! {"

    # The available types of commits and their description.
    #
    # Types are shown in the dialog in the order they appear in this configuration.
"};

/// The new documentation for `scopes`.
pub const SCOPES_DOC: &str = indoc! {"

    # The accepted scopes.
    #
    # This table is optional: if omitted, no scope will be asked for.
"};

/// The documentation for `scopes.accept`.
pub const SCOPES_ACCEPT_DOC: &str = indoc! {r#"
    # What kind of scope to accept.
    #
    # Can be one of: "any", "list". If it is "list", a `list` key containing a list
    # of valid scopes is required.
"#};

/// The documentation for `ticket`.
pub const TICKET_DOC: &str = indoc! {"

    # The ticket / issue reference configuration.
    #
    # This table is optional: if omitted, no ticket will be asked for.
"};

/// The documentation for `ticket.required`.
pub const TICKET_REQUIRED_DOC: &str = indoc! {"
    # Set to true to require a ticket number.
    # Set to false to ask for a ticket without requiring it.
"};

/// The new documentation for `ticket.prefixes`.
pub const TICKET_PREFIXES_DOC: &str = indoc! {"
    # The list of valid ticket prefixes.
    #
    # Can be a `#` for GitHub / GitLab issues, or a Jira key for instance.
"};

/// The documentation for `templates`.
pub const TEMPLATES_DOC: &str = indoc! {"

    # Templates written with the Tera [1] templating engine.
    #
    # Each template is documented below, with its list of available variables.
    # Variables marked as optional can be `None`, hence should be checked for
    # presence in the template.
    #
    # [1] https://tera.netlify.app/
"};

/// The new documentation for `templates.commit`.
pub const TEMPLATES_COMMIT_DOC: &str = indoc! {"
    # The commit template.
    #
    # Available variables:
    #
    #   - type: the type of commit
    #   - scope (optional): the scope of the commit
    #   - description: the short description
    #   - breaking_change (optional): the description of the breaking change
    #   - ticket (optional): the ticket reference
"};

/// Updates the version.
pub fn update_version(toml_config: &mut DocumentMut) {
    let version = toml_config.get_mut("version").expect("No `version` key");
    *version = Item::Value(VERSION.into());
}
