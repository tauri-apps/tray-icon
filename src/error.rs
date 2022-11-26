use thiserror::Error;

/// Errors returned by system_tray.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    OsError(#[from] std::io::Error),
}

/// Convenient type alias of Result type for system_tray.
pub type Result<T> = std::result::Result<T, Error>;
