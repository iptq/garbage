/// All errors that could happen
#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Walkdir error: {0}")]
    WalkDir(#[from] walkdir::Error),
    #[error("Bad .trashinfo file: {0}")]
    BadTrashInfo(#[from] TrashInfoError),
    #[error("Date parsing error: {0}")]
    ParseDate(#[from] chrono::format::ParseError),
}

/// Errors related to .trashinfo files
#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum TrashInfoError {
    #[error("Missing [TrashInfo] header")]
    MissingHeader,
    #[error("Missing path attribute")]
    MissingPath,
    #[error("Missing date attribute")]
    MissingDate,
}
