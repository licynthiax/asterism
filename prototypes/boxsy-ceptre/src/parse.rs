use nom::{branch::*, bytes::complete::*, character::complete::*, multi::*, sequence::*};
use nom::{error::Error, IResult, Parser};

pub(crate) fn parse(s: &AnnoteString) -> IResult<&str, Annote> {
    let s = s.get_str();
    let (i, mut annote) = alt((ws(synthesis), ws(query), ws(data))).parse(s)?;
    let (i, list) = braces(list).parse(i)?;
    match &mut annote {
        Annote::Query(l) => *l = list.iter().map(|s| Logic::new(s)).collect(),
        Annote::Synthesis(l) => *l = list.iter().map(|s| Logic::new(s)).collect(),
        Annote::Data(l) => *l = list.iter().map(|s| Logic::new(s)).collect(),
    }
    Ok((i, annote))
}

use crate::json::{Annote, AnnoteString, Logic};

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

fn synthesis(input: &str) -> IResult<&str, Annote> {
    tag_no_case("synthesis")(input).map(|(i, _)| (i, Annote::Synthesis(Vec::new())))
}
fn query(input: &str) -> IResult<&str, Annote> {
    tag_no_case("query")(input).map(|(i, _)| (i, Annote::Query(Vec::new())))
}
fn data(input: &str) -> IResult<&str, Annote> {
    tag_no_case("data")(input).map(|(i, _)| (i, Annote::Data(Vec::new())))
}

fn list(input: &str) -> IResult<&str, Vec<&str>> {
    separated_list0(ws(tag(",")), alpha1).parse(input)
}
