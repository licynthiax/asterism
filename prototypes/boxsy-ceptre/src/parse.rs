use nom::{branch::*, bytes::complete::*, character::complete::*, multi::*, sequence::*};
use nom::{error::Error, IResult, Parser};
use serde::Serialize;

pub(crate) struct AnnoteString(String);

impl AnnoteString {
    pub fn new(s: &str) -> Self {
        Self(s.to_owned())
    }
}

pub(crate) fn parse(s: &AnnoteString) -> IResult<&str, Annote> {
    let s = &s.0;
    let (i, atype) = alt((ws(synthesis), ws(query), ws(data))).parse(s)?;
    let (i, list) = braces(list).parse(i)?;
    let annote = Annote {
        ty: atype,
        logics: list.iter().map(|s| Logic::new(s)).collect(),
    };
    Ok((i, annote))
}

#[derive(Serialize)]
pub(crate) struct Logic(String);

impl Logic {
    pub fn new(s: &str) -> Self {
        Self(s.to_owned())
    }
}

#[derive(Serialize)]
pub struct Annote {
    ty: AnnoteType,
    logics: Vec<Logic>,
}

#[derive(Serialize)]
pub enum AnnoteType {
    Query,
    Synthesis,
    Data,
}

impl std::fmt::Display for AnnoteType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AnnoteType::Query => "AnnoteType::Query",
            AnnoteType::Synthesis => "AnnoteType::Synthesis",
            AnnoteType::Data => "AnnoteType::Data",
        };
        f.write_str(s)
    }
}

fn ws<'a, I, F>(inner: F) -> impl Parser<&'a str, I, Error<&'a str>>
where
    F: Parser<&'a str, I, Error<&'a str>>,
{
    delimited(multispace0, inner, multispace0)
}

fn braces<'a, I, F>(inner: F) -> impl Parser<&'a str, I, Error<&'a str>>
where
    F: Parser<&'a str, I, Error<&'a str>>,
{
    delimited(char('{'), ws(inner), char('}'))
}

fn synthesis(input: &str) -> IResult<&str, AnnoteType> {
    tag_no_case("synthesis")(input).map(|(i, _)| (i, AnnoteType::Synthesis))
}
fn query(input: &str) -> IResult<&str, AnnoteType> {
    tag_no_case("query")(input).map(|(i, _)| (i, AnnoteType::Query))
}
fn data(input: &str) -> IResult<&str, AnnoteType> {
    tag_no_case("data")(input).map(|(i, _)| (i, AnnoteType::Data))
}

fn list(input: &str) -> IResult<&str, Vec<&str>> {
    separated_list0(ws(tag(",")), alpha1).parse(input)
}
