//! garbage

#![warn(missing_docs)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate structopt;
extern crate log;
#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate thiserror;

mod dir;
mod errors;
mod info;
mod mounts;
pub mod ops;
mod utils;

use std::path::PathBuf;

use xdg::BaseDirectories;

pub use crate::dir::TrashDir;
pub use crate::errors::Error;
pub use crate::info::TrashInfo;
use crate::mounts::Mounts;

lazy_static! {
    #[allow(missing_docs)]
    pub static ref MOUNTS: Mounts = Mounts::read().unwrap();
    static ref XDG: BaseDirectories = BaseDirectories::new().unwrap();
    static ref HOME_TRASH: TrashDir = TrashDir::get_home_trash();
    static ref HOME_MOUNT: PathBuf = MOUNTS.get_mount_point(HOME_TRASH.path()).unwrap();
}
