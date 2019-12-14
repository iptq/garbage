use std::env;
use std::fs::{self, File};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::{Duration, Local};

use crate::utils;
use crate::TrashDir;
use crate::TrashInfo;
use crate::{HOME_MOUNT, HOME_TRASH, MOUNTS};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Refusing to remove '.' or '..', skipping...")]
    CannotTrashDotDirs,
}

pub fn put(paths: Vec<PathBuf>, recursive: bool) -> Result<()> {
    let strategy = DeletionStrategy::Copy;
    for path in paths {
        if let Err(err) = strategy.delete(path) {
            eprintln!("{:?}", err);
        }
    }

    Ok(())
}

pub enum DeletionStrategy {
    Copy,
    Topdir,
    TopdirOrCopy,
}

impl DeletionStrategy {
    fn get_target_trash(
        &self,
        mount: impl AsRef<Path>,
        path: impl AsRef<Path>,
    ) -> Option<(TrashDir, bool)> {
        let mount = mount.as_ref();
        let path = path.as_ref();

        // first, are we on the home mount?
        if mount == *HOME_MOUNT {
            return Some((HOME_TRASH.clone(), false));
        }

        // are we just copying?
        if let DeletionStrategy::Copy = self {
            return Some((HOME_TRASH.clone(), true));
        }

        // try to use the $topdir/.Trash directory
        let topdir_trash = mount.join(".Trash");
        if self.should_use_topdir_trash(&topdir_trash) {
            return Some((
                TrashDir(topdir_trash.join(utils::get_uid().to_string())),
                false,
            ));
        }

        // try to use the $topdir/.Trash-$uid directory
        let topdir_trash_uid = mount.join(format!(".Trash-{}", utils::get_uid()));
        if self.should_use_topdir_trash_uid(&topdir_trash_uid) {
            return Some((TrashDir(topdir_trash_uid), false));
        }

        // do we have the copy option
        if let DeletionStrategy::TopdirOrCopy = self {
            return Some((HOME_TRASH.clone(), true));
        }

        None
    }

    fn should_use_topdir_trash(&self, path: impl AsRef<Path>) -> bool {
        let path = path.as_ref();
        if !path.exists() {
            return false;
        }

        let dir = match File::open(path) {
            Ok(file) => file,
            Err(_) => return false,
        };
        let meta = match dir.metadata() {
            Ok(meta) => meta,
            Err(_) => return false,
        };
        if meta.file_type().is_symlink() {
            return false;
        }
        let perms = meta.permissions();

        perms.mode() & 0o1000 > 0
    }

    fn should_use_topdir_trash_uid(&self, path: impl AsRef<Path>) -> bool {
        let path = path.as_ref();
        if !path.exists() {
            match fs::create_dir(path) {
                Ok(_) => (),
                Err(_) => return false,
            };
        }

        return true;
    }

    pub fn delete(&self, target: impl AsRef<Path>) -> Result<()> {
        let target = target.as_ref();

        // don't allow deleting '.' or '..'
        let current_dir = env::current_dir()?;
        ensure!(
            !(target == current_dir
                || (current_dir.parent().is_some() && target == current_dir.parent().unwrap())),
            Error::CannotTrashDotDirs
        );

        let target_mount = MOUNTS
            .get_mount_point(target)
            .ok_or_else(|| anyhow!("couldn't get mount point"))?;
        let (trash_dir, copy) = match self.get_target_trash(target_mount, target) {
            Some(x) => x,
            None => bail!("no trash dir could be selected, u suck"),
        };

        // preparing metadata
        let now = Local::now();
        let elapsed = now.timestamp_millis();
        let file_name = format!(
            "{}.{}",
            elapsed,
            target.file_name().unwrap().to_str().unwrap()
        );

        let trash_file_path = trash_dir.files_dir()?.join(&file_name);
        let trash_info_path = trash_dir.info_dir()?.join(file_name + ".trashinfo");

        let trash_info = TrashInfo {
            path: utils::into_absolute(target)?,
            deletion_date: now,
            deleted_path: trash_file_path.clone(),
            info_path: trash_info_path.clone(),
        };
        {
            let trash_info_file = File::create(trash_info_path)?;
            trash_info.write(&trash_info_file)?;
        }

        // copy the file over
        if copy {
            utils::recursive_copy(&target, &trash_file_path)?;
            fs::remove_dir_all(&target);
        } else {
            fs::rename(&target, &trash_file_path)?;
        }

        Ok(())
    }
}
