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

//! Configuration updater from version 0.1.

// NOTE: Updaters make a heavy usage of `expect` instead of proper error
// handling. This is because `ConfigUpdater::load` already validates the
// configuration by parsing it to a `Config`. Any error occuring here is a bug,
// hence should lead to a panic.
#![allow(clippy::expect_used)]

use toml_edit::{Document, Item, Table};

use super::{super::split_type_and_doc, common, AskForTicket};

const OLD_TYPES_DOC: &str = "
# The available types of commits.
#
# This is a list of types (1 word) and their description, separated by one or
# more spaces.
";

const OLD_SCOPES_DOC: &str = "
#The list of valid scopes.
";

/// Updates the configuration from version 0.1.
pub fn update(
    toml_config: &mut Document,
    switch_scopes_to_any: bool,
    ask_for_ticket: AskForTicket,
    empty_prefix_to_hash: bool,
) {
    common::update_version(toml_config);
    update_types(toml_config);
    update_scopes(toml_config, switch_scopes_to_any);

    match ask_for_ticket {
        AskForTicket::Ask { require } => {
            update_ticket(toml_config, require, empty_prefix_to_hash);
        }
        AskForTicket::DontAsk => remove_ticket(toml_config),
    }

    update_templates(toml_config, empty_prefix_to_hash);
}

fn update_types(toml_config: &mut Document) {
    let (key, value) =
        toml_config.get_key_value("types").expect("No `types` key");

    let doc = key
        .decor()
        .prefix()
        .expect("No prefix decorator for key `types`")
        .as_str()
        .expect("Improper string in the prefix decorator of the `types` key");

    // Update the configuration format.
    let mut types: Table = value
        .as_array()
        .expect("The `types` key is not an array")
        .iter()
        .map(|ty| {
            ty.as_str()
                .expect("Values of the `types` array are not strings")
        })
        .map(split_type_and_doc)
        .collect();

    // Update the documentation.
    types
        .decor_mut()
        .set_prefix(doc.replace(OLD_TYPES_DOC, common::NEW_TYPES_DOC));

    // Replace the old configuration with the new one.
    toml_config.insert("types", Item::Table(types));
}

fn update_scopes(toml_config: &mut Document, switch_scopes_to_any: bool) {
    let (key, value) = toml_config
        .get_key_value("scopes")
        .expect("No `scopes` key");

    let doc = key
        .decor()
        .prefix()
        .expect("No prefix decorator for key `scopes`")
        .as_str()
        .expect("Improper string in the prefix decorator of the `scopes` key");

    // Update the configuration format.
    let mut scopes = Table::new();

    if switch_scopes_to_any {
        scopes.insert("accept", Item::Value("any".into()));
    } else {
        scopes.insert("accept", Item::Value("list".into()));
        scopes.insert("list", value.clone());
    }

    // Update the documentation.
    scopes
        .decor_mut()
        .set_prefix(doc.replace(OLD_SCOPES_DOC, common::NEW_SCOPES_DOC));
    scopes
        .key_decor_mut("accept")
        .expect("No `scopes.accept` key")
        .set_prefix(common::SCOPES_ACCEPT_DOC);

    // Replace the old configuration with the new one.
    toml_config.insert("scopes", Item::Table(scopes));
}

fn update_ticket(
    toml_config: &mut Document,
    required: bool,
    empty_prefix_to_hash: bool,
) {
    let (key, value) = toml_config
        .get_key_value("ticket_prefixes")
        .expect("No `ticket_prefixes` key");

    let doc = key
        .decor()
        .prefix()
        .expect("No prefix decorator for key `ticket_prefixes`")
        .as_str()
        .expect("Improper string in the prefix decorator of the `ticket_prefixes` key");

    // Update the value itself.
    let mut prefixes = value.clone();
    if empty_prefix_to_hash {
        common::empty_prefix_to_hash(&mut prefixes);
    }

    // Update the configuration format.
    let mut ticket = Table::new();
    ticket.insert("required", Item::Value(required.into()));
    ticket.insert("prefixes", prefixes);

    // Update the documentation.
    ticket.decor_mut().set_prefix(common::TICKET_DOC);
    ticket
        .key_decor_mut("required")
        .expect("No `ticket.required` key")
        .set_prefix(common::TICKET_REQUIRED_DOC);
    ticket
        .key_decor_mut("prefixes")
        .expect("No `ticket.prefixes` key")
        .set_prefix(doc.trim_start().replace(
            common::OLD_TICKET_PREFIXES_DOC,
            common::NEW_TICKET_PREFIXES_DOC,
        ));

    // Replace the old configuration with the new one.
    toml_config.remove("ticket_prefixes");
    toml_config.insert("ticket", Item::Table(ticket));
}

fn remove_ticket(toml_config: &mut Document) {
    toml_config.remove("ticket_prefixes");
}

fn update_templates(toml_config: &mut Document, remove_hash_prefix: bool) {
    let (key, value) = toml_config
        .get_key_value("template")
        .expect("No `template` key");

    let doc = key
        .decor()
        .prefix()
        .expect("No prefix decorator for key `template`")
        .as_str()
        .expect(
            "Improper string in the prefix decorator of the `template` key",
        );

    // Update the template itself.
    let template = value.as_str().expect("The `template` key is not a string");
    let template = common::add_ticket_condition_to_commit_template(template);
    let template = if remove_hash_prefix {
        common::remove_hash_ticket_prefix_from_commit_template(&template)
    } else {
        template
    };

    // Update the configuration format.
    let mut templates = Table::new();
    templates.insert("commit", Item::Value(template.into()));

    // Update the documentation.
    templates.decor_mut().set_prefix(common::TEMPLATES_DOC);
    templates
        .key_decor_mut("commit")
        .expect("No `commit` key")
        .set_prefix(doc.trim_start().replace(
            common::OLD_TEMPLATES_COMMIT_DOC,
            common::NEW_TEMPLATES_COMMIT_DOC,
        ));

    // Replace the old configuration with the new one.
    toml_config.remove("template");
    toml_config.insert("templates", Item::Table(templates));
}
