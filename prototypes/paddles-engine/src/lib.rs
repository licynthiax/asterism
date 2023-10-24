#![allow(clippy::new_without_default)]
#![allow(clippy::upper_case_acronyms)]

use asterism::{
    control::{KeyboardControl, MacroquadInputWrapper},
    graphics::draw::{self, Draw},
    physics::PointPhysics,
    resources::QueuedResources,
};
use asterism::{LendingIterator, Logic};
use macroquad::prelude::*;

mod entities;
pub mod events;
mod types;
use events::*;

// reexports
pub use asterism::collision::{AabbColData, AabbCollision, CollisionReaction};
pub use asterism::control::{Action, ControlEventType, ControlReaction, Values};
pub use asterism::physics::{PhysicsEvent, PhysicsReaction, PointPhysData};
pub use asterism::resources::{ResourceEventType, ResourceReaction, Transaction};
pub use types::*;

pub struct Logics {
    pub collision: AabbCollision<CollisionEnt>,
    pub physics: PointPhysics,
    pub resources: QueuedResources<RsrcPool, u16>,
    pub control: KeyboardControl<ActionID, MacroquadInputWrapper>,
}

impl Logics {
    fn new() -> Self {
        Self {
            collision: AabbCollision::new(),
            physics: PointPhysics::new(),
            resources: QueuedResources::new(),
            control: KeyboardControl::new(),
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum EntID {
    Wall(WallID),
    Ball(BallID),
    Paddle(PaddleID),
    Score(ScoreID),
}

#[derive(Clone)]
pub enum Ent {
    Wall(Wall),
    Ball(Ball),
    Paddle(Paddle),
    Score(Score),
}

#[derive(Default)]
pub struct State {
    remove_queue: Vec<EntID>,
    add_queue: Vec<Ent>,
    paddles: Vec<PaddleID>,
    walls: Vec<WallID>,
    balls: Vec<BallID>,
    scores: Vec<ScoreID>,
    paddle_id_max: usize,
    ball_id_max: usize,
    wall_id_max: usize,
    score_id_max: usize,
}

impl State {
    pub fn get_col_idx(&self, i: usize, col: CollisionEnt) -> usize {
        match col {
            CollisionEnt::Paddle => i,
            CollisionEnt::Wall => i + self.paddles.len(),
            CollisionEnt::Ball => i + self.paddles.len() + self.walls.len(),
        }
    }

    // i hope this logic is correct...
    pub fn get_id(&self, idx: usize) -> EntID {
        let mut idx = idx as isize;
        if idx - (self.paddles.len() as isize) < 0 {
            let paddle = self.paddles[idx as usize];
            return EntID::Paddle(paddle);
        }
        idx -= self.paddles.len() as isize;
        if idx - (self.walls.len() as isize) < 0 {
            let wall = self.walls[idx as usize];
            return EntID::Wall(wall);
        }
        idx -= self.walls.len() as isize;
        let ball = self.balls[idx as usize];
        EntID::Ball(ball)
    }

    pub fn paddles(&self) -> &[PaddleID] {
        &self.paddles
    }
    pub fn walls(&self) -> &[WallID] {
        &self.walls
    }
    pub fn balls(&self) -> &[BallID] {
        &self.balls
    }
    pub fn scores(&self) -> &[ScoreID] {
        &self.scores
    }

    pub fn queue_remove(&mut self, ent: EntID) {
        if !self.remove_queue.iter().any(|id| ent == *id) {
            self.remove_queue.push(ent);
        }
    }
    pub fn queue_add(&mut self, ent: Ent) {
        self.add_queue.push(ent);
    }
}

pub struct Game {
    pub state: State,
    pub logics: Logics,
    pub events: Events,
    pub draw: Draw,
}

impl Game {
    pub fn new() -> Self {
        let mut draw = Draw::new();
        draw.background_color = draw::DARKBLUE;

        Self {
            state: State::default(),
            logics: Logics::new(),
            events: Events::new(),
            draw,
        }
    }
}

// macro to make matching entities to statements take up less space
macro_rules! match_ent {
    (
        $match_to:expr,
        $wall:ident: $wall_block:block,
        $ball:ident: $ball_block:block,
        $paddle:ident: $paddle_block:block,
        $score:ident: $score_block:block
    ) => {
        match $match_to {
            Ent::Wall($wall) => $wall_block,
            Ent::Ball($ball) => $ball_block,
            Ent::Paddle($paddle) => $paddle_block,
            Ent::Score($score) => $score_block,
        }
    };
    (
        $match_to:expr,
        only $ent:ident: $ent_block:block
    ) => {
        match $match_to {
            EntID::Wall($ent) => $ent_block,
            EntID::Ball($ent) => $ent_block,
            EntID::Paddle($ent) => $ent_block,
            EntID::Score($ent) => $ent_block,
        }
    };
}

// macro to make matching entity ids to statements less verbose
macro_rules! match_ent_id {
    (
        $match_to:expr,
        $wall:ident: $wall_block:block,
        $ball:ident: $ball_block:block,
        $paddle:ident: $paddle_block:block,
        $score:ident: $score_block:block
    ) => {
        match $match_to {
            EntID::Wall($wall) => $wall_block,
            EntID::Ball($ball) => $ball_block,
            EntID::Paddle($paddle) => $paddle_block,
            EntID::Score($score) => $score_block,
        }
    };
    (
        $match_to:expr,
        only $ent:ident: $ent_block:block
    ) => {
        match $match_to {
            EntID::Wall($ent) => $ent_block,
            EntID::Ball($ent) => $ent_block,
            EntID::Paddle($ent) => $ent_block,
            EntID::Score($ent) => $ent_block,
        }
    };
}

pub async fn run(mut game: Game) {
    loop {
        if is_key_down(KeyCode::Escape) {
            break;
        }

        draw(&mut game);

        // remove and add entities from previous frame
        game.state.remove_queue.sort_by(|a, b| {
            let a = match_ent_id!(a, only ent: { ent.idx() } );
            let b = match_ent_id!(b, only ent: { ent.idx() });
            a.cmp(&b)
        });
        let remove_queue = std::mem::take(&mut game.state.remove_queue);
        for ent in remove_queue {
            match_ent_id!(
                ent,
                wall: { game.remove_wall(wall); },
                ball: { game.remove_ball(ball); },
                paddle: { game.remove_paddle(paddle); },
                score: { game.remove_score(score); }
            );
        }

        // add
        let add_queue = std::mem::take(&mut game.state.add_queue);
        for ent in add_queue {
            match_ent!(
                ent,
                wall: { game.add_wall(wall); },
                ball: { game.add_ball(ball); },
                paddle: { game.add_paddle(paddle); },
                score: { game.add_score(score); }
            );
        }

        control(&mut game);
        physics(&mut game);
        collision(&mut game);
        resources(&mut game);

        next_frame().await;
    }
}

fn control(game: &mut Game) {
    game.logics.control.update(&());

    // todo: idr what was happening here lol
    let events = game.logics.control.events();
    for e in events.iter() {
        for (event_data, engine_action) in game.events.control.iter() {
            match event_data {
                EngineCtrlEvents::MovePaddle(paddle, id) => {
                    if e.action_id == *id {
                        engine_action.perform_action(&mut game.state, &mut game.logics);
                    }
                }
                EngineCtrlEvents::ServePressed(paddle, id) => {
                    // todo
                }
            }
        }
    }
}

fn physics(game: &mut Game) {
    game.logics.physics.update();

    let mut ans = game.logics.physics.data_iter();

    // update physics positions to collision
    while let Some((idx, data)) = ans.next() {
        let idx = game.state.get_col_idx(idx, CollisionEnt::Ball);

        game.logics
            .collision
            .handle_predicate(&CollisionReaction::SetPos(idx, *data.pos));
    }
}

fn collision(game: &mut Game) {
    game.logics.collision.update();

    // update collision positions to physics
    let paddles_len = game.state.paddles.len();
    let walls_len = game.state.walls.len();
    let mut ans = game.logics.collision.data_iter();

    while let Some((idx, data)) = ans.next() {
        if idx > paddles_len + walls_len {
            let idx = idx - paddles_len - walls_len;
            game.logics
                .physics
                .handle_predicate(&PhysicsReaction::SetPos(
                    idx,
                    *data.center - *data.half_size,
                ));
        }
    }
}

fn resources(game: &mut Game) {
    game.logics.resources.update();
}

pub fn draw(game: &mut Game) {
    let mut col_data = game.logics.collision.data_iter().enumerate();
    while let Some((i, (_, col))) = col_data.next() {
        let center = col.center;
        let hs = col.half_size;
        let rect = draw::Rect::new(center.x - hs.x, center.y - hs.y, hs.x * 2.0, hs.y * 2.0);
        game.draw.update_rect(i, rect);
    }
    game.draw.draw();
    // draw_text(&get_fps().to_string(), 0.0, 16.0, 16.0, BLACK);
}
