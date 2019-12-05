use std::borrow::Cow;
use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::Error;
use libmount::mountinfo::Parser;

use crate::utils;

#[derive(Debug)]
pub struct MountPoint {
    pub mount_id: u64,
    pub parent_id: u64,
    pub major: u64,
    pub minor: u64,
    pub root: PathBuf,
    pub mount_point: PathBuf,
}

fn get_path(s: Cow<OsStr>) -> PathBuf {
    Path::new(&s).to_path_buf()
}

impl From<libmount::mountinfo::MountPoint<'_>> for MountPoint {
    fn from(mp: libmount::mountinfo::MountPoint) -> Self {
        MountPoint {
            mount_id: mp.mount_id,
            parent_id: mp.parent_id,
            major: mp.major,
            minor: mp.minor,
            root: get_path(mp.root),
            mount_point: get_path(mp.mount_point),
        }
    }
}

#[derive(Debug)]
pub struct Mounts(Vec<MountPoint>);

impl Mounts {
    pub fn read() -> Result<Mounts, Error> {
        let pid = unsafe { libc::getpid() };
        let path = Path::new("/")
            .join("proc")
            .join(pid.to_string())
            .join("mountinfo");

        let mut buf = Vec::new();
        {
            let mut file = File::open(path)?;
            file.read_to_end(&mut buf)?;
        }

        let parser = Parser::new(&buf);
        let mut mounts = Vec::new();

        for mp in parser {
            let mp = mp?;
            mounts.push(MountPoint::from(mp));
        }

        Ok(Mounts(mounts))
    }

    pub fn get_mount_point(&self, path: impl AsRef<Path>) -> Option<PathBuf> {
        let path = utils::into_absolute(path).ok()?;

        self.0
            .iter()
            .filter(|mp| path.starts_with(&mp.mount_point))
            .max_by_key(|mp| mp.mount_point.components().count())
            .map(|mp| mp.mount_point.to_path_buf())
    }
}
