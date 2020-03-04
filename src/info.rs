use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Local, TimeZone};
use regex::Regex;

use crate::errors::{Error, TrashInfoError};

lazy_static! {
    static ref KEY_VALUE_PATTERN: Regex = Regex::new(r"([A-Za-z]+)\s*=\s*(.*)").unwrap();
}

const DATE_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

/// .trashinfo Data
#[derive(Debug)]
pub struct TrashInfo {
    /// The original path where this file was located before it was deleted.
    pub path: PathBuf,

    /// The date the file was deleted.
    pub deletion_date: DateTime<Local>,

    /// The location of the deleted file after deletion.
    pub deleted_path: PathBuf,

    /// The location of the `info` description file.
    pub info_path: PathBuf,
}

impl TrashInfo {
    /// Create a new TrashInfo based on the .trashinfo path and the deleted file path
    ///
    /// This is useful for reading files from the Trash.
    pub fn from_files(
        info_path: impl AsRef<Path>,
        deleted_path: impl AsRef<Path>,
    ) -> Result<Self, Error> {
        let info_path = info_path.as_ref().to_path_buf();
        let deleted_path = deleted_path.as_ref().to_path_buf();
        let file = File::open(&info_path)?;
        let reader = BufReader::new(file);

        let mut path = None;
        let mut deletion_date = None;

        for (i, line) in reader.lines().enumerate() {
            let line = line?;

            // first line must be "[Trash Info]"
            if i == 0 && line != "[Trash Info]" {
                return Err(Error::BadTrashInfo(TrashInfoError::MissingHeader));
            }

            // look for path and deletion date
            let captures = match KEY_VALUE_PATTERN.captures(&line) {
                Some(captures) => captures,
                None => continue,
            };

            // safe to unwrap because the parser confirmed their existence
            let key = captures.get(1).unwrap().as_str();
            let value = captures.get(2).unwrap().as_str();

            match key {
                "Path" => {
                    let value = PathBuf::from(value);
                    path = Some(value)
                }
                "DeletionDate" => {
                    let date = Local.datetime_from_str(value, DATE_FORMAT)?;
                    deletion_date = Some(date)
                }
                _ => continue,
            }
        }

        let path = match path {
            Some(path) => path,
            None => return Err(Error::BadTrashInfo(TrashInfoError::MissingPath)),
        };

        let deletion_date = match deletion_date {
            Some(deletion_date) => deletion_date,
            None => return Err(Error::BadTrashInfo(TrashInfoError::MissingDate)),
        };

        Ok(TrashInfo {
            path,
            deletion_date,
            deleted_path,
            info_path,
        })
    }

    /// Write the current TrashInfo into a .trashinfo file.
    pub fn write(&self, mut out: impl Write) -> Result<(), io::Error> {
        writeln!(out, "[Trash Info]")?;
        writeln!(out, "Path={}", self.path.to_str().unwrap())?;
        writeln!(
            out,
            "DeletionDate={}",
            self.deletion_date.format(DATE_FORMAT)
        )?;
        Ok(())
    }
}
