#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    WalkDir(walkdir::Error),
    BadTrashInfo(TrashInfoError),
    ParseDate(chrono::format::ParseError),
}

#[derive(Debug)]
pub enum TrashInfoError {
    MissingHeader,
    MissingPath,
    MissingDate,
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<walkdir::Error> for Error {
    fn from(err: walkdir::Error) -> Self {
        Error::WalkDir(err)
    }
}

impl From<chrono::format::ParseError> for Error {
    fn from(err: chrono::format::ParseError) -> Self {
        Error::ParseDate(err)
    }
}
