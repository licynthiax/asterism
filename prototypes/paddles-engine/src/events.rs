use crate::{types::*, Ent, EntID, EntType};
use asterism::Logic;
use macroquad::math::Vec2;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum EngineCtrlEvent {
    MovePaddle(PaddleID, ActionID),
    ServePressed(PaddleID, ActionID),
}

pub enum EngineCollisionEvent {
    Match(EntityMatch, EntityMatch),
    Filter(Box<dyn Fn(EntID, EntID) -> bool>),
}

pub enum EntityMatch {
    ByID(EntID),
    ByType(EntType),
    All,
    Filter(Box<dyn Fn(EntID) -> bool>),
}

impl std::fmt::Debug for EntityMatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityMatch::ByID(id) => f.write_fmt(format_args!("EntityMatch::ByID({:?})", id)),
            EntityMatch::ByType(ty) => f.write_fmt(format_args!("EntityMatch::ByID({:?})", ty)),
            EntityMatch::All => f.write_str("EntityMatch::All"),
            EntityMatch::Filter(_) => f.write_str("EntityMatch::Filter(_)"),
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum EngineRsrcEvent {
    ScoreIncreased(ScoreID),
    ScoreReset(ScoreID),
    ScoreEquals(ScoreID, i16),
}

#[derive(Debug)]
pub enum EngineAction {
    // can only bounce between balls, walls, and paddles
    BounceBall(BallID, Option<crate::EntID>),
    SetBallVel(BallID, Vec2),
    SetBallPos(BallID, Vec2),
    SetPaddlePos(PaddleID, Vec2),
    MovePaddleBy(PaddleID, Vec2),
    SetKeyValid(PaddleID, ActionID),
    SetKeyInvalid(PaddleID, ActionID),
    ChangeScoreBy(ScoreID, i16),
    ChangeScore(ScoreID, i16),
    RemoveEntity(Option<EntityMatch>),
    AddEntity(Ent),
}

impl EngineAction {
    pub(crate) fn perform_action(&self, state: &mut crate::State, logics: &mut crate::Logics) {
        match self {
            Self::BounceBall(_, None) => {} // no entity to be bounced off of
            Self::BounceBall(ball, Some(ent)) => {
                let ball_idx = state.get_col_idx((*ball).into());
                let ent_idx = state.get_col_idx(*ent);

                let sides_touched = logics.collision.sides_touched(ball_idx, ent_idx);
                let vals = logics.physics.get_ident_data_mut(ball.idx());
                if sides_touched.y != 0.0 {
                    vals.vel.y *= -1.0;
                }
                if sides_touched.x != 0.0 {
                    vals.vel.x *= -1.0;
                }
            }
            Self::SetBallPos(ball, pos) => {
                logics
                    .physics
                    .handle_predicate(&crate::PhysicsReaction::SetPos(ball.idx(), *pos));
                let col_idx = state.get_col_idx((*ball).into());
                logics
                    .collision
                    .handle_predicate(&crate::CollisionReaction::SetCenter(col_idx, *pos));
            }
            Self::SetBallVel(ball, vel) => {
                logics
                    .physics
                    .handle_predicate(&crate::PhysicsReaction::SetVel(ball.idx(), *vel));
            }
            Self::ChangeScore(score, val) => {
                logics.resources.handle_predicate(&(
                    crate::RsrcPool::Score(*score),
                    asterism::resources::Transaction::Set(*val),
                ));
            }
            Self::ChangeScoreBy(score, val) => {
                logics.resources.handle_predicate(&(
                    crate::RsrcPool::Score(*score),
                    asterism::resources::Transaction::Change(*val),
                ));
            }
            Self::SetPaddlePos(paddle, pos) => {
                let col_idx = state.get_col_idx((*paddle).into());
                logics
                    .collision
                    .handle_predicate(&crate::CollisionReaction::SetCenter(col_idx, *pos));
            }
            Self::MovePaddleBy(paddle, delta) => {
                let col_idx = state.get_col_idx((*paddle).into());
                let new_pos = *logics.collision.get_ident_data(col_idx).center + *delta;
                logics
                    .collision
                    .handle_predicate(&crate::CollisionReaction::SetVel(col_idx, *delta));
                logics
                    .collision
                    .handle_predicate(&crate::CollisionReaction::SetCenter(col_idx, new_pos))
            }
            Self::SetKeyValid(set, action) => {
                logics
                    .control
                    .handle_predicate(&crate::ControlReaction::SetKeyValid(set.idx(), *action));
            }
            Self::SetKeyInvalid(set, action) => {
                logics
                    .control
                    .handle_predicate(&crate::ControlReaction::SetKeyInvalid(set.idx(), *action));
            }
            Self::RemoveEntity(None) => {} // no entity to remove
            Self::RemoveEntity(Some(match_ent)) => match match_ent {
                EntityMatch::ByID(id) => state.queue_remove(*id),
                EntityMatch::ByType(ty) => match ty {
                    EntType::Wall => {
                        for wall in state.walls.clone() {
                            state.queue_remove(wall.into());
                        }
                    }
                    EntType::Paddle => {
                        for paddle in state.paddles.clone() {
                            state.queue_remove(paddle.into());
                        }
                    }
                    EntType::Ball => {
                        for ball in state.balls.clone() {
                            state.queue_remove(ball.into());
                        }
                    }
                    EntType::Score => {
                        for score in state.scores.clone() {
                            state.queue_remove(score.into());
                        }
                    }
                },
                EntityMatch::All => {
                    for wall in state.walls.clone() {
                        state.queue_remove(wall.into());
                    }
                    for paddle in state.paddles.clone() {
                        state.queue_remove(paddle.into());
                    }
                    for ball in state.balls.clone() {
                        state.queue_remove(ball.into());
                    }
                    for score in state.scores.clone() {
                        state.queue_remove(score.into());
                    }
                }
                EntityMatch::Filter(filter) => {
                    for wall in state
                        .walls
                        .clone()
                        .into_iter()
                        .map(|wall| wall.into())
                        .filter(|wall| filter(*wall))
                    {
                        state.queue_remove(wall);
                    }
                    for paddle in state
                        .paddles
                        .clone()
                        .into_iter()
                        .map(|paddle| paddle.into())
                        .filter(|paddle| filter(*paddle))
                    {
                        state.queue_remove(paddle);
                    }
                    for ball in state
                        .balls
                        .clone()
                        .into_iter()
                        .map(|ball| ball.into())
                        .filter(|ball| filter(*ball))
                    {
                        state.queue_remove(ball);
                    }
                    for score in state
                        .scores
                        .clone()
                        .into_iter()
                        .map(|score| score.into())
                        .filter(|score| filter(*score))
                    {
                        state.queue_remove(score);
                    }
                }
            },
            Self::AddEntity(ent) => state.queue_add(ent.clone()),
        }
    }
}

pub struct Events {
    pub(crate) control: Vec<(EngineCtrlEvent, Vec<EngineAction>)>,
    pub(crate) collision: Vec<(EngineCollisionEvent, Vec<EngineAction>)>,
    pub(crate) resources: Vec<(EngineRsrcEvent, Vec<EngineAction>)>,
}

impl Events {
    pub fn new() -> Self {
        Self {
            control: Vec::new(),
            collision: Vec::new(),
            resources: Vec::new(),
        }
    }

    pub fn add_ctrl_event(&mut self, event: EngineCtrlEvent, reaction: EngineAction) {
        if let Some(idx) = self.control.iter().position(|(e, _)| *e == event) {
            let (_, reactions) = &mut self.control[idx];
            reactions.push(reaction);
        } else {
            self.control.push((event, vec![reaction]));
        }
    }

    pub fn add_ctrl_events(&mut self, event: EngineCtrlEvent, mut reactions: Vec<EngineAction>) {
        if let Some(idx) = self.control.iter().position(|(e, _)| *e == event) {
            let (_, r) = &mut self.control[idx];
            r.append(&mut reactions);
        } else {
            self.control.push((event, reactions));
        }
    }

    pub fn add_col_events(&mut self, event: EngineCollisionEvent, reactions: Vec<EngineAction>) {
        self.collision.push((event, reactions));
    }

    pub fn add_rsrc_event(&mut self, event: EngineRsrcEvent, reaction: EngineAction) {
        if let Some(idx) = self.resources.iter().position(|(e, _)| *e == event) {
            let (_, reactions) = &mut self.resources[idx];
            reactions.push(reaction);
        } else {
            self.resources.push((event, vec![reaction]));
        }
    }
    pub fn add_rsrc_events(&mut self, event: EngineRsrcEvent, mut reactions: Vec<EngineAction>) {
        if let Some(idx) = self.resources.iter().position(|(e, _)| *e == event) {
            let (_, r) = &mut self.resources[idx];
            r.append(&mut reactions);
        } else {
            self.resources.push((event, reactions));
        }
    }
}
