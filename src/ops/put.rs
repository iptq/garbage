use std::env;
use std::fs::{self, File};
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::Local;

use crate::utils;
use crate::{TrashDir, TrashInfo};
use crate::{HOME_MOUNT, MOUNTS};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Refusing to remove directory {0} without '-r' option")]
    MissingRecursiveOption(PathBuf),

    #[error("Refusing to remove '.' or '..', skipping...")]
    CannotTrashDotDirs,

    #[error("Cancelled by user.")]
    CancelledByUser,
}

/// Options to pass to put
#[derive(StructOpt)]
pub struct PutOptions {
    /// The target path to be trashed
    #[structopt(parse(from_os_str))]
    paths: Vec<PathBuf>,

    /// Don't actually move anything, just print the files to be removed
    #[structopt(long = "dry")]
    dry: bool,

    /// Prompt before every removal
    #[structopt(long = "prompt", short = "i")]
    prompt: bool,

    /// Trashes directories recursively (ignored)
    #[structopt(long = "recursive", short = "r")]
    _recursive: bool,

    /// Suppress prompts/messages
    #[structopt(long = "force", short = "f")]
    force: bool,

    /// Put all the trashed files into this trash directory
    /// regardless of what filesystem is on.
    ///
    /// If a copy is required to copy the file, a prompt will be raised,
    /// which can be bypassed by passing --force.
    ///
    /// If this option is not passed, the best strategy will be chosen
    /// automatically for each file.
    #[structopt(long = "trash-dir", parse(from_os_str))]
    trash_dir: Option<PathBuf>,
}

/// Throw some files into the trash.
pub fn put(options: PutOptions) -> Result<()> {
    for path in options.paths.iter() {
        // don't allow deleting '.' or '..'
        let current_dir = env::current_dir()?;
        ensure!(
            !(utils::into_absolute(&path)? == current_dir.as_path()
                || (current_dir.parent().is_some()
                    && utils::into_absolute(&path)? == current_dir.parent().unwrap())),
            Error::CannotTrashDotDirs
        );

        // pick the best strategy for deleting this particular file
        let strategy = if let Some(ref trash_dir) = options.trash_dir {
            DeletionStrategy::Fixed(TrashDir::from(trash_dir))
        } else {
            DeletionStrategy::pick_strategy(path)?
        };
        // println!("Strategy: {:?}", strategy);

        if options.dry {
            eprintln!("Dry-deleting: {}", path.to_str().unwrap());
        } else if let Err(err) = strategy.delete(path, &options) {
            eprintln!("{}", err);
        }
    }

    Ok(())
}

/// DeletionStrategy describes a strategy by which a file is deleted
#[derive(Debug)]
enum DeletionStrategy {
    /// move or copy the file to this particular trash
    Fixed(TrashDir),

    /// move the candidate files/directories to the trash directory
    /// (this requires that both the candidate and the trash directories be on the same filesystem)
    MoveTo(TrashDir),

    /// recursively copy the candidate files/directories to the trash directory
    CopyTo(TrashDir),
}

impl DeletionStrategy {
    /// This method picks the ideal strategy
    pub fn pick_strategy(target: impl AsRef<Path>) -> Result<DeletionStrategy> {
        let target = target.as_ref();
        let target_mount = MOUNTS
            .get_mount_point(target)
            .ok_or_else(|| anyhow!("couldn't get mount point"))?;

        // first, are we on the home mount?
        if target_mount == *HOME_MOUNT {
            return Ok(DeletionStrategy::MoveTo(TrashDir::get_home_trash()));
        }

        // try to use the $topdir/.Trash directory
        if should_use_topdir_trash(&target_mount) {
            let topdir_trash_dir = target_mount
                .join(".Trash")
                .join(utils::get_uid().to_string());
            let trash_dir = TrashDir::from(topdir_trash_dir);
            trash_dir.create()?;
            return Ok(DeletionStrategy::MoveTo(trash_dir));
        }

        // try to use the $topdir/.Trash-$uid directory
        if should_use_topdir_trash_uid(&target_mount) {
            let topdir_trash_uid = target_mount.join(format!(".Trash-{}", utils::get_uid()));
            let trash_dir = TrashDir::from(topdir_trash_uid);
            trash_dir.create()?;
            return Ok(DeletionStrategy::MoveTo(trash_dir));
        }

        // it's not on the home mount, but we'll copy into it anyway
        Ok(DeletionStrategy::CopyTo(TrashDir::get_home_trash()))
    }

    fn get_target_trash(&self) -> (&TrashDir, bool) {
        match self {
            DeletionStrategy::Fixed(trash) => {
                // TODO: finish
                (trash, true)
            }
            DeletionStrategy::MoveTo(trash) => (trash, false),
            DeletionStrategy::CopyTo(trash) => (trash, true),
        }
    }

    /// The actual deletion happens here
    pub fn delete(&self, target: impl AsRef<Path>, options: &PutOptions) -> Result<()> {
        let target = target.as_ref();

        // this will be None if target isn't a symlink
        let link_info = target.read_link().ok();

        // file is a directory
        // if !link_info.is_some() && target.is_dir() && !options.recursive {
        //     bail!(Error::MissingRecursiveOption(target.to_path_buf()));
        // }

        let (trash_dir, requires_copy) = self.get_target_trash();

        // prompt if not suppressed
        // TODO: streamline this logic better
        if !options.force && (requires_copy || options.prompt) {
            // TODO: actually handle prompting instead of manually flushing
            if requires_copy {
                eprint!(
                    "Removing file '{}' requires potentially expensive copying. Continue? [Y/n] ",
                    target.to_str().unwrap()
                );
            } else if options.prompt {
                eprint!("Remove file '{}'? [Y/n] ", target.to_str().unwrap());
            }
            io::stderr().flush()?;

            let should_continue = loop {
                let stdin = io::stdin();
                let mut s = String::new();
                stdin.read_line(&mut s).unwrap();
                match s.trim().to_lowercase().as_str() {
                    "yes" | "y" => break true,
                    "no" | "n" => break false,
                    _ => {
                        eprint!("Invalid response. Please type yes or no: ");
                    }
                }
            };
            if !should_continue {
                bail!(Error::CancelledByUser);
            }
        }

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
        if requires_copy {
            utils::recursive_copy(&target, &trash_file_path)?;
            fs::remove_dir_all(&target)?;
        } else {
            fs::rename(&target, &trash_file_path)?;
        }

        Ok(())
    }
}

/// Can we use $topdir/.Trash?
///
/// 1. If it doesn't exist, don't create it.
/// 2. All users should be able to write to it
/// 3. It must have sticky-bit permissions if the filesystem supports it.
/// 4. The directory must not be a symbolic link.
fn should_use_topdir_trash(mount: impl AsRef<Path>) -> bool {
    let mount = mount.as_ref();
    let trash_dir = mount.join(".Trash");

    if !trash_dir.exists() {
        return false;
    }

    let dir = match File::open(trash_dir) {
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

/// Can we use $topdir/.Trash-uid?
fn should_use_topdir_trash_uid(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();
    if !path.exists() {
        match fs::create_dir(path) {
            Ok(_) => (),
            Err(_) => return false,
        };
    }
    return true;
}
