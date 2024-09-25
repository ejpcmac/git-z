// git-z - A Git extension to go beyond.
// Copyright (C) 2024 Jean-Philippe Cugnet <jean-philippe@cugnet.eu>
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
