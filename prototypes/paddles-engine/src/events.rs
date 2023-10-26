use crate::types::*;
use asterism::Logic;
use macroquad::math::Vec2;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum EngineCtrlEvent {
    MovePaddle(PaddleID, ActionID),
    ServePressed(PaddleID, ActionID),
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum EngineCollisionEvent {
    BallPaddleCollide(BallID, PaddleID),
    BallWallCollide(BallID, WallID),
    BallScoreWallCollide(BallID, WallID),
    PaddleCollisions,
    WallCollisions,
}

#[derive(Clone, Debug)]
pub enum EngineAction {
    // can only bounce between balls, walls, and paddles
    BounceBall(BallID, crate::EntID),
    SetBallVel(BallID, Vec2),
    SetBallPos(BallID, Vec2),
    SetPaddlePos(PaddleID, Vec2),
    MovePaddleBy(PaddleID, Vec2),
    SetKeyValid(PaddleID, ActionID),
    SetKeyInvalid(PaddleID, ActionID),
    ChangeScore(ScoreID, u16),
    RemoveEntity(crate::EntID),
    AddEntity(crate::Ent),
}

impl EngineAction {
    pub(crate) fn perform_action(&self, state: &mut crate::State, logics: &mut crate::Logics) {
        match self {
            Self::BounceBall(ball, ent) => {
                let ball_idx = state.get_col_idx(ball.idx(), CollisionEnt::Ball);
                let ent_idx = match ent {
                    crate::EntID::Wall(wall) => state.get_col_idx(wall.idx(), CollisionEnt::Wall),
                    crate::EntID::Ball(ball) => state.get_col_idx(ball.idx(), CollisionEnt::Ball),
                    crate::EntID::Paddle(paddle) => {
                        state.get_col_idx(paddle.idx(), CollisionEnt::Paddle)
                    }
                    crate::EntID::Score(_) => panic!("cannot bounce off a score"),
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
                let col_idx = state.get_col_idx(ball.idx(), CollisionEnt::Ball);
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
                let col_idx = state.get_col_idx(paddle.idx(), CollisionEnt::Paddle);
                logics
                    .collision
                    .handle_predicate(&crate::CollisionReaction::SetCenter(col_idx, *pos));
            }
            Self::MovePaddleBy(paddle, delta) => {
                let col_idx = state.get_col_idx(paddle.idx(), CollisionEnt::Paddle);
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
            Self::RemoveEntity(ent_id) => state.queue_remove(*ent_id),
            Self::AddEntity(ent) => state.queue_add(ent.clone()),
        }
    }
}

pub struct Events {
    pub(crate) control: Vec<(EngineCtrlEvent, Vec<EngineAction>)>,
    pub(crate) collision: Vec<(EngineCollisionEvent, Vec<EngineAction>)>,
}

impl Events {
    pub fn new() -> Self {
        Self {
            control: Vec::new(),
            collision: Vec::new(),
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

    pub fn add_col_event(&mut self, event: EngineCollisionEvent, reaction: EngineAction) {
        if let Some(idx) = self.collision.iter().position(|(e, _)| *e == event) {
            let (_, reactions) = &mut self.collision[idx];
            reactions.push(reaction);
        } else {
            self.collision.push((event, vec![reaction]));
        }
    }
}
