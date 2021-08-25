use std::{self, fmt, io};

use crate::tty;

#[derive(Debug)]
pub enum Error {
    Pty(tty::ForkError),
    Io(io::Error),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::Pty(ref err) => Some(err),
            Error::Io(ref err) => Some(err),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "pty-shell error")
    }
}

impl From<tty::ForkError> for Error {
    fn from(err: tty::ForkError) -> Error {
        Error::Pty(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}
