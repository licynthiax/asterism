use macroquad::prelude::*;
use paddles_engine::*;

const WIDTH: u8 = 255;
const HEIGHT: u8 = 255;
const BALL_SIZE: u8 = 10;
const PADDLE_OFF_X: u8 = 16;
const PADDLE_HEIGHT: u8 = 48;
const PADDLE_WIDTH: u8 = 8;

fn window_conf() -> Conf {
    Conf {
        window_title: "paddles".to_owned(),
        window_width: WIDTH as i32,
        window_height: HEIGHT as i32,
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // initialize game
    let mut game = Game::new();
    init(&mut game);
    run(game).await;
}

fn init(game: &mut Game) {
    let mut ball = Ball::new();
    ball.set_pos(Vec2::new(
        WIDTH as f32 / 2.0 - BALL_SIZE as f32 / 2.0,
        HEIGHT as f32 / 2.0 - BALL_SIZE as f32 / 2.0,
    ));
    ball.set_size(Vec2::new(BALL_SIZE as f32, BALL_SIZE as f32));

    game.add_ball(ball);

    // left
    let mut wall = Wall::new();
    wall.set_pos(Vec2::new(-1.0, 0.0));
    wall.set_size(Vec2::new(1.0, HEIGHT as f32));
    game.add_wall(wall);
    // right
    let mut wall = Wall::new();
    wall.set_pos(Vec2::new(WIDTH as f32, 0.0));
    wall.set_size(Vec2::new(1.0, HEIGHT as f32));
    game.add_wall(wall);
    // top
    let mut wall = Wall::new();
    wall.set_pos(Vec2::new(0.0, -1.0));
    wall.set_size(Vec2::new(WIDTH as f32, 1.0));
    game.add_wall(wall);
    // bottom
    let mut wall = Wall::new();
    wall.set_pos(Vec2::new(0.0, HEIGHT as f32));
    wall.set_size(Vec2::new(WIDTH as f32, 1.0));
    game.add_wall(wall);

    let mut paddle1 = Paddle::new();
    let action_q = paddle1.add_control_map(KeyCode::Q);
    let action_a = paddle1.add_control_map(KeyCode::A);
    let action_w = paddle1.add_control_map(KeyCode::W);
    paddle1.set_pos(Vec2::new(
        PADDLE_OFF_X as f32,
        HEIGHT as f32 / 2.0 - PADDLE_HEIGHT as f32 / 2.0,
    ));
    paddle1.set_size(Vec2::new(PADDLE_WIDTH as f32, PADDLE_HEIGHT as f32));

    game.add_paddle(paddle1);

    let mut paddle2 = Paddle::new();
    let action_o = paddle2.add_control_map(KeyCode::O);
    let action_l = paddle2.add_control_map(KeyCode::L);
    let action_i = paddle2.add_control_map(KeyCode::I);
    paddle2.set_pos(Vec2::new(
        WIDTH as f32 - PADDLE_OFF_X as f32 - PADDLE_WIDTH as f32,
        HEIGHT as f32 / 2.0 - PADDLE_HEIGHT as f32 / 2.0,
    ));
    paddle2.set_size(Vec2::new(PADDLE_WIDTH as f32, PADDLE_HEIGHT as f32));

    game.add_paddle(paddle2);

    let score1 = Score::new();
    game.add_score(score1);
    let score2 = Score::new();
    game.add_score(score2);

    let inc_score =
        |state: &mut State, logics: &mut Logics, event: &CollisionEvent<CollisionEnt>| {
            if let CollisionEnt::Wall(wall_id) = event.1 {
                logics.resources.handle_predicate(&(
                    RsrcPool::Score(state.scores[wall_id.idx()]),
                    Transaction::Change(1),
                ));
            }
        };

    game.add_collision_predicate(
        CollisionEnt::Ball(game.state.balls[0]),
        CollisionEnt::Wall(game.state.walls[0]),
        Box::new(inc_score),
    );

    game.add_collision_predicate(
        CollisionEnt::Ball(game.state.balls[0]),
        CollisionEnt::Wall(game.state.walls[1]),
        Box::new(inc_score),
    );

    let reset_ball = |state: &mut State, logics: &mut Logics, _: &ResourceEvent<RsrcPool>| {
        logics
            .physics
            .handle_predicate(&PhysicsReaction::SetVel(state.balls[0].idx(), Vec2::ZERO));
        logics.physics.handle_predicate(&PhysicsReaction::SetPos(
            state.balls[0].idx(),
            Vec2::new(
                WIDTH as f32 / 2.0 - BALL_SIZE as f32 / 2.0,
                WIDTH as f32 / 2.0 - BALL_SIZE as f32 / 2.0,
            ),
        ));
    };

    game.add_rsrc_predicate(
        RsrcPool::Score(game.state.scores[0]),
        ResourceEventType::PoolUpdated,
        Box::new(reset_ball),
    );
    game.add_rsrc_predicate(
        RsrcPool::Score(game.state.scores[1]),
        ResourceEventType::PoolUpdated,
        Box::new(reset_ball),
    );

    let bounce_ball_y =
        |_: &mut State, logics: &mut Logics, event: &CollisionEvent<CollisionEnt>| {
            if let CollisionEnt::Ball(ball_id) = event.0 {
                let vel = logics.physics.velocities[ball_id.idx()];
                logics.physics.handle_predicate(&PhysicsReaction::SetVel(
                    ball_id.idx(),
                    Vec2::new(vel.x, vel.y * -1.0),
                ));
            }
        };

    let bounce_ball_x =
        |_: &mut State, logics: &mut Logics, event: &CollisionEvent<CollisionEnt>| {
            if let CollisionEnt::Ball(ball_id) = event.0 {
                let vel = logics.physics.velocities[ball_id.idx()];
                logics.physics.handle_predicate(&PhysicsReaction::SetVel(
                    ball_id.idx(),
                    Vec2::new(vel.x * -1.0, vel.y),
                ));
            }
        };

    game.add_collision_predicate(
        CollisionEnt::Ball(game.state.balls[0]),
        CollisionEnt::Wall(game.state.walls[2]),
        Box::new(bounce_ball_y),
    );

    game.add_collision_predicate(
        CollisionEnt::Ball(game.state.balls[0]),
        CollisionEnt::Wall(game.state.walls[3]),
        Box::new(bounce_ball_y),
    );

    game.add_collision_predicate(
        CollisionEnt::Ball(game.state.balls[0]),
        CollisionEnt::Paddle(game.state.paddles[0]),
        Box::new(bounce_ball_x),
    );

    game.add_collision_predicate(
        CollisionEnt::Ball(game.state.balls[0]),
        CollisionEnt::Paddle(game.state.paddles[1]),
        Box::new(bounce_ball_x),
    );

    // these controls should be a mapping?
    game.add_ctrl_predicate(
        game.state.paddles[0],
        action_q,
        ControlEventType::KeyHeld,
        Box::new(|state, logics, event| {
            let col_idx = state.get_col_idx(CollisionEnt::Paddle(state.paddles[event.set]));
            logics.collision.centers[col_idx].y -= 1.0;
            logics.collision.velocities[col_idx].y = -1.0;
        }),
    );

    game.add_ctrl_predicate(
        game.state.paddles[0],
        action_a,
        ControlEventType::KeyHeld,
        Box::new(|state, logics, event| {
            let col_idx = state.get_col_idx(CollisionEnt::Paddle(state.paddles[event.set]));
            logics.collision.centers[col_idx].y += 1.0;
            logics.collision.velocities[col_idx].y = 1.0;
        }),
    );

    game.add_ctrl_predicate(
        game.state.paddles[1],
        action_o,
        ControlEventType::KeyHeld,
        Box::new(|state, logics, event| {
            let col_idx = state.get_col_idx(CollisionEnt::Paddle(state.paddles[event.set]));
            logics.collision.centers[col_idx].y -= 1.0;
            logics.collision.velocities[col_idx].y = -1.0;
        }),
    );

    game.add_ctrl_predicate(
        game.state.paddles[1],
        action_l,
        ControlEventType::KeyHeld,
        Box::new(|state, logics, event| {
            let col_idx = state.get_col_idx(CollisionEnt::Paddle(state.paddles[event.set]));
            logics.collision.centers[col_idx].y += 1.0;
            logics.collision.velocities[col_idx].y = 1.0;
        }),
    );

    // but these shouldn't
    let move_ball = |state: &mut State, logics: &mut Logics, event: &ControlEvent<ActionID>| {
        let vel = match event.set {
            0 => Vec2::new(1.0, 1.0),
            1 => Vec2::new(-1.0, -1.0),
            _ => unreachable!(),
        };
        logics
            .physics
            .handle_predicate(&PhysicsReaction::SetVel(state.balls[0].idx(), vel));
        logics
            .control
            .handle_predicate(&ControlReaction::SetKeyInvalid(event.set, event.action_id));
    };
    game.add_ctrl_predicate(
        game.state.paddles[0],
        action_w,
        ControlEventType::KeyPressed,
        Box::new(move_ball),
    );

    game.add_ctrl_predicate(
        game.state.paddles[1],
        action_i,
        ControlEventType::KeyPressed,
        Box::new(move_ball),
    );
}
