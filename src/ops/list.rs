use std::path::PathBuf;

use anyhow::Result;

use crate::TrashDir;

#[derive(StructOpt)]
pub struct ListOptions {
    /// The path to the trash directory to list.
    /// By default, this is your home directory's trash ($XDG_DATA_HOME/Trash)
    #[structopt(long = "trash-dir", parse(from_os_str))]
    trash_dir: Option<PathBuf>,
}

pub fn list(options: ListOptions) -> Result<()> {
    let trash_dir = TrashDir::from_opt(options.trash_dir);

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
    for info in files {
        println!("{}\t{}", info.deletion_date, info.path.to_str().unwrap());
    }

    Ok(())
}
