use std::fs::{self, File};
use std::path::Path;

use chrono::Local;

use crate::errors::Error;
use crate::trashdir::TrashDir;
use crate::trashinfo::TrashInfo;

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
        "{}.{}.trashinfo",
        elapsed,
        path.file_name().unwrap().to_str().unwrap()
    );

    let trash_file_path = home_trash.files_dir()?.join(&file_name);
    let trash_info_path = home_trash.info_dir()?.join(&file_name);

    let trash_info = TrashInfo {
        path: path.clone(),
        deletion_date: now,
        deleted_path: trash_file_path.clone(),
    };
    {
        let trash_info_file = File::create(trash_info_path)?;
        trash_info.write(&trash_info_file)?;
    }

    fs::rename(path, trash_file_path)?;

    Ok(())
}
