#[macro_use]
extern crate anyhow;

use std::fs;
use std::io;
use std::path::PathBuf;

use anyhow::Error;
use structopt::StructOpt;

use garbage::*;

#[derive(StructOpt)]
enum Command {
    #[structopt(name = "empty")]
    Empty {
        /// Only list the files that are to be deleted, without
        /// actually deleting anything.
        #[structopt(long = "dry")]
        dry: bool,

        /// Delete all files older than (this number) of days.
        /// Removes everything if this option is not specified
        days: Option<u32>,
    },

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

fn main() -> Result<(), Error> {
    env_logger::init();

    let cmd = Command::from_args();
    match cmd {
        Command::Empty { dry, days } => match ops::empty(dry, days) {
            Ok(_) => (),
            Err(err) => eprintln!("error: {:?}", err),
        },
        Command::List => {
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
            for info in files {
                println!("{}\t{}", info.deletion_date, info.path.to_str().unwrap());
            }
        }
        Command::Put {
            paths, recursive, force
        } => {
            ops::put(paths, recursive);
        }
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
        }
    }

    Ok(())
}
