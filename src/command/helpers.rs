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

/// Prints a success.
#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {{
        use colored::Colorize;
        let message = format!($($arg)*).green().bold();
        println!("{message}");
    }};
}

/// Prints an error.
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        use colored::Colorize;
        let message = format!($($arg)*).red().bold();
        println!("{message}");
    }};
}

/// Prints a hint.
#[macro_export]
macro_rules! hint {
    ($($arg:tt)*) => {{
        use colored::Colorize;
        let message = format!($($arg)*).blue();
        println!("{message}");
    }};
}
