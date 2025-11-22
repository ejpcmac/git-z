// SPDX-FileCopyrightText: 2023 Jean-Philippe Cugnet <jean-philippe@cugnet.eu>
// SPDX-License-Identifier: GPL-3.0-only

//! Configuration for git-z, version 0.2.

// NOTE: Never update the fields of the types defined in this file. Create a new
// version instead.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// The git-z configuration.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// The version of the configuration.
    pub version: String,
    /// The valid commit types.
    pub types: IndexMap<String, String>,
    /// The accepted scopes.
    pub scopes: Option<Scopes>,
    /// The ticket reference configuration.
    pub ticket: Option<Ticket>,
    /// The templates.
    pub templates: Templates,
}

/// Types of accepted scopes.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "accept", rename_all = "snake_case")]
pub enum Scopes {
    /// Accepts any arbitrary scope.
    Any,
    /// Accepts only scopes from a list.
    List {
        /// The list of accepted scopes.
        list: Vec<String>,
    },
}

/// Ticket reference configuration.
#[derive(Debug, Serialize, Deserialize)]
pub struct Ticket {
    /// Whether the ticket reference is required.
    pub required: bool,
    /// The valid ticket prefixes.
    pub prefixes: Vec<String>,
}

/// Templates.
#[derive(Debug, Serialize, Deserialize)]
pub struct Templates {
    /// The commit message template.
    pub commit: String,
}
