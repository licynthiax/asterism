#![allow(clippy::upper_case_acronyms, non_camel_case_types)]
use std::collections::BTreeSet;
use std::path::Path;

use serde::de::{Deserializer, Visitor};
use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(Deserialize, Serialize)]
pub struct Program {
    pub builtins: Vec<Builtin>,
    pub header: Header,
    pub bwd_rules: Vec<BwdRule>,
    pub stages: BTreeSet<Stage>,
    pub links: Vec<StageRule>,
    pub init_stage: String,
    pub init_state: Vec<Atom>,
}

#[derive(Deserialize, Serialize)]
pub struct Builtin {
    name: String,
    builtin: BuiltinTypes,
}

#[derive(Serialize, Deserialize)]
pub enum BuiltinTypes {
    NAT,
    NAT_ZERO,
    NAT_SUCC,
}

#[derive(Deserialize, Serialize)]
pub struct Header {
    pub types: Vec<Type>,
    pub preds: Vec<Predicate>,
}

#[derive(Deserialize, Serialize)]
pub struct Type {
    name: String,
    tp: Vec<Tp>,
    pub annote: Option<Annote>,
}

#[derive(Deserialize, Serialize)]
pub struct Tp {
    name: String,
    args: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Predicate {
    name: String,
    terms: Vec<Term>,
    annote: Option<Annote>,
}

#[derive(Deserialize, Serialize)]
pub struct Term(String);

#[derive(Deserialize, Serialize)]
/// this struct purposefully left blank
pub struct BwdRule {}

#[derive(Deserialize, Serialize)]
pub struct Stage {
    name: String,
    nondet: Nondet,
    body: Vec<Rule>,
}

#[derive(Deserialize, Serialize)]
pub enum Nondet {
    Random,
    Interactive,
    Ordered,
}

#[derive(Deserialize, Serialize)]
pub struct Rule {
    name: String,
    pivars: u32,
    lhs: Vec<Atom>,
    rhs: Vec<Atom>,
}

#[derive(Deserialize, Serialize)]
pub struct Atom {
    name: String,
    mode: Mode,
    terms: Vec<Term>,
}

#[derive(Deserialize, Serialize)]
pub enum Mode {
    Pers,
    Lin,
}

pub struct AnnoteString(String);

#[derive(Serialize)]
pub struct Logic(String);

impl Logic {
    pub(crate) fn new(s: &str) -> Self {
        Self(s.to_owned())
    }
    pub(crate) fn string(&self) -> &str {
        &self.0
    }
}

#[derive(Serialize)]
pub enum Annote {
    Query(Vec<Logic>),
    Synthesis(Vec<Logic>),
    Data(Vec<Logic>),
}

impl From<String> for AnnoteString {
    fn from(value: String) -> Self {
        AnnoteString::new(&value)
    }
}

#[derive(Deserialize, Serialize)]
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
                    Err(e) => Err(serde::de::Error::custom(e)),
                }
            }
        }
        deserializer.deserialize_string(AnnoteVisitor)
    }
}
