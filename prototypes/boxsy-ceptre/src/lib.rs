use boxsy::*;

mod boxsy_info;
mod convert;
mod json;
mod parse;

use crate::convert::*;
use crate::json::*;

pub fn generate(path: &std::path::Path) -> Result<Game, Error> {
    let program = Program::from_json(path)?;
    let _ceptre = Ceptre::try_from(program)?;
    let game = Game::new();
    Ok(game)
}

#[derive(Debug)]
/// crate-wide error type
pub enum Error<'e> {
    /// deserialization error
    Serde(serde_json::Error),
    /// error from reading files
    Io(std::io::Error),
    /// error from parsing annotations
    Parse(nom::Err<nom::error::Error<&'e str>>),
    BuiltinNotFound(json::BuiltinTypes),
    TypeNotFound(boxsy_info::GameType),
}

impl<'e> std::fmt::Display for Error<'e> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Error::Serde(e) => write!(fmt, "serde error: {e}"),
            Error::Io(e) => write!(fmt, "io error: {e}"),
            Error::Parse(e) => write!(fmt, "parse error: {e}"),
            Error::BuiltinNotFound(b) => write!(fmt, "couldn't find builtin {b}"),
            Error::TypeNotFound(t) => write!(fmt, "unable to map asterism type {t}"),
        }
    }
}

impl<'e> std::error::Error for Error<'e> {}
