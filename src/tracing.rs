// SPDX-FileCopyrightText: 2024 Jean-Philippe Cugnet <jean-philippe@cugnet.eu>
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Utilities to help with tracing.

use crate::helpers::uncapitalise;

/// An extension trait for [`Result`] to insert logging.
pub trait LogResult {
    /// Logs the error.
    ///
    /// If the [`Result`] is an [`Err`], logs the error. Otherwise this function
    /// does nothing.
    fn log_err(self) -> Self;
}

impl<T, E> LogResult for Result<T, E>
where
    E: std::fmt::Display + std::fmt::Debug,
{
    fn log_err(self) -> Self {
        if let Err(error) = &self {
            tracing::error!(?error, "{}", uncapitalise(&error.to_string()));
        }

        self
    }
}
