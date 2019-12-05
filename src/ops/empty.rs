use std::fs;

use anyhow::Error;
use chrono::{Duration, Local};

use crate::TrashDir;
use crate::TrashInfo;

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
