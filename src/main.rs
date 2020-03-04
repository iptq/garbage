#![deny(warnings)]

extern crate anyhow;

use anyhow::Result;
use garbage::ops::{self, EmptyOptions, ListOptions, PutOptions, RestoreOptions};
use structopt::StructOpt;

#[derive(StructOpt)]
enum Command {
    /// Empty a trash directory.
    #[structopt(name = "empty")]
    Empty(EmptyOptions),

    /// List the contents of a trash directory.
    #[structopt(name = "list")]
    List(ListOptions),

    /// Puts files into the trash.
    ///
    /// If a trash directory isn't specified, the best strategy is picked
    /// for each file that's deleted (after shell glob expansion). The algorithm
    /// for deciding a strategy is specified in the FreeDesktop Trash spec.
    #[structopt(name = "put")]
    Put(PutOptions),

    /// Restores files from the trash.
    #[structopt(name = "restore")]
    Restore(RestoreOptions),
}

fn run() -> Result<()> {
    let cmd = Command::from_args();
    match cmd {
        Command::Empty(options) => ops::empty(options),
        Command::List(options) => ops::list(options),
        Command::Put(options) => ops::put(options),
        Command::Restore(options) => ops::restore(options),
    }
}

fn main() {
    env_logger::init();

    match run() {
        Ok(_) => (),
        Err(err) => {
            eprintln!("Error: {:?}", err);
            for cause in err.chain() {
                eprintln!("- {:?}", cause);
            }
        }
    }
}
