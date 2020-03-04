use std::env;
use std::path::PathBuf;

use anyhow::Result;

use crate::strategy::DeletionStrategy;
use crate::HOME_TRASH;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Refusing to remove '.' or '..', skipping...")]
    CannotTrashDotDirs,
}

/// Throw some files into the trash.
pub fn put(paths: Vec<PathBuf>, _recursive: bool, _force: bool) -> Result<()> {
    let strategy = DeletionStrategy::pick_strategy(&HOME_TRASH);
    for path in paths {
        // don't allow deleting '.' or '..'
        let current_dir = env::current_dir()?;
        ensure!(
            !(path == current_dir
                || (current_dir.parent().is_some() && path == current_dir.parent().unwrap())),
            Error::CannotTrashDotDirs
        );

        if let Err(err) = strategy.delete(path) {
            eprintln!("{}", err);
        }
    }

    Ok(())
}
