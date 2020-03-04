#![deny(warnings)]

extern crate anyhow;

use std::fs;
use std::io;
use std::path::PathBuf;

use anyhow::Result;
use garbage::{
    ops::{self, EmptyOptions},
    TrashDir,
};
use structopt::StructOpt;

#[derive(StructOpt)]
enum Command {
    #[structopt(name = "empty")]
    Empty(EmptyOptions),

    #[structopt(name = "list")]
    List,

    #[structopt(name = "put")]
    Put {
        /// The target path to be trashed
        #[structopt(parse(from_os_str))]
        paths: Vec<PathBuf>,

        /// Trashes directories recursively
        #[structopt(long = "recursive", short = "r")]
        recursive: bool,

        /// Suppress prompts/messages
        #[structopt(long = "force", short = "f")]
        force: bool,
    },

    #[structopt(name = "restore")]
    Restore,
}

fn run() -> Result<()> {
    env_logger::init();

    let cmd = Command::from_args();
    match cmd {
        Command::Empty(options) => ops::empty(options),
        Command::List => {
            ops::list();
            Ok(())
        }
        Command::Put {
            paths,
            recursive,
            force,
        } => ops::put(paths, recursive, force),
        Command::Restore => {
            let home_trash = TrashDir::get_home_trash();
            let mut files = home_trash
                .iter()
                .unwrap()
                .filter_map(|entry| match entry {
                    Ok(info) => Some(info),
                    Err(err) => {
                        eprintln!("failed to get file info: {:?}", err);
                        None
                    }
                })
                .collect::<Vec<_>>();
            files.sort_unstable_by_key(|info| info.deletion_date);
            for (i, info) in files.iter().enumerate() {
                println!(
                    "[{}]\t{}\t{}",
                    i,
                    info.deletion_date,
                    info.path.to_str().unwrap()
                );
            }

            let stdin = io::stdin();
            let mut s = String::new();
            println!("which file to restore? [0..{}]", files.len() - 1);
            stdin.read_line(&mut s).unwrap();

            match s.trim_end().parse::<usize>() {
                Ok(i) if i < files.len() => {
                    let info = files.get(i).unwrap();
                    println!("moving {:?} to {:?}", &info.deleted_path, &info.path);
                    fs::rename(&info.deleted_path, &info.path)?;
                }
                _ => println!("Invalid number."),
            }
            Ok(())
        }
    }
}

fn main() {
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
