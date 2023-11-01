use crate::{types::*, Ent, EntID};
use asterism::Logic;
use macroquad::math::Vec2;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum EngineCtrlEvent {
    MovePaddle(PaddleID, ActionID),
    ServePressed(PaddleID, ActionID),
}

pub enum EngineCollisionEvent {
    Match(CollisionEventMatch, CollisionEventMatch),
    Filter(Box<dyn Fn(EntID, EntID) -> bool>),
}

pub enum CollisionEventMatch {
    ByID(EntID),
    ByType(CollisionEnt),
    All,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum EngineRsrcEvent {
    ScoreIncreased(ScoreID),
    ScoreReset(ScoreID),
    ScoreEquals(ScoreID, u16),
}

#[derive(Clone, Debug)]
pub enum EngineAction {
    // can only bounce between balls, walls, and paddles
    BounceBall(BallID, Option<crate::EntID>),
    SetBallVel(BallID, Vec2),
    SetBallPos(BallID, Vec2),
    SetPaddlePos(PaddleID, Vec2),
    MovePaddleBy(PaddleID, Vec2),
    SetKeyValid(PaddleID, ActionID),
    SetKeyInvalid(PaddleID, ActionID),
    ChangeScoreBy(ScoreID, u16),
    ChangeScore(ScoreID, u16),
    RemoveEntity(Option<EntID>),
    AddEntity(Ent),
}

impl EngineAction {
    pub(crate) fn perform_action(&self, state: &mut crate::State, logics: &mut crate::Logics) {
        match self {
            Self::BounceBall(ball, ent) => {
                let ball_idx = state.get_col_idx((*ball).into());
                let ent_idx =
                    match ent.unwrap_or_else(|| panic!["no entity to be bounced off given!"]) {
                        crate::EntID::Wall(wall) => state.get_col_idx(wall.into()),
                        crate::EntID::Ball(ball) => state.get_col_idx(ball.into()),
                        crate::EntID::Paddle(paddle) => state.get_col_idx(paddle.into()),
                        crate::EntID::Score(_) => panic!("cannot bounce off a score!"),
                    };

                let sides_touched = logics.collision.sides_touched(ball_idx, ent_idx);

                let vals = logics.physics.get_ident_data(ball.idx());
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
                println!(
                    "score for p{} is now {}",
                    score.idx() + 1,
                    logics
                        .resources
                        .get_ident_data(crate::RsrcPool::Score(*score))
                        .val
                        + val
                );
            }
            Self::ChangeScoreBy(score, val) => {
                logics.resources.handle_predicate(&(
                    crate::RsrcPool::Score(*score),
                    asterism::resources::Transaction::Change(*val),
                ));
                println!(
                    "score for p{} is now {}",
                    score.idx() + 1,
                    logics
                        .resources
                        .get_ident_data(crate::RsrcPool::Score(*score))
                        .val
                        + val
                );
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
            Self::RemoveEntity(ent_id) => {
                let ent_id = ent_id.unwrap_or_else(|| panic!("no entity to remove given!"));
                state.queue_remove(ent_id);
            }
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

    pub fn add_ctrl_events(&mut self, event: EngineCtrlEvent, reactions: &[EngineAction]) {
        if let Some(idx) = self.control.iter().position(|(e, _)| *e == event) {
            let (_, r) = &mut self.control[idx];
            r.append(&mut reactions.to_owned());
        } else {
            self.control.push((event, reactions.to_owned()));
        }
    }

    pub fn add_col_events(&mut self, event: EngineCollisionEvent, reactions: &[EngineAction]) {
        self.collision.push((event, reactions.to_owned()));
    }

    pub fn add_rsrc_event(&mut self, event: EngineRsrcEvent, reaction: EngineAction) {
        if let Some(idx) = self.resources.iter().position(|(e, _)| *e == event) {
            let (_, reactions) = &mut self.control[idx];
            reactions.push(reaction);
        } else {
            self.resources.push((event, vec![reaction]));
        }
    }
    pub fn add_rsrc_events(&mut self, event: EngineRsrcEvent, reactions: &[EngineAction]) {
        if let Some(idx) = self.resources.iter().position(|(e, _)| *e == event) {
            let (_, r) = &mut self.control[idx];
            r.append(&mut reactions.to_owned());
        } else {
            self.resources.push((event, reactions.to_owned()));
        }
    }
}
