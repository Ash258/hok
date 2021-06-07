use std::fmt;
use std::fmt::Debug;
use std::io;
use std::result;

/// A type alias for `Result<T, scoop::Error>`.
pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub struct Error(ErrorKind);

#[derive(Debug)]
pub enum ErrorKind {
    Io(io::Error),
    SerdeJson(serde_json::Error),
    /// Hints that destructuring should not be exhaustive.
    ///
    /// This enum may grow additional variants, so this makes sure clients
    /// don't count on exhaustive matching. (Otherwise, adding a new variant
    /// could break existing code.)
    #[doc(hidden)]
    __Nonexhaustive,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error(ErrorKind::Io(err))
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error(ErrorKind::SerdeJson(err))
    }
}
