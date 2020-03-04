use std::fs;
use std::path::{Path, PathBuf};

use walkdir::{DirEntry, WalkDir};

use crate::Error;
use crate::TrashInfo;
use crate::XDG;

/// A trash directory represented by a path.
#[derive(Clone, Debug)]
pub struct TrashDir(pub PathBuf);

impl TrashDir {
    /// Gets your user's "home" trash directory.
    ///
    /// According to Trash spec v1.0:
    ///
    /// > For every user2 a “home trash” directory MUST be available.
    /// > Its name and location are $XDG_DATA_HOME/Trash;
    /// > $XDG_DATA_HOME is the base directory for user-specific data, as defined in the Desktop Base Directory Specification.
    pub fn get_home_trash() -> Self {
        TrashDir(XDG.get_data_home().join("Trash"))
    }

    /// Returns the path to this trash directory.
    pub fn path(&self) -> &Path {
        self.0.as_ref()
    }

    pub fn files_dir(&self) -> Result<PathBuf, Error> {
        let target = self.0.join("files");
        if !target.exists() {
            fs::create_dir_all(&target)?;
        }
        Ok(target)
    }

    pub fn info_dir(&self) -> Result<PathBuf, Error> {
        let target = self.0.join("info");
        if !target.exists() {
            fs::create_dir_all(&target)?;
        }
        Ok(target)
    }

    pub fn iter(&self) -> Result<TrashDirIter, Error> {
        let iter = WalkDir::new(&self.info_dir()?)
            .contents_first(true)
            .into_iter()
            .filter_entry(|entry| match entry.path().extension() {
                Some(x) => x == "trashinfo",
                _ => false,
            });
        Ok(TrashDirIter(self.0.clone(), Box::new(iter)))
    }
}

pub struct TrashDirIter(PathBuf, Box<dyn Iterator<Item = walkdir::Result<DirEntry>>>);

impl Iterator for TrashDirIter {
    type Item = Result<TrashInfo, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let entry = {
            let mut entry;
            loop {
                entry = match self.1.next() {
                    Some(Ok(entry)) => entry,
                    Some(Err(err)) => return Some(Err(Error::from(err))),
                    None => return None,
                };
                if entry.path().is_dir() {
                    continue;
                }
                break;
            }
            entry
        };

        let name = entry.path().file_name().unwrap().to_str().unwrap();
        let deleted_path = if !name.ends_with(".trashinfo") {
            return self.next();
        } else {
            self.0
                .join("files")
                .join(name.trim_end_matches(".trashinfo"))
        };
        Some(TrashInfo::from_files(entry.path(), deleted_path).map_err(Error::from))
    }
}
