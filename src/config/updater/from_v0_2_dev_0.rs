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

//! Configuration updater from version 0.2-dev.0.

use toml_edit::{Document, Item};

use super::{common, AskForTicket};

/// Updates the configuration from version 0.2-dev.0.
pub fn update(toml_config: &mut Document, ask_for_ticket: AskForTicket) {
    common::update_version(toml_config);

    match ask_for_ticket {
        AskForTicket::Ask { require } => {
            add_required_key_to_ticket(toml_config, require);
        }
        AskForTicket::DontAsk => {
            remove_ticket(toml_config);
        }
    }

    update_commit_template(toml_config);
}

// NOTE: Updaters make a heavy usage of `expect` instead of proper error
// handling. This is because `ConfigUpdater::load` already validates the
// configuration by parsing it to a `Config`. Any error occuring here is a bug,
// hence should lead to a panic.

fn add_required_key_to_ticket(toml_config: &mut Document, required: bool) {
    let ticket = toml_config
        .get_mut("ticket")
        .expect("No `ticket` key")
        .as_table_mut()
        .expect("The `ticket` key is not a table");

    let prefixes_doc = ticket
        .key_decor("prefixes")
        .expect("No `ticket.prefixes` key")
        .prefix()
        .expect("No prefix decorator for key `ticket.prefixes`")
        .as_str()
        .expect("Improper string in the prefix decorator of the `ticket.prefixes` key")
        .to_owned();

    let prefixes = ticket.remove("prefixes").expect("No `ticket.prefixes` key");

    ticket.insert("required", Item::Value(required.into()));
    ticket.insert("prefixes", prefixes);
    ticket
        .key_decor_mut("prefixes")
        .expect("No `ticket.prefixes` key")
        .set_prefix(prefixes_doc);
}

fn remove_ticket(toml_config: &mut Document) {
    toml_config.remove("ticket");
}

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

    let template = common::add_ticket_condition_to_commit_template(template);

    *commit_template = Item::Value(template.into());
}
