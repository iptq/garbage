#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

mod errors;
mod ops;
mod trashdir;
mod trashinfo;

use std::fs;
use std::io;
use std::path::PathBuf;

use structopt::StructOpt;
use xdg::BaseDirectories;

use crate::trashdir::TrashDir;

lazy_static! {
    static ref XDG: BaseDirectories = BaseDirectories::new().unwrap();
}

#[derive(StructOpt)]
enum Command {
    #[structopt(name = "empty")]
    Empty,

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

        /// -f to stay compatible with GNU rm
        #[structopt(long = "force", short = "f")]
        force: bool,
    },

    #[structopt(name = "restore")]
    Restore,
}

fn main() {
    env_logger::init();

    let cmd = Command::from_args();
    match cmd {
        Command::Empty => {
            println!("TODO");
        }
        Command::List => {
            let home_trash = TrashDir::get_home_trash();
            for info in home_trash.iter() {
                let info = match info {
                    Ok(info) => info,
                    Err(err) => {
                        warn!("failed to get file info: {:?}", err);
                        continue;
                    }
                };
                println!("{}\t{}", info.deletion_date, info.path.to_str().unwrap());
            }
        }
        Command::Put { paths, recursive } => {
            for path in paths {
                match crate::ops::put(path, recursive) {
                    Ok(_) => (),
                    Err(err) => error!("error: {:?}", err),
                }
            }
        }
        Command::Restore => {
            let home_trash = TrashDir::get_home_trash();
            let files = home_trash
                .iter()
                .filter_map(|entry| match entry {
                    Ok(info) => Some(info),
                    Err(err) => {
                        warn!("failed to get file info: {:?}", err);
                        None
                    }
                })
                .collect::<Vec<_>>();
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
            println!("which file to restore? [0..{}]", files.len());
            stdin.read_line(&mut s).unwrap();

            match s.trim_end().parse::<usize>() {
                Ok(i) if i < files.len() => {
                    let info = files.get(i).unwrap();
                    fs::rename(&info.deleted_path, &info.path);
                }
                _ => println!("Invalid number."),
            }
        }
    }
}
