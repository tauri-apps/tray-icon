use thiserror::Error;

/// Errors returned by tray-icon.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    OsError(#[from] std::io::Error),
}

/// Convenient type alias of Result type for tray-icon.
pub type Result<T> = std::result::Result<T, Error>;
