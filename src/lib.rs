#[cfg(test)] #[macro_use] extern crate matches;

use std::error;
use std::fmt;
use std::io;
use std::num;

//pub use self::figfont::{FIGchar, FIGfont};
pub use self::figfont::*;
pub use self::wrapper::{Align, Wrapper};
pub use self::smusher::Smusher;
pub use self::smusher::strsmush::CharExt;

mod figfont;
mod wrapper;
mod smusher;

#[derive(Debug)]
pub enum Error {
    FontFormat(&'static str),
    Io(io::Error),
    Parse(num::ParseIntError),
    CodeTag(u32),
    LineFull,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::FontFormat(descr) => write!(f, "{}", descr),
            Error::Io(ref err)       => write!(f, "{}", err),
            Error::Parse(ref err)    => write!(f, "Can't parse value: {}", err),
            Error::CodeTag(tag)      => write!(f, "Invalid code tag: {}", tag),
            Error::LineFull          => write!(f, "Line is full"), 
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::FontFormat(_)  => "Unsupported font format",
            Error::Io(ref err)    => err.description(),
            Error::Parse(ref err) => err.description(),
            Error::CodeTag(_)     => "Invalid code tag",
            Error::LineFull       => "Line full", 
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Io(ref err)    => Some(err),
            Error::Parse(ref err) => Some(err),
            _                     => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<num::ParseIntError> for Error {
    fn from(err: num::ParseIntError) -> Error {
        Error::Parse(err)
    }
}

