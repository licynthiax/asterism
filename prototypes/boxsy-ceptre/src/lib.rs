use boxsy::*;

mod boxsy_info;
mod convert;
mod json;
mod parse;

use crate::convert::*;
use crate::json::*;

pub fn generate<'a>(path: std::path::PathBuf) -> Result<Game, Error<'a>> {
    let program = Program::from_json(path)?;
    let ceptre = Ceptre::from_program(program)?;
    let game: Game = ceptre.into();
    Ok(game)
}

#[derive(Debug)]
/// crate-wide error type
pub enum Error<'e> {
    /// command line
    CommandLine(&'e str),
    /// error from reading files
    Io(std::io::Error),
    /// deserialization error
    Serde(serde_json::Error),
    /// error from parsing annotations
    Parse(nom::Err<nom::error::Error<&'e str>>),
    /// ceptre file missing builtins
    BuiltinNotFound(json::BuiltinTypes),
    /// couldn't find a core game type
    TypeNotFound(boxsy_info::GameType),
    /// couldn't find a rule
    RuleNotFound(&'e str),
    /// couldn't find a predicate in the rule?
    PredNotFound(&'e str),
    /// something went wrong with establishing the initial state
    State(&'e str),
    /// anything else
    Custom(&'e str),
}

impl<'e> std::fmt::Display for Error<'e> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Error::CommandLine(e) => write!(fmt, "command line error: {e}"),
            Error::Serde(e) => write!(fmt, "serde error: {e}"),
            Error::Io(e) => write!(fmt, "io error: {e}"),
            Error::Parse(e) => write!(fmt, "parse error: {e}"),
            Error::BuiltinNotFound(b) => write!(fmt, "couldn't find builtin {b}"),
            Error::TypeNotFound(t) => write!(fmt, "unable to map asterism type {t}"),
            Error::RuleNotFound(e) => write!(fmt, "unable to match rule {e}"),
            Error::PredNotFound(e) => write!(fmt, "unable to match predicate {e}"),
            Error::State(e) => write!(fmt, "{e}"),
            Error::Custom(e) => write!(fmt, "{e}"),
        }
    }
}

impl<'e> std::error::Error for Error<'e> {}
