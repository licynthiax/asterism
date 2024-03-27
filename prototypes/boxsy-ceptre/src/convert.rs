// what kinds of ceptre games work for this
#![allow(unused)]
use crate::boxsy_info::*;
use crate::json::*;
use boxsy::Logic as AsterismLogic;
use std::collections::{BTreeMap, BTreeSet};

pub struct Ceptre {
    builtins: Vec<Builtin>,
    types: CeptreTypes,
    queries: BTreeMap<CData, String>,
    rules: Vec<CEvent>, // ????
    init_state: State,
}

pub struct CeptreTypes {
    player: CType,
    tile: CType,
    character: CType,
    room: CType,
    rsrc_id: CType,
}

impl CeptreTypes {
    fn find_type(&self, s: &str) -> Option<&CType> {
        if self.player.name == s {
            Some(&self.player)
        } else if self.tile.name == s {
            Some(&self.tile)
        } else if self.character.name == s {
            Some(&self.character)
        } else if self.room.name == s {
            Some(&self.room)
        } else if self.rsrc_id.name == s {
            Some(&self.rsrc_id)
        } else {
            None
        }
    }
}

#[derive(Clone)]
struct CType {
    name: String,
    tp: Vec<Tp>,
    annote: Annote,
}

impl TryFrom<Type> for CType {
    type Error = crate::Error<'static>;
    fn try_from(t: Type) -> Result<Self, Self::Error> {
        Ok(Self {
            name: t.name,
            tp: t.tp,
            annote: t.annote.ok_or(crate::Error::Custom("no annote"))?,
        })
    }
}

struct Pred {
    name: String,
    types: Vec<CType>,
    annote: Annote,
}

impl Pred {
    fn from_predicate(p: Predicate, types: &CeptreTypes) -> Result<Self, crate::Error<'static>> {
        Ok(Self {
            name: p.name,
            types: p
                .terms
                .iter()
                .filter_map(|t| types.find_type(t.str()))
                .cloned()
                .collect(),
            annote: p.annote.ok_or(crate::Error::Custom("no annote"))?,
        })
    }
}

impl Ceptre {
    pub fn from_program<'a>(program: Program) -> Result<Self, crate::Error<'a>> {
        let builtins = get_builtins(&program)?;
        let types = CeptreTypes::get_types(&program)?;

        // some kind of processing has to be done here
        let mut queries = Vec::new();
        for pred in program.header.preds.iter() {
            if let Some(Annote::Query(_)) = pred.annote {
                queries.push(pred.clone());
            }
        }
        let queries: BTreeMap<CData, String> = queries
            .iter()
            .filter_map(|p| process_pred(p, &types).ok())
            .collect();

        let mut rules = Vec::new();
        for stage in program.stages.iter() {
            for rule in stage.body.iter() {
                if let Some(Annote::Integration(_)) = rule.annote {
                    rules.push(rule);
                }
            }
        }

        let rules: Vec<CEvent> = rules
            .iter()
            .filter_map(|rule| process_rule(rule, builtins.as_slice(), &types, &queries).ok())
            .collect();

        let init_state = process_state(program.init_state, &builtins, &types, &queries, &rules);
        let init_state = match init_state {
            Ok(s) => s,
            Err(e) => return Err(e),
        };

        // do something with initial stage/state
        Ok(Ceptre {
            types,
            builtins,
            queries,
            rules,
            init_state,
        })
    }
}

impl From<Ceptre> for boxsy::Game {
    #[allow(unused)]
    fn from(c: Ceptre) -> Self {
        todo!()
    }
}

/// identify builtin types
fn get_builtins(program: &Program) -> Result<Vec<Builtin>, crate::Error<'static>> {
    let nat = program
        .builtins
        .iter()
        .find(|b| b.builtin == BuiltinTypes::NAT);
    let s = program
        .builtins
        .iter()
        .find(|b| b.builtin == BuiltinTypes::NAT_SUCC);
    let z = program
        .builtins
        .iter()
        .find(|b| b.builtin == BuiltinTypes::NAT_ZERO);

    // we want all three or nothin
    let nat = nat.ok_or(crate::Error::BuiltinNotFound(BuiltinTypes::NAT))?;
    let s = s.ok_or(crate::Error::BuiltinNotFound(BuiltinTypes::NAT_SUCC))?;
    let z = z.ok_or(crate::Error::BuiltinNotFound(BuiltinTypes::NAT_ZERO))?;

    Ok(vec![nat.clone(), s.clone(), z.clone()])
}

impl CeptreTypes {
    /// match asterism types with the ceptre ones
    fn get_types(program: &Program) -> Result<CeptreTypes, crate::Error<'static>> {
        // grab annotes on types. legally speaking data and syntheses could be one thing here but
        // i like maintaining the difference because rsrcs aren't like coherent things distinct
        // from the things theyre attached to (?) at least in boxsy
        let mut syntheses = Vec::new();
        let mut data = Vec::new();
        for ty in program.header.types.iter() {
            match ty.annote {
                Some(Annote::Synthesis(_)) => syntheses.push(ty),
                Some(Annote::Data(_)) => data.push(ty),
                _ => {}
            }
        }

        let mut player = None;
        let mut character = None;
        let mut tile = None;
        let mut room = None;
        for s in syntheses {
            if let Some(a) = s.annote.as_ref() {
                if a.get_logics() == &GameType::Player.associated_logics() {
                    player = Some(s.clone().try_into()?);
                }
                if a.get_logics() == &GameType::Character.associated_logics() {
                    character = Some(s.clone().try_into()?);
                }
                if a.get_logics() == &GameType::Tile.associated_logics() {
                    tile = Some(s.clone().try_into()?);
                }
                if a.get_logics() == &GameType::Room.associated_logics() {
                    room = Some(s.clone().try_into()?);
                }
            }
        }

        let mut rsrc_id = None;
        for d in data {
            if let Some(a) = d.annote.as_ref() {
                if a.get_logics() == &GameType::Rsrc.associated_logics() {
                    rsrc_id = Some(d.clone().try_into()?);
                }
            }
        }

        let player = player.ok_or(crate::Error::TypeNotFound(GameType::Player))?;
        let character = character.ok_or(crate::Error::TypeNotFound(GameType::Character))?;
        let tile = tile.ok_or(crate::Error::TypeNotFound(GameType::Tile))?;
        let room = room.ok_or(crate::Error::TypeNotFound(GameType::Room))?;
        let rsrc_id = rsrc_id.ok_or(crate::Error::TypeNotFound(GameType::Rsrc))?;

        Ok(CeptreTypes {
            player,
            tile,
            character,
            room,
            rsrc_id,
        })
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
enum CData {
    PlayerInRoom,
    CharInRoom,
    TileInRoom,
    CharHasItem,
    PlayerHasItem,
}

impl CData {
    fn associated_types(&self, types: &CeptreTypes) -> BTreeSet<String> {
        match self {
            Self::PlayerInRoom => {
                BTreeSet::from([types.player.name.clone(), types.room.name.clone()])
            }
            Self::CharInRoom => {
                BTreeSet::from([types.character.name.clone(), types.room.name.clone()])
            }
            Self::TileInRoom => BTreeSet::from([types.tile.name.clone(), types.room.name.clone()]),
            Self::CharHasItem => {
                BTreeSet::from([types.character.name.clone(), types.rsrc_id.name.clone()])
            }
            Self::PlayerHasItem => {
                BTreeSet::from([types.player.name.clone(), types.rsrc_id.name.clone()])
            }
        }
    }
}

fn process_pred<'a>(
    p: &'a Predicate,
    types: &'a CeptreTypes,
) -> Result<(CData, String), crate::Error<'a>> {
    let involved: BTreeSet<String> = p
        .terms
        .iter()
        .filter_map(|term| types.find_type(term.str()))
        .map(|term| term.name.clone())
        .collect();
    let ty = if involved == CData::PlayerInRoom.associated_types(types) {
        CData::PlayerInRoom
    } else if involved == CData::PlayerHasItem.associated_types(types) {
        CData::PlayerHasItem
    } else if involved == CData::CharInRoom.associated_types(types) {
        CData::CharInRoom
    } else if involved == CData::CharHasItem.associated_types(types) {
        CData::CharHasItem
    } else if involved == CData::TileInRoom.associated_types(types) {
        CData::TileInRoom
    } else {
        return Err(crate::Error::PredNotFound(p.name.as_str()));
    };
    Ok((ty, p.name.clone()))
}

#[derive(Debug)]
enum CEvent {
    ChangeResource {
        giver: GameType,
        receiver: GameType,
        amount: u16,
    },
    MoveRoom,
}

fn process_rule<'a>(
    r: &'a Rule,
    builtins: &'a [Builtin],
    types: &'a CeptreTypes,
    queries: &'a BTreeMap<CData, String>,
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
        // get the atoms in the statement
        // get the terms in the atoms
        // intersection of the types in annote with each
    }
    Err(crate::Error::RuleNotFound(r.name.as_str()))
}

pub struct State {
    player: Player,
    characters: Vec<Character>,
    rooms: usize, // a count of how many rooms there are
    tile: Vec<Tile>,
}

struct Player {
    in_room: usize,
    items: Vec<Rsrc>,
}
struct Character {
    id: String,
    item: Rsrc,
    link_to: usize,
}
struct Tile {
    id: String,
    link_to: usize,
}
struct Rsrc {
    id: String,
    count: u16,
}

fn process_state<'a, 'b>(
    init_state: Vec<Atom>,
    builtins: &'a [Builtin],
    types: &'a CeptreTypes,
    queries: &'a BTreeMap<CData, String>,
    rules: &'a [CEvent],
) -> Result<State, crate::Error<'b>>
where
    'b: 'a,
{
    // find these predicates in the state
    let player_in_room = init_state
        .iter()
        .find(|a| Some(&a.name) == queries.get(&CData::PlayerInRoom));
    let player_has_item = init_state
        .iter()
        .find(|a| Some(&a.name) == queries.get(&CData::PlayerHasItem));
    let char_in_room = init_state
        .iter()
        .find(|a| Some(&a.name) == queries.get(&CData::CharInRoom));
    let char_has_item = init_state
        .iter()
        .find(|a| Some(&a.name) == queries.get(&CData::CharHasItem));
    let tile_in_room = init_state
        .iter()
        .find(|a| Some(&a.name) == queries.get(&CData::TileInRoom));
    dbg!(player_in_room.unwrap());

    Err(crate::Error::Custom("h"))
}

/* this is reasoning for a more complicated version of this
let lhs_terms: BTreeSet<Predicate> = r
    .lhs
    .into_iter()
    .filter_map(|a| queries.iter().find(|q| q.name == a.name).cloned())
    .collect();
let rhs_terms: BTreeSet<Predicate> = r
    .rhs
    .into_iter()
    .filter_map(|a| queries.iter().find(|q| q.name == a.name).cloned())
    .collect();

let mut preds: Vec<Pred> = lhs_terms
    .intersection(&rhs_terms)
    .filter_map(|p| Pred::from_predicate((*p).clone(), types).ok())
    .collect();
let mut pred_logics = preds.iter_mut().fold(BTreeSet::new(), |mut set, p| {
    set.append(p.annote.get_logics_mut());
    set
});
let logics_changed = pred_logics.intersection(logics);
for l in logics_changed {
    dbg!(l.to_string());
} */

impl CType {
    fn find_tp(&self, name: String) -> Option<(usize, &Tp)> {
        self.tp.iter().enumerate().find(|(_, tp)| tp.name == name)
    }
}
