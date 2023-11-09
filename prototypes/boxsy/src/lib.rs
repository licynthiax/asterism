//! # What a Bitsy is, to me
//!
//! - characters moving around a tilemap
//! - tiles
//! - sprites that you interact with (dialogue?)
//! - items/inventory??
//! - multiple rooms, moving from one to another
//!
//! drawing: I'm not doing the pixel art thing. The player, interactable characters, and tiles all have different colors
//!
//! TODO:
//! - [x] Write tilemap collision
//! - ~~Write syntheses functions~~
//! - [x] Write test game
//! - [x] See what errors still persist from there
//! - [x] Add resource logics/inventory (I think????)
//! - [x] Adding/removing entities
//! - [x] Add linking logics
//!     - [x] graph/state machine struct
//! - [x] composing multiple queries

#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::new_without_default)]

use std::collections::BTreeMap;

use asterism::{
    control::{KeyboardControl, MacroquadInputWrapper},
    linking::GraphedLinking,
    resources::QueuedResources,
};
use macroquad::prelude::*;

// reexports
pub use asterism::control::{Action, ControlEventType, ControlReaction, Values};
pub use asterism::linking::{LinkingEvent, LinkingEventType, LinkingReaction};
pub use asterism::resources::{ResourceEventType, ResourceReaction, Transaction};
pub use asterism::Logic;
pub use collision::*;
pub use events::EngineAction;
pub use types::*;

const TILE_SIZE: usize = 32;
pub const WORLD_SIZE: usize = 8;
pub const GAME_SIZE: usize = TILE_SIZE * WORLD_SIZE;

mod collision;
mod entities;
mod events;
mod types;
use events::*;

pub fn window_conf() -> Conf {
    Conf {
        window_title: "extreme dungeon crawler".to_owned(),
        window_width: GAME_SIZE as i32,
        window_height: GAME_SIZE as i32,
        fullscreen: false,
        ..Default::default()
    }
}

pub struct Game {
    pub state: State,
    pub logics: Logics,
    events: Events,
    pub colors: Colors,
}

impl Game {
    pub fn new() -> Self {
        Self {
            state: State::new(),
            logics: Logics::new(),
            events: Events::new(),
            colors: Colors {
                background_color: DARKBLUE,
                colors: BTreeMap::new(),
            },
        }
    }

    pub fn get_current_room(&self) -> usize {
        let node = self.logics.linking.graphs[0].get_current_node();
        self.state.links.get(&node).unwrap().0
    }
}

pub struct Colors {
    pub background_color: Color,
    pub colors: BTreeMap<EntID, Color>,
}

#[derive(Default)]
pub struct Room {
    pub chars: Vec<(CharacterID, IVec2)>,
    pub map: [[Option<TileID>; WORLD_SIZE]; WORLD_SIZE],
}

pub struct State {
    pub rooms: Vec<Room>,
    pub player: bool,
    pub resources: Vec<RsrcID>,
    rsrc_id_max: usize,
    char_id_max: usize,
    pub links: BTreeMap<LinkID, (usize, IVec2)>,
    link_id_max: usize,
    tile_type_count: usize,
    add_queue: Vec<Ent>,
    remove_queue: Vec<EntID>,
}

impl State {
    fn new() -> Self {
        Self {
            rooms: Vec::new(),
            player: false,
            char_id_max: 0,
            resources: Vec::new(),
            rsrc_id_max: 0,
            links: BTreeMap::new(),
            link_id_max: 0,
            tile_type_count: 0,
            add_queue: Vec::new(),
            remove_queue: Vec::new(),
        }
    }

    pub fn get_col_idx(&self, i: usize, ent: CollisionEnt) -> usize {
        match ent {
            CollisionEnt::Player => 0,
            CollisionEnt::Character => i + 1,
        }
    }

    pub fn queue_remove(&mut self, ent: EntID) {
        self.remove_queue.push(ent);
    }
    pub fn queue_add(&mut self, ent: Ent) {
        self.add_queue.push(ent);
    }
}

pub struct Logics {
    pub control: KeyboardControl<ActionID, MacroquadInputWrapper>,
    pub collision: TileMapCollision<TileID, CollisionEnt>,
    pub resources: QueuedResources<RsrcID, u16>,
    pub linking: GraphedLinking<LinkID>,
}

impl Logics {
    fn new() -> Self {
        Self {
            control: KeyboardControl::new(),
            collision: TileMapCollision::new(WORLD_SIZE, WORLD_SIZE),
            resources: QueuedResources::new(),
            linking: {
                let mut linking = GraphedLinking::new();
                linking.add_graph(0, []);
                linking
            },
        }
    }
}

pub async fn run(mut game: Game) {
    setup(&mut game);

    loop {
        draw(&mut game);

        let add_queue = std::mem::take(&mut game.state.add_queue);
        for ent in add_queue {
            match ent {
                Ent::TileID(tile, pos, room) => {
                    game.add_tile_at_pos(tile, room, pos);
                }
                Ent::Character(character, room) => {
                    game.add_character(character, room);
                }
            }
        }

        control(&mut game);
        collision(&mut game);
        resources(&mut game);
        linking(&mut game);

        let remove_queue = std::mem::take(&mut game.state.remove_queue);
        for ent in remove_queue {
            match ent {
                EntID::Player => {
                    game.remove_player();
                }
                EntID::Tile(id) => {
                    let mut remove = Vec::new();
                    for (room_idx, room) in game.state.rooms.iter().enumerate() {
                        for (y, row) in room.map.iter().enumerate() {
                            for (x, tile) in row.iter().enumerate() {
                                if let Some(tile) = tile {
                                    if *tile == id {
                                        remove.push((room_idx, IVec2::new(x as i32, y as i32)));
                                    }
                                }
                            }
                        }
                    }
                    for (i, pos) in remove {
                        game.remove_tile_at_pos(i, pos);
                    }
                }
                EntID::Character(id) => {
                    game.remove_character(id);
                }
            }
        }

        if is_key_down(KeyCode::Escape) {
            return;
        }
        next_frame().await;
    }
}

fn control(game: &mut Game) {
    game.logics.control.update(&());

    for (ctrl_event, reaction) in game.events.control.iter() {
        if game
            .logics
            .control
            .events()
            .iter()
            .any(|event| ctrl_event == event)
        {
            reaction.perform_action(&mut game.state, &mut game.logics);
        }
    }

    let ans = game
        .logics
        .control
        .events()
        .iter()
        .filter(|event| event.event_type == ControlEventType::KeyPressed);

    // if all four direction keys are not being pressed, set vel = 0
    if ans.count() == 0 {
        game.logics
            .collision
            .handle_predicate(&CollisionReaction::SetEntVel(0, IVec2::ZERO));
    }
}

fn collision(game: &mut Game) {
    game.logics.collision.update();

    for ((room, col_event), reaction) in game.events.collision.iter() {
        if game
            .logics
            .collision
            .events()
            .iter()
            .any(|event| col_event == event)
        {
            reaction.perform_action(&mut game.state, &mut game.logics);
        }
    }
}

fn resources(game: &mut Game) {
    game.logics.resources.update();

    for (rsrc_event, reaction) in game.events.resource_event.iter() {
        if game
            .logics
            .resources
            .events()
            .iter()
            .any(|event| rsrc_event == event)
        {
            reaction.perform_action(&mut game.state, &mut game.logics);
        }
    }
}

fn linking(game: &mut Game) {
    game.logics.linking.update();

    // only linking events
    for (lnk_event, reaction) in game.events.linking.iter() {
        if game
            .logics
            .linking
            .events()
            .iter()
            .any(|event| lnk_event == event)
        {
            reaction.perform_action(&mut game.state, &mut game.logics);
        }
    }
}

fn draw(game: &mut Game) {
    clear_background(game.colors.background_color);
    for (y, row) in game.logics.collision.map.iter().enumerate() {
        for (x, tile) in row.iter().enumerate() {
            if let Some(tile) = tile {
                let color = game
                    .colors
                    .colors
                    .get(&EntID::Tile(*tile))
                    .unwrap_or_else(|| panic!("tile {} color undefined", tile.idx()));
                draw_rectangle(
                    x as f32 * TILE_SIZE as f32,
                    y as f32 * TILE_SIZE as f32,
                    TILE_SIZE as f32,
                    TILE_SIZE as f32,
                    *color,
                );
            }
        }
    }

    if game.state.player {
        let color = game
            .colors
            .colors
            .get(&EntID::Player)
            .expect("player color not set");
        let pos = game.logics.collision.get_ident_data(ColIdent::EntIdx(
            game.state.get_col_idx(0, CollisionEnt::Player),
        ));
        if let TileMapColData::Ent { pos, .. } = pos {
            draw_rectangle(
                pos.x as f32 * TILE_SIZE as f32,
                pos.y as f32 * TILE_SIZE as f32,
                TILE_SIZE as f32,
                TILE_SIZE as f32,
                *color,
            );
        }
    }

    let current_room = game.get_current_room();

    // skips the first element in the collision entity list (true casts to 1, false casts to 0) if a player is set
    for (i, pos) in game
        .logics
        .collision
        .positions
        .iter()
        .skip(game.state.player as usize)
        .enumerate()
    {
        let character = game.state.rooms[current_room].chars[i].0;
        let color = game
            .colors
            .colors
            .get(&EntID::Character(character))
            .unwrap_or_else(|| panic!("character {} color defined", character.idx()));
        draw_rectangle(
            pos.x as f32 * TILE_SIZE as f32,
            pos.y as f32 * TILE_SIZE as f32,
            TILE_SIZE as f32,
            TILE_SIZE as f32,
            *color,
        );
    }
}

fn setup(game: &mut Game) {
    game.logics
        .collision
        .clear_and_resize_map(WORLD_SIZE, WORLD_SIZE);
    let current_room = game.get_current_room();

    entities::load_room(&mut game.state, &mut game.logics, current_room);

    let player = game.logics.collision.get_ident_data(ColIdent::EntIdx(0));
    if let TileMapColData::Ent {
        pos: &mut pos,
        amt_moved,
        ..
    } = player
    {
        game.logics.collision.positions.insert(0, pos);
        game.logics.collision.amt_moved.insert(0, amt_moved);
        game.logics
            .collision
            .metadata
            .insert(0, CollisionData::new(true, false, CollisionEnt::Player));
    } else {
        unreachable!();
    }

    // control events default
    if game.events.control.is_empty() {
        game.add_ctrl_predicate(
            ActionID::Up,
            ControlEventType::KeyPressed,
            EngineAction::MovePlayerBy(IVec2::new(0, -1)),
        );

        game.add_ctrl_predicate(
            ActionID::Down,
            ControlEventType::KeyPressed,
            EngineAction::MovePlayerBy(IVec2::new(0, 1)),
        );

        game.add_ctrl_predicate(
            ActionID::Left,
            ControlEventType::KeyPressed,
            EngineAction::MovePlayerBy(IVec2::new(-1, 0)),
        );

        game.add_ctrl_predicate(
            ActionID::Right,
            ControlEventType::KeyPressed,
            EngineAction::MovePlayerBy(IVec2::new(1, 0)),
        );
    }
}
