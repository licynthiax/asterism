//! adding/removing entities
use asterism::collision::CollisionData;
use asterism::graphics::draw;
use asterism::Logic;
use asterism::{collision::CollisionReaction, physics::PhysicsReaction, resources::PoolValues};
use macroquad::math::Vec2;

use crate::types::*;
use crate::{Game, LogicsList};

macro_rules! add_ent {
    (@attach $game:expr, $id:ident $gamefield:ident $ent_name:ident $ent_ty:ty; $id_ty:ty [collision: $col_data:expr]) => {
        let col_idx = match $col_data.id {
            CollisionEnt::Paddle => $game.state.$gamefield.len(),
            CollisionEnt::Wall => $game.state.$gamefield.len() + $game.state.paddles.len(),
            CollisionEnt::Ball => $game.state.$gamefield.len() + $game.state.paddles.len() + $game.state.walls.len()
        };

        let collision = &mut $game.logics.collision;
        let hs = $ent_name.size / 2.0;
        let center = $ent_name.pos + hs;
        collision.centers.insert(col_idx, center);
        collision.half_sizes.insert(col_idx, hs);
        collision.velocities.insert(col_idx, Vec2::ZERO);
        collision.metadata.insert(col_idx, $col_data);
    };


    (@attach $game:expr, $id:ident $gamefield:ident $ent_name:ident $ent_ty:ty; $id_ty:ty [control]) => {
        let control = &mut $game.logics.control;
        for (act_id, keycode, valid) in $ent_name.controls {
            control.add_key_map($id.idx(), keycode, act_id, valid);
        }
    };

    (@attach $game:expr, $id:ident $gamefield:ident $ent_name:ident $ent_ty:ty; $id_ty:ty [resource]) => {
        $game.logics.resources
            .items
            .insert(RsrcPool::Score($id), PoolValues{ val: $ent_name.value, min: <$ent_ty>::MIN, max: <$ent_ty>::MAX});
    };

    (@attach $game:expr, $id:ident $gamefield:ident $ent_name:ident $ent_ty:ty; $id_ty:ty [resource draw: $pos:expr]) => {
        let draw_idx = $game.state.balls.len() + $game.state.paddles.len() + $game.state.walls.len() + $game.state.scores.len();

        $game.draw.add_drawable(
            draw_idx,
            draw::DrawType::FixedPoint($pos),
            draw::Drawable::Text(
                "".to_string(),
                22,
                macroquad::color::WHITE
            )
        );
    };

    (@attach $game:expr, $id:ident $gamefield:ident $ent_name:ident $ent_ty:ty; $id_ty:ty [physics]) => {
        $game.logics.physics
            .add_physics_entity($ent_name.pos, $ent_name.vel, Vec2::ZERO);
    };

    // 'col_ent' is the collision entity because in this engine, everything that can be
    // collided with is also drawn
    (@attach $game:expr, $id:ident $gamefield:ident $ent_name:ident $ent_ty:ty; $id_ty:ty [collision draw: $col_ent:expr, $color:expr]) => {
        let col_idx = match $col_ent {
            CollisionEnt::Paddle => $game.state.$gamefield.len(),
            CollisionEnt::Wall => $game.state.$gamefield.len() + $game.state.paddles.len(),
            CollisionEnt::Ball => $game.state.$gamefield.len() + $game.state.paddles.len() + $game.state.walls.len()
        };

        $game.draw.add_drawable(col_idx, draw::DrawType::FromLogic(LogicsList::Collision),
            draw::Drawable::Rectangle($ent_name.size, $color));
    };

    ($gamefield:ident: ($ent_name:ident: $ent_ty:ty) -> $id_ty:ty {$([$($logic:tt)*]),*}, $game:expr, $id:ident) => {
        $(
            add_ent!(@attach $game, $id $gamefield $ent_name $ent_ty; $id_ty [$($logic)*]);
        )*

        $game.state.$gamefield.push($id);
    };
}

impl Game {
    pub fn add_paddle(&mut self, paddle: Paddle) -> PaddleID {
        let id = PaddleID::new(self.state.paddle_id_max);
        let col_data = CollisionData {
            solid: true,
            fixed: true,
            id: CollisionEnt::Paddle,
        };

        self.state.paddle_id_max += 1;
        add_ent!(
            paddles: (paddle: Paddle) -> PaddleID {
                [collision: col_data],
                [control],
                [collision draw: CollisionEnt::Paddle, draw::WHITE]
            }, self, id);
        id
    }

    pub fn add_ball(&mut self, ball: Ball) -> BallID {
        let id = BallID::new(self.state.ball_id_max);
        let col_data = CollisionData {
            solid: true,
            fixed: false,
            id: CollisionEnt::Ball,
        };
        self.state.ball_id_max += 1;
        add_ent!(
            balls: (ball: Ball) -> BallID {
                [collision: col_data],
                [physics],
                [collision draw: CollisionEnt::Ball, draw::YELLOW]
            }, self, id);
        id
    }

    pub fn add_wall(&mut self, wall: Wall) -> WallID {
        let id = WallID::new(self.state.wall_id_max);
        self.state.wall_id_max += 1;
        let col_data = CollisionData {
            solid: true,
            fixed: true,
            id: CollisionEnt::Wall,
        };

        add_ent!(
            walls: (wall: Wall) -> WallID {
                [collision: col_data],
                [collision draw: CollisionEnt::Wall, draw::SKYBLUE]
            }, self, id);

        id
    }

    pub fn add_score(&mut self, score: Score) -> ScoreID {
        let id = ScoreID::new(self.state.score_id_max);
        self.state.score_id_max += 1;
        add_ent!(
            scores: (score: Score) -> ScoreID {
                [resource],
                [resource draw: score.position]
            }, self, id);
        id
    }

    pub(crate) fn remove_paddle(&mut self, paddle: PaddleID) {
        let col_idx = self.state.get_col_idx(paddle.into());
        let ent_idx = self
            .state
            .paddles
            .iter()
            .position(|pid| *pid == paddle)
            .unwrap();

        self.logics.control.mapping.remove(ent_idx);
        self.logics
            .collision
            .handle_predicate(&CollisionReaction::RemoveBody(col_idx));

        self.draw.remove_drawable(col_idx);
        self.state.paddles.remove(ent_idx);
    }

    pub(crate) fn remove_wall(&mut self, wall: WallID) {
        let col_idx = self.state.get_col_idx(wall.into());
        let ent_idx = self
            .state
            .walls
            .iter()
            .position(|wid| *wid == wall)
            .unwrap();

        self.logics
            .collision
            .handle_predicate(&CollisionReaction::RemoveBody(col_idx));

        self.draw.remove_drawable(col_idx);
        self.state.walls.remove(ent_idx);
    }

    pub(crate) fn remove_ball(&mut self, ball: BallID) {
        let col_idx = self.state.get_col_idx(ball.into());
        let ent_idx = self
            .state
            .balls
            .iter()
            .position(|bid| *bid == ball)
            .unwrap();

        self.logics
            .physics
            .handle_predicate(&PhysicsReaction::RemoveBody(ent_idx));
        self.logics
            .collision
            .handle_predicate(&CollisionReaction::RemoveBody(col_idx));

        self.draw.remove_drawable(col_idx);
        self.state.balls.remove(ent_idx);
    }

    pub(crate) fn remove_score(&mut self, score: ScoreID) {
        let ent_i = self
            .state
            .scores
            .iter()
            .position(|sid| *sid == score)
            .unwrap();
        let rsrc = RsrcPool::Score(score);
        self.logics.resources.items.remove(&rsrc);

        self.state.scores.remove(ent_i);
    }
}
