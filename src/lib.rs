// SPDX-FileCopyrightText: 2023-2024 Jean-Philippe Cugnet <jean-philippe@cugnet.eu>
// SPDX-License-Identifier: GPL-3.0-only

//! A Git extension to go beyond.

mod command;
mod commit_cache;
mod config;
mod helpers;
mod tracing;

#[doc(hidden)]
pub use command::GitZ;
