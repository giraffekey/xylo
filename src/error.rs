#[cfg(feature = "no-std")]
use alloc::{
    format,
    string::{String, ToString},
};

#[derive(Debug)]
pub enum Error {
    ParseError,
    NotDigit(String),
    InvalidList,
    InvalidRoot,
    MissingSeed,
    UnknownFunction(String),
    InvalidArgument(String),
    InvalidDefinition(String),
    InvalidCondition,
    InvalidMatch,
    MatchNotFound,
    NotIterable,
    NegativeNumber,
    OutOfBounds,
    NotFound,
    MaxDepthReached,
    PngError(png::EncodingError),
    #[cfg(feature = "std")]
    FileError(std::io::Error),
}

impl ToString for Error {
    fn to_string(&self) -> String {
        match self {
            Error::ParseError => "Could not parse file.".into(),
            Error::NotDigit(name) => format!("Value passed to `{}` was not a digit.", name),
            Error::InvalidList => "Type mismatch in list.".into(),
            Error::InvalidRoot => "The `root` function must return a shape.".into(),
            Error::MissingSeed => "Seed required for rng.".into(),
            Error::UnknownFunction(name) => format!("Unknown function `{}`.", name),
            Error::InvalidArgument(name) => {
                format!("Invalid argument passed to `{}` function.", name)
            }
            Error::InvalidDefinition(name) => {
                format!("Incorrect parameters in `{}` function.", name)
            }
            Error::InvalidCondition => "If condition must reduce to a boolean.".into(),
            Error::InvalidMatch => "Incorrect type comparison in match statement.".into(),
            Error::MatchNotFound => "Not all possibilities covered in match statement".into(),
            Error::NotIterable => "Value is not iterable.".into(),
            Error::NegativeNumber => "Number cannot be negative.".into(),
            Error::OutOfBounds => "Index out of bounds.".into(),
            Error::NotFound => "Value not found.".into(),
            Error::MaxDepthReached => "Max call stack depth reached.".into(),
            Error::PngError(e) => e.to_string(),
            #[cfg(feature = "std")]
            Error::FileError(e) => e.to_string(),
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;
