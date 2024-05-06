// what kinds of ceptre games work for this
#![allow(unused)]
use crate::boxsy_info::*;
pub(crate) use crate::convert::{builtins::*, ctypes::*, pred::*, rules::*, state::*};
use crate::json::*;

use boxsy::{Logic as AsterismLogic, WORLD_SIZE};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) mod builtins;
pub(crate) mod ctypes;
pub(crate) mod pred;
pub(crate) mod rules;
pub(crate) mod state;

pub struct Ceptre {
    // i don't actually know if i need these first three fields bc they're just useful for
    // matching with the state
    types: CeptreTypes,
    queries: BTreeMap<CData, Predicate>,
    rules: Vec<CEvent>,
    pub init_state: State,
}

impl Ceptre {
    pub fn from_program<'a>(program: Program) -> Result<Self, crate::Error<'a>> {
        let builtins = Builtins::new(&program)?;
        let types = CeptreTypes::get_types(&program)?;

        let mut preds = Vec::new();
        for pred in program.header.preds.iter() {
            if let Some(Annote::Query(_)) = pred.annote {
                preds.push(pred.clone());
            }
        }

        let queries: BTreeMap<CData, Predicate> = preds
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
            .filter_map(|rule| process_rule(rule, &builtins, &types, &queries).ok())
            .collect();

        let init_state = process_state(program.init_state, &types, &queries, &rules)?;

        // do something with initial stage/state
        Ok(Ceptre {
            types,
            queries,
            rules,
            init_state,
        })
    }
}

impl From<Ceptre> for boxsy::Game {
    #[allow(unused)]
    fn from(c: Ceptre) -> Self {
        use boxsy::*;
        use macroquad::prelude::*;

        let state = c.init_state;

        macroquad::rand::srand(get_time().to_bits());
        let mut game = Game::new();

        // map
        let (mut map, tiles) = state.generate_map();
        for m in map.iter() {
            game.add_room_from_str(m).unwrap();
        }

        // log items
        let items: Vec<RsrcID> = state
            .items
            .iter()
            .map(|i| game.log_rsrc(i.id.clone()))
            .collect();

        // generate player
        let player = {
            let mut p = Player::new();
            p.color = PURPLE;
            p.pos = state.generate_pos(&map[0]);
            add_char(&mut map[0], p.pos, 'p');
            for item in state.player.items.iter() {
                if let Some(i) = items.iter().find(|i| i.name() == item.id) {
                    p.add_inventory_item(i.clone(), item.count);
                }
            }
            p
        };
        game.set_player(player);

        // tiles
        for i in 0..tiles.len() {
            let mut tile = Tile::new();
            game.log_tile_info(tile);
        }

        // chars
        for c in state.characters.iter() {
            let mut item_ids: Vec<RsrcID> = Vec::new();
            let mut ch = Character::new();

            ch.color = BROWN;
            ch.pos = state.generate_pos(&map[c.in_room]);
            add_char(&mut map[c.in_room], ch.pos, 'c');

            for item in c.items.iter() {
                if let Some(i) = items.iter().find(|i| i.name() == item.id) {
                    ch.add_inventory_item(i.clone(), item.count);
                    item_ids.push(i.clone());
                }
            }

            let char_id = game.add_character(ch.clone(), c.in_room);

            for item in item_ids.iter() {
                game.add_collision_predicate(
                    (
                        c.in_room,
                        CollisionEnt::Player,
                        CollisionEnt::Character(char_id),
                    ),
                    EngineAction::ChangeResource(
                        PoolID::new(EntID::Character(char_id), item.clone()),
                        Transaction::Trade(1, PoolID::new(EntID::Player, item.clone())),
                    ),
                );
            }

            if let Some(to_room) = c.link_to {
                let ending = state.generate_pos(&map[to_room]);
                game.add_link(
                    (c.in_room, CollisionEnt::Character(char_id)),
                    (to_room, ending),
                );
            }
        }

        // tiles
        for (start_pos, link) in tiles.iter().flatten().zip(state.tiles.iter()) {
            let ending = state.generate_pos(&map[link.link_to]);
            game.add_link(
                (link.in_room, CollisionEnt::Tile(*start_pos)),
                (link.link_to, ending),
            );
        }

        game
    }
}

use macroquad::math::IVec2;
impl State {
    fn shift_room(&self, link_to: usize) -> usize {
        use std::cmp::Ordering;
        match link_to.cmp(&self.player.in_room) {
            Ordering::Equal => 0,
            Ordering::Greater => link_to,
            Ordering::Less => link_to + 1,
        }
    }

    /// a vec of strings of world maps + vec of vecs of positions for each tile type
    fn generate_map(&self) -> (Vec<String>, Vec<Vec<IVec2>>) {
        // generate map:
        // - how many rooms?
        // - where's the player?
        // - how many tiles in which rooms?
        // - misc other tiles (decorative) (optional)

        let mut map = {
            let mut map = Vec::new();
            map.resize_with(self.rooms, || {
                (" ".repeat(WORLD_SIZE) + "\n").repeat(WORLD_SIZE - 1) + &" ".repeat(WORLD_SIZE)
            });
            map
        };
        let num_rooms = self.rooms;
        let starting_room = self.player.in_room;

        let tiles = self
            .tiles
            .iter()
            .map(|t| (&t.id, self.shift_room(t.in_room)));

        let mut ids: Vec<&String> = Vec::new();
        let mut positions: Vec<Vec<IVec2>> = Vec::new();

        for (id, room) in tiles {
            let tile_idx =
                if let Some((i, _)) = ids.iter().enumerate().find(|(_, tile_id)| **tile_id == id) {
                    i
                } else {
                    if ids.len() >= 10 {
                        continue;
                    }
                    ids.push(id);
                    positions.push(Vec::new());
                    ids.len() - 1
                };

            let pos @ IVec2 { x, y } = self.generate_pos(&map[room]);
            positions[tile_idx].push(pos);

            let mut map_idx = y as usize * (WORLD_SIZE + 1) + x as usize;
            map[room].replace_range(map_idx..map_idx + 1, &tile_idx.to_string());
        }
        if starting_room != 0 {
            let starting_map = map.remove(starting_room);
            map.insert(0, starting_map);
        }

        (map, positions)
    }

    fn generate_pos(&self, room: &str) -> macroquad::math::IVec2 {
        use macroquad::{math::IVec2, rand::RandomRange};

        let mut x = RandomRange::gen_range(0, WORLD_SIZE);
        let mut y = RandomRange::gen_range(0, WORLD_SIZE);
        let mut map_idx = y * (WORLD_SIZE + 1) + x;

        // checks if chosen spot is taken or not
        while room.chars().nth(map_idx).is_some_and(|c| c != ' ') {
            x = RandomRange::gen_range(0, WORLD_SIZE);
            y = RandomRange::gen_range(0, WORLD_SIZE);
            map_idx = y * (WORLD_SIZE + 1) + x;
        }

        IVec2::new(x as i32, y as i32)
    }
}
fn add_char(room: &mut String, IVec2 { x, y }: IVec2, ch: char) {
    let mut map_idx = y as usize * (WORLD_SIZE + 1) + x as usize;
    room.replace_range(map_idx..map_idx + 1, &ch.to_string());
}
