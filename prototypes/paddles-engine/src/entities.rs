//! adding/removing entities
use asterism::collision::CollisionData;
use asterism::graphics::draw;
use asterism::Logic;
use asterism::{collision::CollisionReaction, physics::PhysicsReaction, resources::PoolValues};
use macroquad::math::Vec2;

use crate::types::*;
use crate::Game;

macro_rules! add_ent {
    (@attach $game:expr, $id:ident $gamefield:ident $ent_name:ident $ent_ty:ty; $id_ty:ty [collision: $col_data:expr]) => {
        let col_ent = $col_data.id;
        let col_idx = $game
            .state
            .get_col_idx($game.state.$gamefield.len(), col_ent);

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

    (@attach $game:expr, $id:ident $gamefield:ident $ent_name:ident $ent_ty:ty; $id_ty:ty [physics]) => {
        $game.logics.physics
            .add_physics_entity($ent_name.pos, $ent_name.vel, Vec2::ZERO);
    };

    // 'col_ent' is the collision entity because in this engine, everything that can be
    // collided with is also drawn
    (@attach $game:expr, $id:ident $gamefield:ident $ent_name:ident $ent_ty:ty; $id_ty:ty [draw: $col_ent:expr, $color:expr]) => {
        let col_idx = $game
            .state
            .get_col_idx($game.state.$gamefield.len(), $col_ent);

        let rect = draw::Drawable::Rectangle(
            draw::Rect::new(
                $ent_name.pos.x,
                $ent_name.pos.y,
                $ent_name.size.x,
                $ent_name.size.y,
            ),
            $color,
        );
        $game.draw.drawables.insert(col_idx, rect);
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
                [draw: CollisionEnt::Paddle, draw::WHITE]
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
                [draw: CollisionEnt::Ball, draw::YELLOW]
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
                [draw: CollisionEnt::Wall, draw::SKYBLUE]
            }, self, id);

        id
    }

    pub fn add_score(&mut self, score: Score) -> ScoreID {
        let id = ScoreID::new(self.state.score_id_max);
        self.state.score_id_max += 1;
        add_ent!(
            scores: (score: Score) -> ScoreID {
                [resource]
            }, self, id);
        id
    }

    pub(crate) fn remove_paddle(&mut self, paddle: PaddleID) {
        let ent_idx = self
            .state
            .paddles
            .iter()
            .position(|pid| *pid == paddle)
            .unwrap();
        let state_idx = self.state.get_col_idx(ent_idx, CollisionEnt::Paddle);

        self.logics.control.mapping.remove(ent_idx);
        self.logics
            .collision
            .handle_predicate(&CollisionReaction::RemoveBody(state_idx));

        self.draw.drawables.remove(state_idx);
        self.state.paddles.remove(ent_idx);
    }

    pub(crate) fn remove_wall(&mut self, wall: WallID) {
        let ent_idx = self
            .state
            .walls
            .iter()
            .position(|wid| *wid == wall)
            .unwrap();
        let state_idx = self.state.get_col_idx(ent_idx, CollisionEnt::Wall);

        self.logics
            .collision
            .handle_predicate(&CollisionReaction::RemoveBody(state_idx));

        self.draw.drawables.remove(state_idx);
        self.state.walls.remove(ent_idx);
    }

    pub(crate) fn remove_ball(&mut self, ball: BallID) {
        let ent_idx = self
            .state
            .balls
            .iter()
            .position(|bid| *bid == ball)
            .unwrap();
        let state_idx = self.state.get_col_idx(ent_idx, CollisionEnt::Ball);

        self.logics
            .physics
            .handle_predicate(&PhysicsReaction::RemoveBody(ent_idx));
        self.logics
            .collision
            .handle_predicate(&CollisionReaction::RemoveBody(state_idx));

        self.draw.drawables.remove(state_idx);
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
