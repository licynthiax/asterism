use macroquad::prelude::*;
use paddles_engine::{events::*, *};

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
    // ball
    let ball = Ball::new(
        Vec2::new(
            WIDTH as f32 / 2.0 - BALL_SIZE as f32 / 2.0,
            HEIGHT as f32 / 2.0 - BALL_SIZE as f32 / 2.0,
        ),
        Vec2::new(BALL_SIZE as f32, BALL_SIZE as f32),
    );
    let ball = game.add_ball(ball);

    // walls
    // left
    let left_wall = game.add_wall(Wall::new(
        Vec2::new(-1.0, 0.0),
        Vec2::new(1.0, HEIGHT as f32),
    ));
    // right
    let right_wall = game.add_wall(Wall::new(
        Vec2::new(WIDTH as f32, 0.0),
        Vec2::new(1.0, HEIGHT as f32),
    ));
    // top
    game.add_wall(Wall::new(
        Vec2::new(0.0, -1.0),
        Vec2::new(WIDTH as f32, 1.0),
    ));
    // bottom
    game.add_wall(Wall::new(
        Vec2::new(0.0, HEIGHT as f32),
        Vec2::new(WIDTH as f32, 1.0),
    ));

    // paddle 1
    let mut p1 = Paddle::new(
        Vec2::new(
            PADDLE_OFF_X as f32,
            HEIGHT as f32 / 2.0 - PADDLE_HEIGHT as f32 / 2.0,
        ),
        Vec2::new(PADDLE_WIDTH as f32, PADDLE_HEIGHT as f32),
    );
    let action_q = p1.add_control_map(KeyCode::Q, true);
    let action_a = p1.add_control_map(KeyCode::A, true);
    let action_w = p1.add_control_map(KeyCode::W, true);
    let paddle1 = game.add_paddle(p1);

    // paddle 2
    let mut p2 = Paddle::new(
        Vec2::new(
            WIDTH as f32 - PADDLE_OFF_X as f32 - PADDLE_WIDTH as f32,
            HEIGHT as f32 / 2.0 - PADDLE_HEIGHT as f32 / 2.0,
        ),
        Vec2::new(PADDLE_WIDTH as f32, PADDLE_HEIGHT as f32),
    );
    let action_o = p2.add_control_map(KeyCode::O, true);
    let action_l = p2.add_control_map(KeyCode::L, true);
    let action_i = p2.add_control_map(KeyCode::I, false);
    let paddle2 = game.add_paddle(p2);

    let score1 = game.add_score(Score::new(0));
    let score2 = game.add_score(Score::new(0));

    // paddle movement
    game.events.add_ctrl_event(
        EngineCtrlEvent::MovePaddle(paddle1, action_q),
        EngineAction::MovePaddleBy(paddle1, Vec2::new(0.0, 1.0)),
    );
    game.events.add_ctrl_event(
        EngineCtrlEvent::MovePaddle(paddle1, action_a),
        EngineAction::MovePaddleBy(paddle1, Vec2::new(0.0, -1.0)),
    );
    game.events.add_ctrl_event(
        EngineCtrlEvent::MovePaddle(paddle2, action_o),
        EngineAction::MovePaddleBy(paddle2, Vec2::new(0.0, 1.0)),
    );
    game.events.add_ctrl_event(
        EngineCtrlEvent::MovePaddle(paddle2, action_l),
        EngineAction::MovePaddleBy(paddle2, Vec2::new(0.0, -1.0)),
    );

    // serving
    game.events.add_ctrl_event(
        EngineCtrlEvent::ServePressed(paddle1, action_w),
        EngineAction::SetBallVel(ball, Vec2::splat(1.0)),
    );
    game.events.add_ctrl_event(
        EngineCtrlEvent::ServePressed(paddle1, action_w),
        EngineAction::SetKeyInvalid(paddle1, action_w),
    );

    game.events.add_ctrl_event(
        EngineCtrlEvent::ServePressed(paddle2, action_i),
        EngineAction::SetBallVel(ball, Vec2::splat(-1.0)),
    );
    game.events.add_ctrl_event(
        EngineCtrlEvent::ServePressed(paddle2, action_i),
        EngineAction::SetKeyInvalid(paddle2, action_i),
    );

    // increase score on collision with side wall
    game.events.add_col_event(
        EngineCollisionEvent::BallScoreWallCollide(ball, right_wall),
        EngineAction::ChangeScore(score1, 1),
    );
    game.events.add_col_event(
        EngineCollisionEvent::BallScoreWallCollide(ball, left_wall),
        EngineAction::ChangeScore(score2, 1),
    );

    game.events.add_col_event(
        EngineCollisionEvent::BallScoreWallCollide(ball, right_wall),
        EngineAction::SetKeyValid(paddle1, action_w),
    );
    game.events.add_col_event(
        EngineCollisionEvent::BallScoreWallCollide(ball, left_wall),
        EngineAction::SetKeyValid(paddle2, action_i),
    );

    /* let bounce_ball = |ColEvent { i, j, .. }, state: &mut State, logics: &mut Logics| {
        let id = state.get_id(i);
        if let EntID::Ball(ball_id) = id {
            let sides_touched = logics.collision.sides_touched(i, j);
            let mut vals = logics.physics.get_ident_data(ball_id.idx());
            if sides_touched.y != 0.0 {
                vals.vel.y *= -1.0;
            }
            if sides_touched.x != 0.0 {
                vals.vel.x *= -1.0;
            }
        }
    }; */

    /*    paddles_engine::rules!(game =>
        control: [
            {
                filter move_paddle,
                QueryType::CtrlEvent => CtrlEvent,
                |ctrl, _, _| {
                    ctrl.event_type == ControlEventType::KeyHeld
                },
                foreach |ctrl, _, logics| {
                    if ctrl.action_id == action_q || ctrl.action_id == action_o {
                        move_up(logics, ctrl.set);
                    } else if ctrl.action_id == action_a || ctrl.action_id == action_l {
                        move_down(logics, ctrl.set);
                    }
                }
            },
            {
                filter serve,
                QueryType::CtrlEvent => CtrlEvent,
                |ctrl, _, _| {
                    ctrl.event_type == ControlEventType::KeyPressed && (ctrl.action_id == action_w || ctrl.action_id == action_i)
                },
                foreach |ctrl, _, logics| {
                    serve_ball(logics, ctrl.set);
                }
            }
        ]

        physics: []

        collision: [
            {
                filter bounce,
                QueryType::ColEvent => ColEvent,
                |(i, j), _, logics| {
                    let i_id = logics.collision.metadata[*i].id;
                    let j_id = logics.collision.metadata[*j].id;
                    i_id == CollisionEnt::Ball &&
                        (j_id == CollisionEnt::Wall || j_id == CollisionEnt::Paddle)
                },
                foreach |col, state, logics| {
                    bounce_ball(col, state, logics);
                }
            },
            {
                filter score,
                QueryType::ColEvent => ColEvent,
                |(i, j), state, logics| {
                    let i_id = logics.collision.metadata[*i].id;
                    i_id == CollisionEnt::Ball &&
                        (*j == state.get_col_idx(left_wall.idx(), CollisionEnt::Wall) || *j == state.get_col_idx(right_wall.idx(), CollisionEnt::Wall))
                },
                foreach |(_, j), state, logics| {
                    if *j == state.get_col_idx(left_wall.idx(), CollisionEnt::Wall) {
                        inc_score(logics, 1);
                    } else if *j == state.get_col_idx(right_wall.idx(), CollisionEnt::Wall) {
                        inc_score(logics, 0);
                    } else {
                        unreachable!();
                    }
                }
            }
        ]

        resources: [
            {
                filter score_increased,
                QueryType::RsrcEvent => RsrcEvent,
                |pool, _, _| {
                    pool.event_type == ResourceEventType::PoolUpdated
                },
                foreach |event, _, logics| {
                    let RsrcPool::Score(score) = event.pool;

                    println!(
                        "p{} scored: {}",
                        score.idx() + 1,
                        logics.resources.get_ident_data(event.pool).0
                    );
                    logics
                        .physics
                        .handle_predicate(&PhysicsReaction::SetVel(0, Vec2::ZERO));

                    logics.physics.handle_predicate(&PhysicsReaction::SetPos(
                        0,
                        Vec2::splat(WIDTH as f32 / 2.0 - BALL_SIZE as f32 / 2.0),
                    ));

                    logics
                        .control
                        .handle_predicate(&ControlReaction::SetKeyValid(
                            score.idx(),
                            match_set!(score.idx(), action_w, action_i),
                        ));
                }
            }
        ]
    );*/
}
