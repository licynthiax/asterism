/// parsing the annotations for the types in the json files
use nom::{
    branch::*, bytes::complete::*, character::complete::*, combinator::*, multi::*, sequence::*,
};
use nom::{error::Error, Parser};

pub(crate) mod json {
    use nom::{IResult, Parser};
    use std::collections::BTreeSet;

    use super::*;
    use crate::boxsy_info::Logic;
    use crate::json::{Annote, AnnoteString};

    pub fn parse(s: &AnnoteString) -> IResult<&str, Annote> {
        let s = s.get_str();
        let (i, mut annote) = alt((synth_or_data, ws(integration), ws(query))).parse(s)?;
        let (i, list) = braces(list).parse(i)?;
        let l = annote.get_logics_mut();
        *l = list;
        Ok((i, annote))
    }

    fn synth_or_data(input: &str) -> IResult<&str, Annote> {
        let s = tag_no_case("synthesis");
        let d = tag_no_case("data");
        let res = alt((
            separated_pair(ws(&s), tag(","), ws(&d)),
            separated_pair(ws(&d), tag(","), ws(&s)),
        ))(input)
        .map(|(i, _)| (i, Annote::SynthData(BTreeSet::new())));
        res.or_else(|_| alt((ws(synthesis), ws(data)))(input))
    }

    fn synthesis(input: &str) -> IResult<&str, Annote> {
        tag_no_case("synthesis")(input).map(|(i, _)| (i, Annote::Synthesis(BTreeSet::new())))
    }
    fn data(input: &str) -> IResult<&str, Annote> {
        tag_no_case("data")(input).map(|(i, _)| (i, Annote::Data(BTreeSet::new())))
    }

    fn integration(input: &str) -> IResult<&str, Annote> {
        tag_no_case("integration")(input).map(|(i, _)| (i, Annote::Integration(BTreeSet::new())))
    }
    fn query(input: &str) -> IResult<&str, Annote> {
        tag_no_case("query")(input).map(|(i, _)| (i, Annote::Query(BTreeSet::new())))
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
}

/// for ceptre terms not parsed by ceptre (???)
pub(crate) mod ceptre {
    use super::*;
    use nom::{error::Error, IResult, Parser};

    use crate::convert::{ctypes::CType, AtomTp};

    pub(crate) fn parse<'a>(s: &'a str, ty: &'a CType) -> IResult<&'a str, AtomTp<'a>> {
        alt((parens(terms(ty)), standalone(ty))).parse(s)
    }

    fn terms<'a>(ty: &'a CType) -> impl FnMut(&'a str) -> IResult<&str, AtomTp<'_>> {
        |input: &'a str| {
            separated_list0(multispace1, is_not(" \t\r\n)"))
                .parse(input)
                .and_then(|(s, list)| {
                    let mut list: Vec<String> = list.iter().map(|s| s.to_string()).collect();
                    let name = list.remove(0);
                    let tp = ty
                        .tp
                        .iter()
                        .find(|t| t.name == name)
                        .ok_or(nom::Err::Error(Error::new(
                            input,
                            nom::error::ErrorKind::Alt,
                        )))?;
                    let at = AtomTp {
                        tp,
                        vals: list.iter().map(|s| s.to_string()).collect(),
                    };
                    Ok((s, at))
                })
        }
    }

    fn standalone<'a>(ty: &'a CType) -> impl FnMut(&'a str) -> IResult<&'a str, AtomTp<'a>> {
        |input: &'a str| -> IResult<&'a str, AtomTp<'a>> {
            let tp = ty
                .tp
                .iter()
                .find(|t| t.name == input)
                .ok_or(nom::Err::Error(Error::new(
                    input,
                    nom::error::ErrorKind::Alt,
                )))?;
            success::<&'a str, AtomTp<'a>, Error<&'a str>>(AtomTp {
                tp,
                vals: Vec::new(),
            })
            .parse(input)
        }
    }
}

// util
fn ws<'a, I, F>(inner: F) -> impl Parser<&'a str, I, Error<&'a str>>
where
    F: Parser<&'a str, I, Error<&'a str>>,
{
    delimited(multispace0, inner, multispace0)
}

fn parens<'a, I, F>(inner: F) -> impl Parser<&'a str, I, Error<&'a str>>
where
    F: Parser<&'a str, I, Error<&'a str>>,
{
    delimited(char('('), ws(inner), char(')'))
}
fn braces<'a, I, F>(inner: F) -> impl Parser<&'a str, I, Error<&'a str>>
where
    F: Parser<&'a str, I, Error<&'a str>>,
{
    delimited(char('{'), ws(inner), char('}'))
}
