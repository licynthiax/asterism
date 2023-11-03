#![allow(clippy::new_without_default)]
#![allow(clippy::upper_case_acronyms)]

use asterism::{
    collision::Contact,
    control::{KeyboardControl, MacroquadInputWrapper},
    graphics::draw::{self, Draw},
    physics::PointPhysics,
    resources::QueuedResources,
    Event,
};
use macroquad::prelude::*;

mod entities;
mod events;
mod types;

// reexports
pub use asterism::collision::{AabbColData, AabbCollision, CollisionReaction};
pub use asterism::control::{Action, ControlEventType, ControlReaction, Values};
pub use asterism::physics::{PhysicsEvent, PhysicsReaction, PointPhysData};
pub use asterism::resources::{ResourceEventType, ResourceReaction, Transaction};
pub use asterism::{LendingIterator, Logic};
pub use events::*;
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

pub enum LogicsList {
    Collision,
    Physics,
    Resources,
    Control,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EntID {
    Wall(WallID),
    Ball(BallID),
    Paddle(PaddleID),
    Score(ScoreID),
}

impl EntID {
    pub fn get_col_type(&self) -> CollisionEnt {
        match self {
            Self::Ball(_) => CollisionEnt::Ball,
            Self::Paddle(_) => CollisionEnt::Paddle,
            Self::Wall(_) => CollisionEnt::Wall,
            Self::Score(_) => {
                panic!("can't collide with a score!")
            }
        }
    }

    pub fn get_type(&self) -> EntType {
        match self {
            Self::Ball(_) => EntType::Ball,
            Self::Paddle(_) => EntType::Paddle,
            Self::Wall(_) => EntType::Wall,
            Self::Score(_) => EntType::Score,
        }
    }

    fn get_idx(&self) -> usize {
        match self {
            EntID::Wall(id) => id.idx(),
            EntID::Ball(id) => id.idx(),
            EntID::Paddle(id) => id.idx(),
            EntID::Score(id) => id.idx(),
        }
    }

    pub fn get_ball(&self) -> Option<BallID> {
        match self {
            Self::Ball(id) => Some(*id),
            _ => None,
        }
    }
    pub fn get_wall(&self) -> Option<WallID> {
        match self {
            Self::Wall(id) => Some(*id),
            _ => None,
        }
    }
    pub fn get_paddle(&self) -> Option<PaddleID> {
        match self {
            Self::Paddle(id) => Some(*id),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EntType {
    Wall,
    Paddle,
    Ball,
    Score,
}

impl From<CollisionEnt> for EntType {
    fn from(col: CollisionEnt) -> Self {
        match col {
            CollisionEnt::Paddle => Self::Paddle,
            CollisionEnt::Wall => Self::Wall,
            CollisionEnt::Ball => Self::Ball,
        }
    }
}

#[derive(Clone)]
pub enum Ent {
    Wall(Wall),
    Ball(Ball),
    Paddle(Paddle),
    Score(Score),
}

impl std::fmt::Debug for Ent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ent_type = match self {
            Ent::Wall(_) => "Ent::Wall",
            Ent::Ball(_) => "Ent::Ball",
            Ent::Paddle(_) => "Ent::Paddle",
            Ent::Score(_) => "Ent::Score",
        };
        f.write_str(ent_type)
    }
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
    pub fn get_col_idx(&self, id: EntID) -> usize {
        match id.get_col_type() {
            CollisionEnt::Paddle => self
                .paddles
                .iter()
                .position(|&p| p == id.get_paddle().unwrap())
                .unwrap(),
            CollisionEnt::Wall => {
                self.walls
                    .iter()
                    .position(|&w| w == id.get_wall().unwrap())
                    .unwrap()
                    + self.paddles.len()
            }
            CollisionEnt::Ball => {
                self.balls
                    .iter()
                    .position(|&b| b == id.get_ball().unwrap())
                    .unwrap()
                    + self.paddles.len()
                    + self.walls.len()
            }
        }
    }

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
        if idx - (self.balls.len() as isize) < 0 {
            let ball = self.balls[idx as usize];
            return EntID::Ball(ball);
        }
        idx -= self.balls.len() as isize;
        let score = self.scores[idx as usize];
        EntID::Score(score)
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
    pub draw: Draw<LogicsList>,
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

pub async fn run(mut game: Game) {
    use std::collections::VecDeque;
    let mut fps = VecDeque::with_capacity(1000);
    loop {
        if is_key_down(KeyCode::Escape) {
            break;
        }

        if fps.len() == fps.capacity() {
            fps.pop_front();
            fps.push_back(get_fps());
        } else {
            fps.push_back(get_fps());
        }

        draw(&mut game);

        // remove and add entities from previous frame
        game.state.remove_queue.sort_by(|a, b| {
            let a = a.get_idx();
            let b = b.get_idx();
            b.cmp(&a)
        });
        let remove_queue = std::mem::take(&mut game.state.remove_queue);
        for ent in remove_queue {
            match ent {
                EntID::Wall(wall) => game.remove_wall(wall),
                EntID::Ball(ball) => game.remove_ball(ball),
                EntID::Paddle(paddle) => game.remove_paddle(paddle),
                EntID::Score(score) => game.remove_score(score),
            };
        }

        // add
        let add_queue = std::mem::take(&mut game.state.add_queue);
        for ent in add_queue {
            match ent {
                Ent::Wall(wall) => {
                    game.add_wall(wall);
                }
                Ent::Ball(ball) => {
                    game.add_ball(ball);
                }
                Ent::Paddle(paddle) => {
                    game.add_paddle(paddle);
                }
                Ent::Score(score) => {
                    game.add_score(score);
                }
            }
        }

        control(&mut game);
        physics(&mut game);
        collision(&mut game);
        resources(&mut game);

        next_frame().await;
    }
    println!("{}", fps.iter().sum::<i32>() / fps.len() as i32);
}

fn control(game: &mut Game) {
    game.logics.control.update(&());

    for (event_data, actions) in game.events.control.iter() {
        let events = game.logics.control.events();
        match event_data {
            EngineCtrlEvent::MovePaddle(paddle, id) => {
                let relevant = events.iter().any(|e| {
                    e.action_id == *id
                        && e.set == paddle.idx()
                        && e.event_type == ControlEventType::KeyHeld
                });
                if relevant {
                    for action in actions {
                        action.perform_action(&mut game.state, &mut game.logics);
                    }
                }
            }
            EngineCtrlEvent::ServePressed(paddle, id) => {
                let relevant = events.iter().any(|e| {
                    e.action_id == *id
                        && e.set == paddle.idx()
                        && e.event_type == ControlEventType::KeyPressed
                });
                if relevant {
                    for action in actions {
                        action.perform_action(&mut game.state, &mut game.logics);
                    }
                }
            }
        }
    }
}

fn physics(game: &mut Game) {
    game.logics.physics.update();

    let mut ans = game.logics.physics.data_iter();
    // dbg!(ans.next().is_some());

    // update physics positions to collision
    while let Some((idx, data)) = ans.next() {
        let idx = game.state.get_col_idx(EntID::Ball(BallID::new(idx)));

        game.logics
            .collision
            .handle_predicate(&CollisionReaction::SetPos(idx, *data.pos));
        game.logics
            .collision
            .handle_predicate(&CollisionReaction::SetVel(idx, *data.vel));
    }
}

fn collision(game: &mut Game) {
    game.logics.collision.update();

    // update collision positions to physics
    let paddles_len = game.state.paddles.len();
    let walls_len = game.state.walls.len();
    let mut ans = game.logics.collision.data_iter();

    while let Some((idx, data)) = ans.next() {
        if idx >= paddles_len + walls_len {
            let idx = idx - paddles_len - walls_len;
            game.logics
                .physics
                .handle_predicate(&PhysicsReaction::SetPos(
                    idx,
                    *data.center - *data.half_size,
                ));
        }
    }

    for (event_data, actions) in game.events.collision.iter() {
        let events = game.logics.collision.events().iter();

        // "how should we filter these events?"
        let events: Vec<Contact> = match event_data {
            EngineCollisionEvent::Match(fst, snd) => match (fst, snd) {
                (EntityMatch::ByID(id1), EntityMatch::ByID(id2)) => events
                    .filter(|Contact { i, j, .. }| {
                        game.state.get_col_idx(*id1) == *i && game.state.get_col_idx(*id2) == *j
                    })
                    .copied()
                    .collect(),
                (EntityMatch::ByID(id1), EntityMatch::ByType(type2)) => events
                    .filter(|Contact { i, j, .. }| {
                        let ty2: EntType = game.state.get_id(*j).get_col_type().into();
                        game.state.get_col_idx(*id1) == *i && ty2 == *type2
                    })
                    .copied()
                    .collect(),
                (EntityMatch::ByID(id1), EntityMatch::All) => events
                    .filter(|Contact { i, .. }| game.state.get_col_idx(*id1) == *i)
                    .copied()
                    .collect(),
                (EntityMatch::ByID(id1), EntityMatch::Filter(filter2)) => events
                    .filter(|Contact { i, .. }| game.state.get_col_idx(*id1) == *i)
                    .filter(|Contact { j, .. }| filter2(game.state.get_id(*j)))
                    .copied()
                    .collect(),
                (EntityMatch::ByType(type1), EntityMatch::ByID(id2)) => events
                    .filter(|Contact { i, j, .. }| {
                        let ty1: EntType = game.state.get_id(*i).get_col_type().into();
                        ty1 == *type1 && game.state.get_col_idx(*id2) == *j
                    })
                    .copied()
                    .collect(),
                (EntityMatch::ByType(type1), EntityMatch::ByType(type2)) => events
                    .filter(|Contact { i, j, .. }| {
                        let ty1: EntType = game.state.get_id(*i).get_col_type().into();
                        let ty2: EntType = game.state.get_id(*j).get_col_type().into();
                        ty1 == *type1 && ty2 == *type2
                    })
                    .copied()
                    .collect(),
                (EntityMatch::ByType(type1), EntityMatch::All) => events
                    .filter(|Contact { i, .. }| {
                        let ty1: EntType = game.state.get_id(*i).get_col_type().into();
                        ty1 == *type1
                    })
                    .copied()
                    .collect(),
                (EntityMatch::ByType(type1), EntityMatch::Filter(filter2)) => events
                    .filter(|Contact { i, .. }| {
                        let ty1: EntType = game.state.get_id(*i).get_col_type().into();
                        ty1 == *type1
                    })
                    .filter(|Contact { j, .. }| filter2(game.state.get_id(*j)))
                    .copied()
                    .collect(),
                (EntityMatch::All, EntityMatch::ByID(id2)) => events
                    .filter(|Contact { j, .. }| game.state.get_col_idx(*id2) == *j)
                    .copied()
                    .collect(),
                (EntityMatch::All, EntityMatch::ByType(type2)) => events
                    .filter(|Contact { j, .. }| {
                        let ty2: EntType = game.state.get_id(*j).get_col_type().into();
                        ty2 == *type2
                    })
                    .copied()
                    .collect(),
                (EntityMatch::All, EntityMatch::All) => events.copied().collect(),
                (EntityMatch::All, EntityMatch::Filter(filter2)) => events
                    .filter(|Contact { j, .. }| filter2(game.state.get_id(*j)))
                    .copied()
                    .collect(),
                (EntityMatch::Filter(filter1), EntityMatch::ByID(id2)) => events
                    .filter(|Contact { i, .. }| filter1(game.state.get_id(*i)))
                    .filter(|Contact { j, .. }| game.state.get_col_idx(*id2) == *j)
                    .copied()
                    .collect(),
                (EntityMatch::Filter(filter1), EntityMatch::ByType(type2)) => events
                    .filter(|Contact { i, .. }| filter1(game.state.get_id(*i)))
                    .filter(|Contact { j, .. }| {
                        let ty2: EntType = game.state.get_id(*j).get_col_type().into();
                        ty2 == *type2
                    })
                    .copied()
                    .collect(),
                (EntityMatch::Filter(filter1), EntityMatch::All) => events
                    .filter(|Contact { i, .. }| filter1(game.state.get_id(*i)))
                    .copied()
                    .collect(),
                (EntityMatch::Filter(filter1), EntityMatch::Filter(filter2)) => events
                    .filter(|Contact { i, j, .. }| {
                        filter1(game.state.get_id(*i)) && filter2(game.state.get_id(*j))
                    })
                    .copied()
                    .collect(),
            },
            EngineCollisionEvent::Filter(filter) => events
                .filter(|Contact { i, j, .. }| filter(game.state.get_id(*i), game.state.get_id(*j)))
                .copied()
                .collect(),
        };

        for event in events {
            for action in actions.iter() {
                match action {
                    EngineAction::BounceBall(ball, None) => {
                        let j = game.state.get_id(event.j);
                        EngineAction::BounceBall(*ball, Some(j))
                            .perform_action(&mut game.state, &mut game.logics);
                    }
                    EngineAction::RemoveEntity(None) => {
                        let j = game.state.get_id(event.j);
                        EngineAction::RemoveEntity(Some(EntityMatch::ByID(j)))
                            .perform_action(&mut game.state, &mut game.logics);
                    }
                    _ => action.perform_action(&mut game.state, &mut game.logics),
                }
            }
        }
    }
}

fn resources(game: &mut Game) {
    game.logics.resources.update();

    for (event_data, actions) in game.events.resources.iter() {
        let events = game.logics.resources.events();
        let relevant = match event_data {
            EngineRsrcEvent::ScoreIncreased(score) => {
                let score = RsrcPool::Score(*score);
                events.iter().any(|e| {
                    e.pool == score
                        && *e.get_type() == ResourceEventType::PoolUpdated
                        && e.transaction == Transaction::Change(1)
                })
            }
            EngineRsrcEvent::ScoreReset(score) => {
                let score = RsrcPool::Score(*score);

                events.iter().any(|e| {
                    e.pool == score
                        && *e.get_type() == ResourceEventType::PoolUpdated
                        && e.transaction == Transaction::Set(0)
                })
            }
            EngineRsrcEvent::ScoreEquals(score, v) => {
                let score = RsrcPool::Score(*score);
                let val = game.logics.resources.get_ident_data(score).val;
                let events = game.logics.resources.events();

                events.iter().any(|e| {
                    e.pool == score && *e.get_type() == ResourceEventType::PoolUpdated && val == *v
                })
            }
        };
        if relevant {
            for action in actions.iter() {
                action.perform_action(&mut game.state, &mut game.logics);
            }
        }
    }
}

pub fn draw(game: &mut Game) {
    use draw::DrawType;

    let mut positions = Vec::new();
    for (i, draw_type) in game.draw.positions.iter().enumerate() {
        match draw_type {
            DrawType::FromLogic(logic) => {
                let mut position = Vec2::ZERO;

                // this engine only connects drawing to collision logics
                if let LogicsList::Collision = logic {
                    match game.state.get_id(i) {
                        EntID::Score(_) => {}
                        _ => {
                            let col = game.logics.collision.get_ident_data(i);
                            position = *col.center - *col.half_size;
                        }
                    }
                }
                positions.push(position);
            }
            // this engine only connects drawing to collision logics
            DrawType::Offset(_, _) => {}
            DrawType::FixedPoint(position) => {
                if let EntID::Score(s) = game.state.get_id(i) {
                    let score = game.logics.resources.get_ident_data(RsrcPool::Score(s));
                    let drawable = &mut game.draw.drawables[i];
                    if let draw::Drawable::Text(text, _, _) = drawable {
                        *text = format!("{}", score.val);
                    }

                    positions.push(*position);
                }
            }
        }
    }
    game.draw.draw(positions);
}
