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
    // ball
    let center = Vec2::new(
        WIDTH as f32 / 2.0 - BALL_SIZE as f32 / 2.0,
        HEIGHT as f32 / 2.0 - BALL_SIZE as f32 / 2.0,
    );
    let ball = Ball::new(center, Vec2::new(BALL_SIZE as f32, BALL_SIZE as f32));
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

    let score1 = game.add_score(Score::new(0, Vec2::ZERO));
    let score2 = game.add_score(Score::new(0, Vec2::new((WIDTH - PADDLE_OFF_X) as f32, 0.0)));

    // paddle movement
    game.events.add_ctrl_event(
        EngineCtrlEvent::MovePaddle(paddle1, action_q),
        EngineAction::MovePaddleBy(paddle1, Vec2::new(0.0, -1.0)),
    );
    game.events.add_ctrl_event(
        EngineCtrlEvent::MovePaddle(paddle1, action_a),
        EngineAction::MovePaddleBy(paddle1, Vec2::new(0.0, 1.0)),
    );
    game.events.add_ctrl_event(
        EngineCtrlEvent::MovePaddle(paddle2, action_o),
        EngineAction::MovePaddleBy(paddle2, Vec2::new(0.0, -1.0)),
    );
    game.events.add_ctrl_event(
        EngineCtrlEvent::MovePaddle(paddle2, action_l),
        EngineAction::MovePaddleBy(paddle2, Vec2::new(0.0, 1.0)),
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
    game.events.add_col_events(
        EngineCollisionEvent::Match(
            EntityMatch::ByID(ball.into()),
            EntityMatch::ByID(right_wall.into()),
        ),
        vec![
            EngineAction::ChangeScoreBy(score1, 1),
            EngineAction::SetKeyValid(paddle1, action_w),
            EngineAction::SetBallPos(ball, center),
            EngineAction::SetBallVel(ball, Vec2::ZERO),
        ],
    );

    game.events.add_col_events(
        EngineCollisionEvent::Match(
            EntityMatch::ByID(ball.into()),
            EntityMatch::ByID(left_wall.into()),
        ),
        vec![
            EngineAction::ChangeScoreBy(score2, 1),
            EngineAction::SetKeyValid(paddle2, action_i),
            EngineAction::SetBallPos(ball, center),
            EngineAction::SetBallVel(ball, Vec2::ZERO),
        ],
    );

    game.events.add_col_events(
        EngineCollisionEvent::Match(
            EntityMatch::ByID(ball.into()),
            EntityMatch::ByID(paddle1.into()),
        ),
        vec![EngineAction::BounceBall(ball, Some(EntID::Paddle(paddle1)))],
    );
    game.events.add_col_events(
        EngineCollisionEvent::Match(
            EntityMatch::ByID(ball.into()),
            EntityMatch::ByID(paddle2.into()),
        ),
        vec![EngineAction::BounceBall(ball, Some(EntID::Paddle(paddle2)))],
    );

    game.events.add_col_events(
        EngineCollisionEvent::Match(
            EntityMatch::ByID(ball.into()),
            EntityMatch::ByType(CollisionEnt::Wall.into()),
        ),
        vec![EngineAction::BounceBall(ball, None)],
    );
}
