use std::fs;
use std::io;
use std::path::PathBuf;

use anyhow::Result;

use crate::TrashDir;

/// Options to pass to restore
#[derive(StructOpt)]
pub struct RestoreOptions {
    /// The path to the trash directory to restore from.
    /// By default, this is your home directory's trash ($XDG_DATA_HOME/Trash)
    #[structopt(long = "trash-dir", parse(from_os_str))]
    trash_dir: Option<PathBuf>,
}

/// Restore files from a trash directory
pub fn restore(options: RestoreOptions) -> Result<()> {
    let trash_dir = TrashDir::from_opt(options.trash_dir);

    if trash_dir.check_info_dir()?.is_none() {
        bail!("There's no trash directory here.");
    }

    // get list of files sorted by deletion date
    // TODO: possible to get this to be streaming?
    let files = {
        let mut files = trash_dir
            .iter()
            .unwrap()
            .filter_map(|entry| match entry {
                Ok(info) => Some(info),
                Err(err) => {
                    eprintln!("failed to get file info: {:?}", err);
                    None
                }
            })
            .collect::<Vec<_>>();
        files.sort_unstable_by_key(|info| info.deletion_date);
        files
    };

    if files.len() == 0 {
        bail!("No files in this trash directory.");
    }

    for (i, info) in files.iter().enumerate() {
        println!(
            "[{}]\t{}\t{}",
            i,
            info.deletion_date,
            info.path.to_str().unwrap()
        );
    }

    let stdin = io::stdin();
    let mut s = String::new();
    eprintln!("which file to restore? [0..{}]", files.len() - 1);
    stdin.read_line(&mut s).unwrap();

    match s.trim_end().parse::<usize>() {
        Ok(i) if i < files.len() => {
            let info = &files[i]; // should never fail since we just checked
            eprintln!("moving {:?} to {:?}", &info.deleted_path, &info.path);
            fs::remove_file(&info.info_path)?;
            fs::rename(&info.deleted_path, &info.path)?;
        }
        _ => eprintln!("Invalid number."),
    }
    Ok(())
}
