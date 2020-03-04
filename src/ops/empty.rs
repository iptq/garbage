use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use chrono::{Duration, Local};

use crate::TrashDir;

/// Options to pass to empty
#[derive(StructOpt)]
pub struct EmptyOptions {
    /// Only list the files that are to be deleted, without
    /// actually deleting anything.
    #[structopt(long = "dry")]
    pub dry: bool,

    /// Delete all files older than (this number) of integer days.
    /// Removes everything if this option is not specified
    #[structopt(long = "days")]
    days: Option<u32>,

    /// The path to the trash directory to empty.
    /// By default, this is your home directory's trash ($XDG_DATA_HOME/Trash)
    #[structopt(long = "trash-dir", parse(from_os_str))]
    trash_dir: Option<PathBuf>,
}

/// Actually delete files in the trash.
pub fn empty(options: EmptyOptions) -> Result<()> {
    let trash_dir = TrashDir::from_opt(options.trash_dir);

    // cutoff date
    let cutoff = if let Some(days) = options.days {
        Local::now() - Duration::days(days.into())
    } else {
        Local::now()
    };

    for file in trash_dir.iter()? {
        let file = file?;

        // ignore files that were deleted after the cutoff (younger)
        let ignore = file.deletion_date > cutoff;

        if !ignore {
            if options.dry {
                println!("{:?}", file.path);
            } else {
                fs::remove_file(file.info_path)?;

                if file.deleted_path.exists() {
                    if file.deleted_path.is_dir() {
                        fs::remove_dir_all(file.deleted_path)?;
                    } else {
                        fs::remove_file(file.deleted_path)?;
                    }
                }
            }
        }
    }

    Ok(())
}
