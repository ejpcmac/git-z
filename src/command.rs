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

mod commit;
mod hello;
mod helpers;

use clap::Parser;
use eyre::Result;

use self::{commit::Commit, hello::Hello};

/// A Git extension to go beyond.
#[derive(Debug, Parser)]
#[command(author, version)]
pub enum GitZ {
    /// Say hello.
    Hello(Hello),
    /// Runs the commit wizard.
    Commit(Commit),
}

trait Command {
    /// Runs the command.
    fn run(&self) -> Result<()>;
}

impl GitZ {
    /// Runs git-z.
    pub fn run() -> Result<()> {
        let result = match Self::parse() {
            Self::Hello(hello) => hello.run(),
            Self::Commit(commit) => commit.run(),
        };

        match result {
            Err(e) => handle_errors(e),
            Ok(()) => Ok(()),
        }
    }
}

fn handle_errors(e: color_eyre::Report) -> Result<()> {
    // if let Some(e) = e.downcast_ref::<ErrorType>() {
    //     match e {
    //         ErrorType::ErrorKind => {
    //             error!("{e}");
    //             hint!("Some help message.");
    //         }
    //     }
    //     std::process::exit(1);
    // } else {
    Err(e)
    // }
}
