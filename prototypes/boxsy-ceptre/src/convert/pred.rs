use crate::convert::*;

pub struct Pred {
    pub name: String,
    pub types: Vec<CType>,
    pub annote: Annote,
}

impl Pred {
    pub fn from_predicate(
        p: Predicate,
        types: &CeptreTypes,
    ) -> Result<Self, crate::Error<'static>> {
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

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum CData {
    PlayerInRoom,
    PlayerHasItem,
    CharInRoom,
    CharHasItem,
    CharToRoom,
    TileInRoom,
    TileToRoom,
}

impl CData {
    pub fn associated_types(&self, types: &CeptreTypes) -> BTreeSet<String> {
        match self {
            Self::PlayerInRoom => {
                BTreeSet::from([types.player.name.clone(), types.room.name.clone()])
            }
            Self::PlayerHasItem => {
                BTreeSet::from([types.player.name.clone(), types.rsrc.name.clone()])
            }
            Self::CharInRoom => {
                BTreeSet::from([types.character.name.clone(), types.room.name.clone()])
            }
            Self::CharHasItem => {
                BTreeSet::from([types.character.name.clone(), types.rsrc.name.clone()])
            }
            Self::CharToRoom => {
                BTreeSet::from([types.character.name.clone(), types.link.name.clone()])
            }
            Self::TileInRoom => BTreeSet::from([types.link.name.clone(), types.room.name.clone()]),
            Self::TileToRoom => BTreeSet::from([types.link.name.clone(), types.room.name.clone()]),
        }
    }
}

pub fn process_pred<'a>(
    p: &'a Predicate,
    types: &'a CeptreTypes,
) -> Result<(CData, Predicate), crate::Error<'a>> {
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
    } else if involved == CData::CharToRoom.associated_types(types) {
        CData::CharToRoom
    } else if involved == CData::TileInRoom.associated_types(types) {
        CData::TileInRoom
    } else if involved == CData::TileToRoom.associated_types(types) {
        CData::TileToRoom
    } else {
        return Err(crate::Error::PredNotFound(p.name.as_str()));
    };
    Ok((ty, p.clone()))
}
