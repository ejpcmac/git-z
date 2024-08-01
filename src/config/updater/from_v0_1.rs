// git-z - A Git extension to go beyond.
// Copyright (C) 2023-2024 Jean-Philippe Cugnet <jean-philippe@cugnet.eu>
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
// configuration by parsing it to a `Config`. Any error occurring here is a bug,
// hence should lead to a panic.
#![allow(clippy::expect_used, clippy::missing_panics_doc)]

use indoc::indoc;
use regex::Regex;
use toml_edit::{DocumentMut, Item, Table};

use super::{super::split_type_and_doc, common, AskForTicket};

/// The old configuration for `types`.
const OLD_TYPES_DOC: &str = indoc! {"

    # The available types of commits.
    #
    # This is a list of types (1 word) and their description, separated by one or
    # more spaces.
"};

/// The old configuration for `scopes`.
const OLD_SCOPES_DOC: &str = indoc! {"

    # The list of valid scopes.
"};

/// The old documentation for `ticket.prefixes`.
pub const OLD_TICKET_PREFIXES_DOC: &str = indoc! {"
    # The list of valid ticket prefixes.
"};

/// The old documentation for `templates.commit`.
pub const OLD_TEMPLATES_COMMIT_DOC: &str = indoc! {"
    # The commit message template, written with the Tera [1] templating engine.
    # [1] https://tera.netlify.app/
"};

/// Updates the configuration from version 0.1.
pub fn update(
    toml_config: &mut DocumentMut,
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

/// Updates the configuration for the types.
fn update_types(toml_config: &mut DocumentMut) {
    let (key, value) =
        toml_config.get_key_value("types").expect("No `types` key");

    let doc = key
        .leaf_decor()
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
        .set_prefix(doc.replace(OLD_TYPES_DOC, common::TYPES_DOC));

    // Replace the old configuration with the new one.
    toml_config.insert("types", Item::Table(types));
}

/// Updates the configuration for scopes.
fn update_scopes(toml_config: &mut DocumentMut, switch_scopes_to_any: bool) {
    let (key, value) = toml_config
        .get_key_value("scopes")
        .expect("No `scopes` key");

    let doc = key
        .leaf_decor()
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
        .set_prefix(doc.replace(OLD_SCOPES_DOC, common::SCOPES_DOC));
    scopes
        .key_mut("accept")
        .expect("No `scopes.accept` key")
        .leaf_decor_mut()
        .set_prefix(common::SCOPES_ACCEPT_DOC);

    // Replace the old configuration with the new one.
    toml_config.insert("scopes", Item::Table(scopes));
}

/// Updates the configuration for ticket references.
fn update_ticket(
    toml_config: &mut DocumentMut,
    required: bool,
    empty_prefix_to_hash: bool,
) {
    let (key, value) = toml_config
        .get_key_value("ticket_prefixes")
        .expect("No `ticket_prefixes` key");

    let doc = key
        .leaf_decor()
        .prefix()
        .expect("No prefix decorator for key `ticket_prefixes`")
        .as_str()
        .expect("Improper string in the prefix decorator of the `ticket_prefixes` key");

    // Update the value itself.
    let mut prefixes = value.clone();
    if empty_prefix_to_hash {
        replace_empty_prefix_with_hash(&mut prefixes);
    }

    // Update the configuration format.
    let mut ticket = Table::new();
    ticket.insert("required", Item::Value(required.into()));
    ticket.insert("prefixes", prefixes);

    // Update the documentation.
    ticket.decor_mut().set_prefix(common::TICKET_DOC);
    ticket
        .key_mut("required")
        .expect("No `ticket.required` key")
        .leaf_decor_mut()
        .set_prefix(common::TICKET_REQUIRED_DOC);
    ticket
        .key_mut("prefixes")
        .expect("No `ticket.prefixes` key")
        .leaf_decor_mut()
        .set_prefix(
            doc.trim_start()
                .replace(OLD_TICKET_PREFIXES_DOC, common::TICKET_PREFIXES_DOC),
        );

    // Replace the old configuration with the new one.
    toml_config.remove("ticket_prefixes");
    toml_config.insert("ticket", Item::Table(ticket));
}

/// Replaces an empty ticket prefix by `#`.
fn replace_empty_prefix_with_hash(prefixes: &mut Item) {
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

/// Removes the configuration for ticket references.
fn remove_ticket(toml_config: &mut DocumentMut) {
    toml_config.remove("ticket_prefixes");
}

/// Updates the configuration for templates.
fn update_templates(toml_config: &mut DocumentMut, remove_hash_prefix: bool) {
    let (key, value) = toml_config
        .get_key_value("template")
        .expect("No `template` key");

    let doc = key
        .leaf_decor()
        .prefix()
        .expect("No prefix decorator for key `template`")
        .as_str()
        .expect(
            "Improper string in the prefix decorator of the `template` key",
        );

    // Update the template itself.
    let template = value.as_str().expect("The `template` key is not a string");
    let template = add_ticket_condition_to_commit_template(template);
    let template = if remove_hash_prefix {
        remove_hash_ticket_prefix_from_commit_template(&template)
    } else {
        template
    };

    // Update the configuration format.
    let mut templates = Table::new();
    templates.insert("commit", Item::Value(template.into()));

    // Update the documentation.
    templates.decor_mut().set_prefix(common::TEMPLATES_DOC);
    templates
        .key_mut("commit")
        .expect("No `commit` key")
        .leaf_decor_mut()
        .set_prefix(
            doc.trim_start().replace(
                OLD_TEMPLATES_COMMIT_DOC,
                common::TEMPLATES_COMMIT_DOC,
            ),
        );

    // Replace the old configuration with the new one.
    toml_config.remove("template");
    toml_config.insert("templates", Item::Table(templates));
}

/// Adds a condition around the usage of the `ticket` variable.
fn add_ticket_condition_to_commit_template(template: &str) -> String {
    // NOTE(unwrap): This regex is known to be valid.
    #[allow(clippy::unwrap_used)]
    let re = Regex::new(r"(.*\{\{ ticket \}\}.*)").unwrap();
    re.replace(template, "{% if ticket %}$1{% endif %}")
        .to_string()
}

/// Removes the `#` prefix before the `ticket` variable.
fn remove_hash_ticket_prefix_from_commit_template(template: &str) -> String {
    template.replace("#{{ ticket }}", "{{ ticket }}")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::pedantic, clippy::restriction)]

    use super::*;

    const V0_1_STANDARD: &str =
        include_str!("../../../tests/res/config/v0_1_standard.toml");

    const V0_1_USER_COMMENTS: &str =
        include_str!("../../../tests/res/config/v0_1_user-comments.toml");

    const V0_1_DOC_AND_USER_COMMENTS: &str = include_str!(
        "../../../tests/res/config/v0_1_doc-and-user-comments.toml"
    );

    const V0_2_STANDARD: &str =
        include_str!("../../../tests/res/config/v0_2_standard.toml");

    const V0_2_SCOPES_ANY: &str =
        include_str!("../../../tests/res/config/v0_2_scopes-any.toml");

    const V0_2_TICKET_NOT_REQUIRED: &str =
        include_str!("../../../tests/res/config/v0_2_ticket-not-required.toml");

    const V0_2_TICKET_NOT_ASKED_FOR: &str = include_str!(
        "../../../tests/res/config/v0_2_ticket-not-asked-for.toml"
    );

    const V0_2_KEEP_EMPTY_PREFIX: &str =
        include_str!("../../../tests/res/config/v0_2_keep-empty-prefix.toml");

    const V0_2_USER_COMMENTS: &str =
        include_str!("../../../tests/res/config/v0_2_user-comments.toml");

    const V0_2_DOC_AND_USER_COMMENTS: &str = include_str!(
        "../../../tests/res/config/v0_2_doc-and-user-comments.toml"
    );

    #[test]
    fn test_update_works_with_standard_config() {
        let source = V0_1_STANDARD;
        let expected = V0_2_STANDARD;

        let mut document = source.parse().unwrap();
        update(
            &mut document,
            false,
            AskForTicket::Ask { require: true },
            true,
        );

        let actual = document.to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_update_can_switch_scopes_to_any() {
        let source = V0_1_STANDARD;
        let expected = V0_2_SCOPES_ANY;

        let mut document = source.parse().unwrap();
        update(
            &mut document,
            true,
            AskForTicket::Ask { require: true },
            true,
        );

        let actual = document.to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_update_can_ask_for_ticket_without_requiring_it() {
        let source = V0_1_STANDARD;
        let expected = V0_2_TICKET_NOT_REQUIRED;

        let mut document = source.parse().unwrap();
        update(
            &mut document,
            false,
            AskForTicket::Ask { require: false },
            true,
        );

        let actual = document.to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_update_can_omit_to_ask_for_a_ticket() {
        let source = V0_1_STANDARD;
        let expected = V0_2_TICKET_NOT_ASKED_FOR;

        let mut document = source.parse().unwrap();
        update(&mut document, false, AskForTicket::DontAsk, true);

        let actual = document.to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_update_can_skip_updating_an_empty_ticket_prefix_to_hash() {
        let source = V0_1_STANDARD;
        let expected = V0_2_KEEP_EMPTY_PREFIX;

        let mut document = source.parse().unwrap();
        update(
            &mut document,
            false,
            AskForTicket::Ask { require: true },
            false,
        );

        let actual = document.to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_update_preserves_user_comments() {
        let source = V0_1_USER_COMMENTS;
        let expected = V0_2_USER_COMMENTS;

        let mut document = source.parse().unwrap();
        update(
            &mut document,
            false,
            AskForTicket::Ask { require: true },
            true,
        );

        let actual = document.to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_update_updates_default_doc_when_mixed_with_user_comments() {
        let source = V0_1_DOC_AND_USER_COMMENTS;
        let expected = V0_2_DOC_AND_USER_COMMENTS;

        let mut document = source.parse().unwrap();
        update(
            &mut document,
            false,
            AskForTicket::Ask { require: true },
            true,
        );

        let actual = document.to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_add_ticket_condition_makes_reference_conditional_on_refs_footer() {
        let source = indoc! {"
            {{ type }}{% if scope %}({{ scope }}){% endif %}{% if breaking_change %}!{% endif %}: {{ description }}

            # Feel free to enter a longer description here.

            Refs: {{ ticket }}

            {% if breaking_change %}BREAKING CHANGE: {{ breaking_change }}{% endif %}
        "};

        let expected = indoc! {"
            {{ type }}{% if scope %}({{ scope }}){% endif %}{% if breaking_change %}!{% endif %}: {{ description }}

            # Feel free to enter a longer description here.

            {% if ticket %}Refs: {{ ticket }}{% endif %}

            {% if breaking_change %}BREAKING CHANGE: {{ breaking_change }}{% endif %}
        "};

        let actual = add_ticket_condition_to_commit_template(source);

        assert_eq!(actual, expected);
    }
}
