use std::fs::{self, File};
use std::path::Path;

use chrono::{Duration, Local};

use crate::errors::Error;
use crate::trashdir::TrashDir;
use crate::trashinfo::TrashInfo;

pub fn empty(dry: bool, days: Option<u32>) -> Result<(), Error> {
    let home_trash = TrashDir::get_home_trash();
    let cutoff = if let Some(days) = days {
        Local::now() - Duration::days(days.into())
    } else {
        Local::now()
    };
    for file in home_trash.iter()? {
        let file = file?;

        // ignore files that were deleted after the cutoff (younger)
        let ignore = file.deletion_date > cutoff;

        if !ignore {
            if dry {
                println!("{:?}", file.path);
            } else {
                fs::remove_file(file.info_path)?;
                fs::remove_file(file.deleted_path)?;
            }
        }
    }

    Ok(())
}

pub fn put(path: impl AsRef<Path>, recursive: bool) -> Result<(), Error> {
    let path = path.as_ref().canonicalize()?;
    if path.is_dir() && !recursive {
        error!("cannot trash directories without --recursive");
        return Ok(());
    }

    let now = Local::now();
    let elapsed = now.timestamp_millis();

    let home_trash = TrashDir::get_home_trash();
    let file_name = format!(
        "{}.{}",
        elapsed,
        path.file_name().unwrap().to_str().unwrap()
    );

    let trash_file_path = home_trash.files_dir()?.join(&file_name);
    let trash_info_path = home_trash.info_dir()?.join(file_name + ".trashinfo");

    let trash_info = TrashInfo {
        path: path.clone(),
        deletion_date: now,
        deleted_path: trash_file_path.clone(),
        info_path: trash_info_path.clone(),
    };
    {
        let trash_info_file = File::create(trash_info_path)?;
        trash_info.write(&trash_info_file)?;
    }

    let result = fs::rename(&path, &trash_file_path);
    if result.is_err() {
        fs::copy(&path, &trash_file_path)?;
    }

    Ok(())
}
