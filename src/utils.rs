use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;
use anyhow::Error;

pub fn into_absolute(path: impl AsRef<Path>) -> Result<PathBuf, Error> {
    let path = path.as_ref();

    Ok(if !path.is_absolute() {
        env::current_dir()?.canonicalize()?.join(path)
    } else {
        path.to_path_buf()
    })
}

pub fn get_uid() -> u64 {
    unsafe { libc::getuid().into() }
}

pub fn recursive_copy(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<(), Error> {
    let src = src.as_ref();
    let dst = dst.as_ref();

    for entry in WalkDir::new(src).contents_first(false).follow_links(false).same_file_system(true) {
        let entry = entry?;
        let path = entry.path();
        let relative_path = path.strip_prefix(src)?;

        // this must be the root
        if let None = relative_path.file_name() {
            fs::create_dir(dst);
            continue;
        }

        let target_name = dst.join(relative_path);
        if path.is_dir() {
            fs::create_dir(&target_name);
        } else {
            fs::copy(path, &target_name);
        }
        println!("entry path: {:?}", relative_path);
        println!("> copied to: {:?}", target_name);
    }

    Ok(())
}
