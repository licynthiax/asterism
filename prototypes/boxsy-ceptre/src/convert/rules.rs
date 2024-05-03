use crate::convert::*;
#[derive(Debug)]
pub enum CEvent {
    ChangeResource {
        giver: GameType,
        receiver: GameType,
        amount: i16,
    },
    MoveRoom,
}

pub fn process_rule<'a>(
    r: &'a Rule,
    builtins: &'a Builtins,
    types: &'a CeptreTypes,
    queries: &'a BTreeMap<CData, Predicate>,
) -> Result<CEvent, crate::Error<'a>> {
    use std::collections::BTreeSet;
    if let Some(a) = &r.annote {
        let logics = a.get_logics();
        if logics == &Event::ChangeResource.associated_logics() {
            return Ok(CEvent::ChangeResource {
                giver: GameType::Character,
                receiver: GameType::Player,
                amount: 1,
            });
        } else if logics == &Event::MoveRoom.associated_logics() {
            return Ok(CEvent::MoveRoom);
        }
    }
    Err(crate::Error::RuleNotFound(r.name.as_str()))
}
