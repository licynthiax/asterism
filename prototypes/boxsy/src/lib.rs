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
    lending_iterator::*,
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
    pub draw: Draw,
}

impl Game {
    pub fn new() -> Self {
        Self {
            state: State::new(),
            logics: Logics::new(),
            events: Events::new(),
            draw: Draw {
                draw_timer: Vec::new(),
                background_color: DARKBLUE,
                colors: BTreeMap::new(),
            },
        }
    }

    pub fn get_current_room(&self) -> usize {
        self.logics.linking.graphs[0].get_current_node()
    }
}

pub struct Draw {
    draw_timer: Vec<(Box<dyn Fn()>, usize)>,
    background_color: Color,
    colors: BTreeMap<EntID, Color>,
}

#[derive(Default)]
pub struct Room {
    pub chars: Vec<(CharacterID, IVec2)>,
    pub map: [[Option<TileID>; WORLD_SIZE]; WORLD_SIZE],
}

impl Room {
    pub(crate) fn find_char(&self, id: CharacterID) -> Option<(usize, IVec2)> {
        self.chars
            .iter()
            .enumerate()
            .find(|(_, (char_id, _))| *char_id == id)
            .map(|(i, (_, pos))| (i, *pos))
    }
}

pub struct State {
    pub rooms: Vec<Room>,
    pub player: bool,
    pub resources: Vec<RsrcID>,
    rsrc_id_max: usize,
    char_id_max: usize,
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
            tile_type_count: 0,
            add_queue: Vec::new(),
            remove_queue: Vec::new(),
        }
    }

    #[allow(unused)]
    /// looks for a character with the given id and spits out a tuple containing the room number,
    /// character's index, and position. greedy; characters can only be added once
    pub(crate) fn find_char(&self, id: CharacterID) -> Option<(usize, (usize, IVec2))> {
        for (i, room) in self.rooms.iter().enumerate() {
            let ch = room.find_char(id);
            if ch.is_some() {
                return ch.map(|info| (i, info));
            }
        }
        None
    }

    /// returns index and position of character
    pub(crate) fn find_char_in_room(&self, room: usize, id: CharacterID) -> Option<(usize, IVec2)> {
        self.rooms[room].find_char(id)
    }

    /// room number is needed if the entity is a character
    pub(crate) fn get_col_idx(&self, id: CharacterID, room: Option<usize>) -> Option<usize> {
        self.find_char_in_room(room.unwrap(), id)
            .map(|(i, _)| i + 1)
    }

    #[allow(unused)]
    pub(crate) fn queue_remove(&mut self, ent: EntID) {
        self.remove_queue.push(ent);
    }
    pub(crate) fn queue_add(&mut self, ent: Ent) {
        self.add_queue.push(ent);
    }
}

pub struct Logics {
    pub control: KeyboardControl<ActionID, MacroquadInputWrapper>,
    pub collision: TileMapCollision<TileID, ColEntType>,
    pub resources: QueuedResources<PoolID, i16>,
    // usize = room number
    pub linking: GraphedLinking<usize>,
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
    let current_room = game.get_current_room();
    game.logics.collision.update();

    for ((room, col_event), reaction) in game.events.collision.iter() {
        if game
            .logics
            .collision
            .events()
            .iter()
            .any(|event| col_event == event && *room == current_room)
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
    clear_background(game.draw.background_color);
    let current_room = game.get_current_room();

    let mut col_data = game.logics.collision.data_iter();

    while let Some((id, col_data)) = col_data.next() {
        match (id, col_data) {
            (ColIdent::Position(pos), TileMapColData::Position { id: tile, .. }) => {
                let color = game
                    .draw
                    .colors
                    .get(&EntID::Tile(*tile))
                    .unwrap_or_else(|| panic!("tile {} color undefined", tile.idx()));
                draw_rectangle(
                    pos.x as f32 * TILE_SIZE as f32,
                    pos.y as f32 * TILE_SIZE as f32,
                    TILE_SIZE as f32,
                    TILE_SIZE as f32,
                    *color,
                );
            }
            (ColIdent::EntIdx(idx), TileMapColData::Ent { pos, id: ent, .. }) => match ent {
                ColEntType::Player => {
                    let color = game
                        .draw
                        .colors
                        .get(&EntID::Player)
                        .expect("player color not set");
                    draw_rectangle(
                        pos.x as f32 * TILE_SIZE as f32,
                        pos.y as f32 * TILE_SIZE as f32,
                        TILE_SIZE as f32,
                        TILE_SIZE as f32,
                        *color,
                    );
                    draw_text(
                        "P",
                        pos.x as f32 * TILE_SIZE as f32 + 4.0,
                        (pos.y as f32 + 1.0) * TILE_SIZE as f32 - 4.0,
                        TILE_SIZE as f32,
                        WHITE,
                    );
                }
                ColEntType::Character => {
                    let character = game.state.rooms[current_room].chars[idx - 1].0;
                    let color = game
                        .draw
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
                    draw_text(
                        "C",
                        pos.x as f32 * TILE_SIZE as f32 + 4.0,
                        (pos.y as f32 + 1.0) * TILE_SIZE as f32 - 4.0,
                        TILE_SIZE as f32,
                        WHITE,
                    );
                }
            },
            _ => unreachable!(),
        }
    }

    for RsrcEvent {
        pool,
        transaction,
        event_type,
    } in game.logics.resources.events()
    {
        let timer = |x, y, name| {
            Box::new(move || {
                draw_text(
                    &format! {"{} get!", name},
                    x as f32 * TILE_SIZE as f32,
                    y as f32 * TILE_SIZE as f32,
                    (TILE_SIZE / 2) as f32,
                    WHITE,
                )
            })
        };
        if *event_type == ResourceEventType::PoolUpdated {
            match transaction {
                Transaction::Change(_) => {
                    if pool.attached_to == EntID::Player {
                        if let TileMapColData::Ent { pos, .. } =
                            game.logics.collision.get_ident_data(ColIdent::EntIdx(0))
                        {
                            game.draw
                                .draw_timer
                                .push((timer(pos.x, pos.y, pool.rsrc.name()), 120));
                        }
                    }
                }
                Transaction::Trade(_, other) => {
                    if pool.attached_to == EntID::Player || other.attached_to == EntID::Player {
                        if let TileMapColData::Ent { pos, .. } =
                            game.logics.collision.get_ident_data(ColIdent::EntIdx(0))
                        {
                            game.draw
                                .draw_timer
                                .push((timer(pos.x, pos.y, pool.rsrc.name()), 120));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    let mut i = 0;
    while i < game.draw.draw_timer.len() {
        let (_, timer) = game.draw.draw_timer[i];
        if timer == 0 {
            let _ = game.draw.draw_timer.remove(i);
        } else {
            i += 1;
        }
    }

    for (event, timer) in game.draw.draw_timer.iter_mut() {
        *timer -= 1;
        event();
    }
}

fn setup(game: &mut Game) {
    game.logics
        .collision
        .clear_and_resize_map(WORLD_SIZE, WORLD_SIZE);
    let current_room = game.get_current_room();

    entities::load_room(&mut game.state, &mut game.logics, current_room);

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
