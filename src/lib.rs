use std::fmt;

pub use xflags_macros::{args, args_parser};

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct Error {
    msg: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.msg, f)
    }
}

impl std::error::Error for Error {}

impl Error {
    pub fn new(message: impl Into<String>) -> Error {
        Error { msg: message.into() }
    }
}

/// Private impl details for macros.
#[doc(hidden)]
pub mod rt;
