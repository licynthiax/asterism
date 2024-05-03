use crate::convert::*;
use crate::parse::ceptre::*;

use std::collections::BTreeSet;

#[derive(Debug)]
pub struct State {
    pub player: Player,
    pub characters: Vec<Character>,
    pub rooms: usize, // a count of how many rooms there are
    pub tiles: Vec<Tile>,
    pub items: BTreeSet<Rsrc>,
}

pub fn process_state<'a, 'b>(
    init_state: Vec<Atom>,
    types: &'a CeptreTypes,
    queries: &'a BTreeMap<CData, Predicate>,
    rules: &'a [CEvent],
) -> Result<State, crate::Error<'b>>
where
    'b: 'a,
{
    // find these predicates in the state
    let filter = |a: &Atom, q: &CData| {
        if let Some(p) = queries.get(q) {
            p.name == a.name
        } else {
            false
        }
    };
    let collect = |init_state: &[Atom], q: &CData| -> Vec<Atom> {
        init_state
            .iter()
            .filter_map(|a| if filter(a, q) { Some(a.clone()) } else { None })
            .collect()
    };

    let player_in_room = init_state.iter().find(|a| filter(a, &CData::PlayerInRoom));
    let player_has_item = collect(&init_state, &CData::PlayerHasItem);
    let char_in_room = collect(&init_state, &CData::CharInRoom);
    let char_link_to = collect(&init_state, &CData::CharToRoom);
    let char_has_item = collect(&init_state, &CData::CharHasItem);
    let tile_in_room = collect(&init_state, &CData::TileInRoom);
    let tile_link_to = collect(&init_state, &CData::TileToRoom);

    // number of rooms is the number of room ids
    let rooms = types.room.tp.len();
    let player = build_player(player_in_room, player_has_item, types)?;
    let characters = build_chars(char_in_room, char_has_item, char_link_to, types)?;
    let tiles = build_tiles(tile_in_room, tile_link_to, types)?;

    let mut items: BTreeSet<Rsrc> = BTreeSet::from_iter(player.items.clone());
    for ch in characters.iter() {
        items.append(&mut BTreeSet::from_iter(ch.items.clone()));
    }

    Ok(State {
        player,
        characters,
        rooms,
        tiles,
        items,
    })
}

#[derive(Debug)]
pub struct Player {
    pub in_room: usize,
    pub items: Vec<Rsrc>,
}
#[derive(Debug)]
pub struct Character {
    pub id: String,
    pub items: Vec<Rsrc>,
    pub in_room: usize,
    pub link_to: Option<usize>,
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tile {
    pub id: String,
    pub in_room: usize,
    pub link_to: usize,
}
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Rsrc {
    pub id: String,
    pub count: i16,
}

// okay this type sucks but hear me out
pub fn build_item<'a, 'b: 'a>(
    atom: &'a Atom,
    types: &CeptreTypes,
) -> Option<Result<Rsrc, crate::Error<'b>>> {
    let mut id: Option<Tp> = None;
    let mut count: Option<i16> = None;
    for t in atom.terms.iter() {
        if let Ok((_, at)) = parse(t.str(), &types.rsrc) {
            for (tp_name, val) in at.tp.args.iter().zip(at.vals.iter()) {
                if let Some((_, rsrc_id)) = types.rsrc.find_tp(val) {
                    id = Some(rsrc_id.clone());
                }
                if *tp_name == types.builtins.nat {
                    let c = val
                        .as_str()
                        .parse()
                        .map_err(|_| crate::Error::State("unable to parse number"));
                    match c {
                        Ok(c) => count = Some(c),
                        Err(e) => return Some(Err(e)),
                    }
                }
            }
        }
    }
    if let (Some(id), Some(count)) = (id, count) {
        return Some(Ok(Rsrc {
            id: id.name.clone(),
            count,
        }));
    }
    None
}

pub fn build_player<'a, 'b>(
    player_in_room: Option<&Atom>,
    player_has_item: Vec<Atom>,
    types: &'a CeptreTypes,
) -> Result<Player, crate::Error<'b>>
where
    'b: 'a,
{
    let player_in_room =
        player_in_room.ok_or(crate::Error::State("player initial position not found"))?;

    let player = player_in_room
        .terms
        .iter()
        .find(|t| types.player.find_tp(t.str()).is_some())
        .ok_or(crate::Error::State(
            "player tp used could not be found in type definition",
        ))?;

    let in_room = {
        let mut room = None;
        for term in player_in_room.terms.iter() {
            if let Ok((term, _)) = parse(term.str(), &types.room) {
                if let Some((r, _)) = types.room.find_tp(term) {
                    room = Some(r);
                    break;
                }
            }
        }
        room.ok_or(crate::Error::State("player initial position not found"))
    }?;

    let mut items = Vec::new();
    for atom in player_has_item {
        if let Some(item) = build_item(&atom, types) {
            let item = item?;
            items.push(item);
        }
    }

    Ok(Player { in_room, items })
}

#[derive(Clone, Debug)]
pub struct AtomTp<'tp> {
    pub tp: &'tp Tp,
    pub vals: Vec<String>,
}

pub fn build_chars<'a, 'b>(
    char_in_room: Vec<Atom>,
    char_has_item: Vec<Atom>,
    char_link_to: Vec<Atom>,
    types: &'a CeptreTypes,
) -> Result<Vec<Character>, crate::Error<'b>>
where
    'b: 'a,
{
    #[derive(Debug)]
    struct CharBuilder {
        in_room: Option<usize>,
        items: Option<Vec<Rsrc>>,
        link_to: Option<usize>,
    }
    impl CharBuilder {
        fn new() -> Self {
            Self {
                in_room: None,
                items: None,
                link_to: None,
            }
        }
        fn try_to_char(self, tp: &Tp) -> Option<Character> {
            Some(Character {
                id: tp.name.clone(),
                in_room: self.in_room?,
                items: self.items?,
                link_to: self.link_to,
            })
        }
    }

    let mut builders: BTreeMap<Tp, CharBuilder> = BTreeMap::new();

    for a in char_in_room {
        let mut room = None;
        let mut ch = None;
        for term in a.terms.iter() {
            if ch.is_none() {
                if let Ok((_, term)) = parse(term.str(), &types.character) {
                    if let Some((_, c)) = types.character.find_tp(&term.tp.name) {
                        ch = Some(c);
                        continue;
                    }
                }
            }
            if room.is_none() {
                if let Ok((_, term)) = parse(term.str(), &types.room) {
                    if let Some((r, _)) = types.room.find_tp(&term.tp.name) {
                        room = Some(r);
                    }
                }
            }
        }
        if let (Some(c), Some(r)) = (ch, room) {
            builders.insert(c.clone(), {
                let mut cb = CharBuilder::new();
                cb.in_room = room;
                cb
            });
        }
    }

    for a in char_has_item {
        let mut char = None;
        for term in a.terms.iter() {
            if char.is_none() {
                if let Ok((_, term)) = parse(term.str(), &types.character) {
                    if let Some((_, c)) = types.character.find_tp(&term.tp.name) {
                        char = Some(c);
                        continue;
                    }
                }
            }
        }

        let mut items = Vec::new();
        if let Some(item) = build_item(&a, types) {
            let item = item?;
            items.push(item);
        }

        if let Some(c) = char {
            if let Some(builder) = builders.get_mut(c) {
                builder.items = Some(items);
            }
        }
    }

    for a in char_link_to {
        let mut link = None;
        let mut char = None;
        for term in a.terms.iter() {
            if char.is_none() {
                if let Ok((_, term)) = parse(term.str(), &types.character) {
                    if let Some((_, c)) = types.character.find_tp(&term.tp.name) {
                        char = Some(c);
                        continue;
                    }
                }
            }
            if link.is_none() {
                if let Ok((_, term)) = parse(term.str(), &types.link) {
                    for (val, arg) in term.vals.iter().zip(term.tp.args.iter()) {
                        if *arg == types.room.name {
                            let (l, _) = types.room.find_tp(val).unwrap();
                            link = Some(l);
                        }
                    }
                }
            }
        }
        if let (Some(c), Some(l)) = (char, link) {
            if let Some(builder) = builders.get_mut(c) {
                builder.link_to = link;
            }
        }
    }

    let mut chars = Vec::new();
    for (tp, builder) in builders {
        if let Some(ch) = builder.try_to_char(&tp) {
            chars.push(ch);
        }
    }

    Ok(chars)
}

pub fn build_tiles<'a, 'b>(
    tile_in_room: Vec<Atom>,
    tile_link_to: Vec<Atom>,
    types: &'a CeptreTypes,
) -> Result<Vec<Tile>, crate::Error<'b>>
where
    'b: 'a,
{
    let mut tiles = Vec::new();

    // this finds links
    for a in tile_in_room.iter().chain(tile_link_to.iter()) {
        let mut room = None;
        let mut exit = None;
        for term in a.terms.iter() {
            if room.is_none() {
                if let Ok((_, term)) = parse(term.str(), &types.room) {
                    if let Some((r, _)) = types.room.find_tp(&term.tp.name) {
                        room = Some(r);
                        continue;
                    }
                }
            }
            if exit.is_none() {
                if let Ok((_, term)) = parse(term.str(), &types.link) {
                    for (val, arg) in term.vals.iter().zip(term.tp.args.iter()) {
                        if *arg == types.room.name {
                            let (l, _) = types.room.find_tp(val).unwrap();
                            exit = Some(l);
                        }
                    }
                }
            }
        }
        if let (Some(r), Some(x)) = (room, exit) {
            tiles.push(Tile {
                id: "1".to_string(),
                in_room: r,
                link_to: x,
            });
        }
    }

    Ok(tiles)
}
