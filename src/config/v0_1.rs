// SPDX-FileCopyrightText: 2023 Jean-Philippe Cugnet <jean-philippe@cugnet.eu>
// SPDX-License-Identifier: GPL-3.0-only

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
