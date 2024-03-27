#![allow(clippy::upper_case_acronyms, non_camel_case_types, unused)]
/// types for JSON deserialization
use std::collections::BTreeSet;
use std::path::Path;

use serde::de::{Deserializer, Visitor};
use serde::Deserialize;

use crate::boxsy_info::Logic;
use crate::Error;

#[derive(Deserialize)]
pub struct Program {
    pub builtins: Vec<Builtin>,
    pub header: Header,
    pub bwd_rules: Vec<BwdRule>,
    pub stages: BTreeSet<Stage>,
    pub links: Vec<StageRule>,
    pub init_stage: String,
    pub init_state: Vec<Atom>,
}

#[derive(Deserialize, Clone)]
pub struct Builtin {
    pub name: String,
    pub builtin: BuiltinTypes,
}

#[derive(Deserialize, Eq, PartialEq, Debug, Clone, Copy)]
pub enum BuiltinTypes {
    NAT,
    NAT_ZERO,
    NAT_SUCC,
}

impl std::fmt::Display for BuiltinTypes {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            BuiltinTypes::NAT => fmt.write_str("BuiltinTypes::NAT"),
            BuiltinTypes::NAT_ZERO => fmt.write_str("BuiltinTypes::NAT_ZERO"),
            BuiltinTypes::NAT_SUCC => fmt.write_str("BuiltinTypes::NAT_SUCC"),
        }
    }
}

#[derive(Deserialize)]
pub struct Header {
    pub types: Vec<Type>,
    pub preds: Vec<Predicate>,
}

#[derive(Deserialize, Clone)]
pub struct Type {
    pub name: String,
    pub tp: Vec<Tp>,
    pub annote: Option<Annote>,
}

#[derive(Deserialize, Clone)]
pub struct Tp {
    pub name: String,
    pub args: Vec<String>,
}

#[derive(Deserialize, Ord, PartialOrd, Eq, PartialEq, Clone, Debug)]
pub struct Predicate {
    pub name: String,
    pub terms: Vec<Term>,
    pub annote: Option<Annote>,
}

#[derive(Deserialize, Eq, PartialEq, Ord, PartialOrd, Debug, Clone)]
pub struct Term(String);
impl Term {
    pub fn str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Deserialize)]
/// this struct purposefully left blank
pub struct BwdRule {}

#[derive(Deserialize)]
pub struct Stage {
    name: String,
    nondet: Nondet,
    pub body: Vec<Rule>,
}

#[derive(Deserialize, Copy, Clone)]
pub enum Nondet {
    Random,
    Interactive,
    Ordered,
}

#[derive(Deserialize, Clone)]
pub struct Rule {
    pub name: String,
    pivars: u32,
    pub lhs: Vec<Atom>,
    pub rhs: Vec<Atom>,
    pub annote: Option<Annote>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Atom {
    pub name: String,
    mode: Mode,
    pub terms: Vec<Term>,
}

#[derive(Deserialize, Copy, Clone, Debug)]
pub enum Mode {
    Pers,
    Lin,
}

pub struct AnnoteString(String);

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Debug)]
pub enum Annote {
    /// syntheses describe _structural syntheses_-- _which_ logics a type is associated with.
    Synthesis(BTreeSet<Logic>),
    /// data mark different instantiations of syntheses
    Data(BTreeSet<Logic>),
    /// queries check relationships between logics-- while syntheses define what those
    /// relationships _are_, queries maintain them at runtime.
    Query(BTreeSet<Logic>),
    /// integrations mark the ways one logic might change in reaction to another
    Integration(BTreeSet<Logic>),
}

impl Annote {
    pub fn get_logics_mut(&mut self) -> &mut BTreeSet<Logic> {
        match self {
            Annote::Integration(l) => l,
            Annote::Synthesis(l) => l,
            Annote::Query(l) => l,
            Annote::Data(l) => l,
        }
    }
    pub fn get_logics(&self) -> &BTreeSet<Logic> {
        match self {
            Annote::Integration(l) => l,
            Annote::Synthesis(l) => l,
            Annote::Query(l) => l,
            Annote::Data(l) => l,
        }
    }
}

impl From<String> for AnnoteString {
    fn from(value: String) -> Self {
        AnnoteString::new(&value)
    }
}

#[derive(Deserialize)]
pub struct StageRule {
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

impl AnnoteString {
    pub(crate) fn new(s: &str) -> Self {
        Self(s.to_owned())
    }

    pub(crate) fn get_str(&self) -> &str {
        &self.0
    }
}

impl Program {
    pub fn from_json(file: &Path) -> Result<Self, Error> {
        let json = std::fs::read_to_string(file).map_err(Error::Io)?;
        serde_json::from_str(&json).map_err(Error::Serde)
    }
}

impl<'de> Deserialize<'de> for Annote {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct AnnoteVisitor;

        impl<'de> Visitor<'de> for AnnoteVisitor {
            type Value = Annote;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string containing a parseable annotation")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let a_s = AnnoteString::new(s);
                let a = crate::parse::parse(&a_s);
                match a {
                    Ok((_, annote)) => Ok(annote),
                    Err(e) => Err(serde::de::Error::custom(Error::Parse(e))),
                }
            }
        }
        deserializer.deserialize_string(AnnoteVisitor)
    }
}
