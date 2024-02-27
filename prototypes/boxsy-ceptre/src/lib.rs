use boxsy::*;

mod json;
mod parse;
mod types;

use crate::json::*;

pub fn generate(path: &std::path::Path) -> Result<Game, Error> {
    let program = Program::from_json(path)?;
    for tp in program.header.types.iter() {
        if let Some(Annote::Synthesis(logics)) = &tp.annote {
            for l in logics.iter() {
                print!("{}, ", l.string());
            }
            println!();
        }
    }
    let game = Game::new();
    Ok(game)
}

#[derive(Debug)]
pub enum Error<'e> {
    Serde(serde_json::Error),
    Io(std::io::Error),
    Parse(nom::Err<nom::error::Error<&'e str>>),
}

impl<'e> std::fmt::Display for Error<'e> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Error::Serde(e) => write!(fmt, "serde error: {e}"),
            Error::Io(e) => write!(fmt, "io error: {e}"),
            Error::Parse(e) => write!(fmt, "parse error: {e}"),
        }
    }
}

impl<'e> std::error::Error for Error<'e> {}
