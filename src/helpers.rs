// SPDX-FileCopyrightText: 2023, 2025 Jean-Philippe Cugnet <jean-philippe@cugnet.eu>
// SPDX-License-Identifier: GPL-3.0-only

//! General helpers.

/// Uncapitalises the first character in s.
pub fn uncapitalise(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_lowercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod test {
    #![expect(clippy::missing_panics_doc, reason = "tests")]

    use super::*;

    #[test]
    fn uncapitalise_sets_the_first_char_to_lowercase() {
        assert_eq!(uncapitalise("Hello"), "hello");
    }

    #[test]
    fn uncapitalise_returns_an_empty_string_on_empty_string() {
        assert_eq!(uncapitalise(""), "");
    }
}
