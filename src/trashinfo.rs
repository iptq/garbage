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

#[derive(Debug)]
pub struct TrashInfo {
    pub path: PathBuf,
    pub deletion_date: DateTime<Local>,
    pub deleted_path: PathBuf,
}

impl TrashInfo {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        let original_path = path.to_path_buf();
        let file = File::open(path)?;
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
            deleted_path: original_path,
        })
    }

    pub fn write(&self, mut out: impl Write) -> Result<(), io::Error> {
        out.write(b"[Trash Info]\n")?;
        write!(out, "Path={}\n", self.path.to_str().unwrap())?;
        write!(
            out,
            "DeletionDate={}\n",
            self.deletion_date.format(DATE_FORMAT)
        )?;
        Ok(())
    }
}
