/// parsing the annotations for the types in the json files
use std::collections::BTreeSet;

use nom::{branch::*, bytes::complete::*, character::complete::*, multi::*, sequence::*};
use nom::{error::Error, IResult, Parser};

use crate::boxsy_info::Logic;

pub(crate) fn parse(s: &AnnoteString) -> IResult<&str, Annote> {
    let s = s.get_str();
    let (i, mut annote) = alt((ws(synthesis), ws(integration), ws(query), ws(data))).parse(s)?;
    let (i, list) = braces(list).parse(i)?;
    let l = annote.get_logics_mut();
    *l = list;
    Ok((i, annote))
}

use crate::json::{Annote, AnnoteString};

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
    tag_no_case("synthesis")(input).map(|(i, _)| (i, Annote::Synthesis(BTreeSet::new())))
}
fn integration(input: &str) -> IResult<&str, Annote> {
    tag_no_case("integration")(input).map(|(i, _)| (i, Annote::Integration(BTreeSet::new())))
}
fn query(input: &str) -> IResult<&str, Annote> {
    tag_no_case("query")(input).map(|(i, _)| (i, Annote::Query(BTreeSet::new())))
}
fn data(input: &str) -> IResult<&str, Annote> {
    tag_no_case("data")(input).map(|(i, _)| (i, Annote::Data(BTreeSet::new())))
}

fn list(input: &str) -> IResult<&str, BTreeSet<Logic>> {
    let list = separated_list0(ws(tag(",")), list_item).parse(input)?;
    Ok((list.0, BTreeSet::from_iter(list.1)))
}

fn list_item(input: &str) -> IResult<&str, Logic> {
    alt((collision, control, linking, resource))(input)
}

fn collision(input: &str) -> IResult<&str, Logic> {
    tag_no_case("collision")(input).map(|(i, _)| (i, Logic::Collision))
}
fn control(input: &str) -> IResult<&str, Logic> {
    tag_no_case("control")(input).map(|(i, _)| (i, Logic::Control))
}
fn resource(input: &str) -> IResult<&str, Logic> {
    tag_no_case("resource")(input).map(|(i, _)| (i, Logic::Resource))
}
fn linking(input: &str) -> IResult<&str, Logic> {
    tag_no_case("linking")(input).map(|(i, _)| (i, Logic::Linking))
}
