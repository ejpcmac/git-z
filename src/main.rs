// SPDX-FileCopyrightText: 2023 Jean-Philippe Cugnet <jean-philippe@cugnet.eu>
// SPDX-License-Identifier: GPL-3.0-only

//! A Git extension to go beyond.

use eyre::Result;

use git_z::GitZ;

fn main() -> Result<()> {
    color_eyre::install()?;
    GitZ::run()
}
