#![allow(clippy::upper_case_acronyms, non_camel_case_types)]
use std::collections::BTreeSet;

use boxsy::*;
use serde::{Deserialize, Serialize};

pub mod json;
pub mod parse;

use crate::parse::{Annote, AnnoteString};

pub fn generate(path: &std::path::Path) -> Result<Game, Error> {
    let _program = Program::from_json(path)?;
    todo!()
    // Ok(())
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

#[derive(Deserialize, Serialize)]
pub struct Program {
    builtins: Vec<Builtin>,
    header: Header,
    bwd_rules: Vec<BwdRule>,
    stages: BTreeSet<Stage>,
    links: Vec<StageRule>,
    init_stage: String,
    init_state: Vec<Atom>,
}

#[derive(Deserialize, Serialize)]
struct Builtin {
    name: String,
    builtin: BuiltinTypes,
}

#[derive(Serialize, Deserialize)]
enum BuiltinTypes {
    NAT,
    NAT_ZERO,
    NAT_SUCC,
}

#[derive(Deserialize, Serialize)]
struct Header {
    types: Vec<Type>,
    preds: Vec<Predicate>,
}

#[derive(Deserialize, Serialize)]
struct Type {
    name: String,
    tp: Vec<Tp>,
    annote: Option<Annote>,
}

#[derive(Deserialize, Serialize)]
struct Tp {
    name: String,
    args: Vec<String>,
}

#[derive(Deserialize, Serialize)]
struct Predicate {
    name: String,
    terms: Vec<Term>,
    annote: Option<Annote>,
}

#[derive(Deserialize, Serialize)]
struct Term(String);

#[derive(Deserialize, Serialize)]
// this struct purposefully left blank
struct BwdRule {}

#[derive(Deserialize, Serialize)]
struct Stage {
    name: String,
    nondet: Nondet,
    body: Vec<Rule>,
}

#[derive(Deserialize, Serialize)]
enum Nondet {
    Random,
    Interactive,
    Ordered,
}

#[derive(Deserialize, Serialize)]
struct Rule {
    name: String,
    pivars: u32,
    lhs: Vec<Atom>,
    rhs: Vec<Atom>,
}

#[derive(Deserialize, Serialize)]
struct Atom {
    name: String,
    mode: Mode,
    terms: Vec<Term>,
}

#[derive(Deserialize, Serialize)]
enum Mode {
    Pers,
    Lin,
}

impl From<String> for AnnoteString {
    fn from(value: String) -> Self {
        AnnoteString::new(&value)
    }
}

#[derive(Deserialize, Serialize)]
struct StageRule {
    name: String,
    pivars: u32,
    pre_stage: String,
    post_stage: String,
    lhs: Vec<Atom>,
    rhs: Vec<Atom>,
}

impl PartialEq for Stage {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl Eq for Stage {}

impl Ord for Stage {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for Stage {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
