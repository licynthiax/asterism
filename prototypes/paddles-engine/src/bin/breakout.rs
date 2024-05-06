use macroquad::prelude::*;
use paddles_engine::*;

const WIDTH: u8 = 255;
const HEIGHT: u8 = 255;
const BALL_SIZE: u8 = 10;
const PADDLE_OFF_X: u8 = 16;
const PADDLE_WIDTH: u8 = 48;
const PADDLE_HEIGHT: u8 = 8;

fn window_conf() -> Conf {
    Conf {
        window_title: "breakout".to_owned(),
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
    let ball = game.add_ball(Ball::new(
        Vec2::new(
            WIDTH as f32 / 2.0 - BALL_SIZE as f32 / 2.0,
            HEIGHT as f32 - PADDLE_OFF_X as f32 * 2.0,
        ),
        Vec2::new(BALL_SIZE as f32, BALL_SIZE as f32),
    ));

    // walls
    // left
    game.add_wall(Wall::new(
        Vec2::new(-1.0, 0.0),
        Vec2::new(1.0, HEIGHT as f32),
    ));
    // right
    game.add_wall(Wall::new(
        Vec2::new(WIDTH as f32, 0.0),
        Vec2::new(1.0, HEIGHT as f32),
    ));
    // top
    game.add_wall(Wall::new(
        Vec2::new(0.0, -1.0),
        Vec2::new(WIDTH as f32, 1.0),
    ));
    // bottom
    let bottom_wall = game.add_wall(Wall::new(
        Vec2::new(0.0, HEIGHT as f32),
        Vec2::new(WIDTH as f32, 1.0),
    ));

    // blocks
    for block in add_blocks() {
        game.add_wall(block);
    }

    // paddle 1
    let mut paddle = Paddle::new(
        Vec2::new(
            WIDTH as f32 / 2.0 - PADDLE_WIDTH as f32 / 2.0,
            HEIGHT as f32 - PADDLE_OFF_X as f32,
        ),
        Vec2::new(PADDLE_WIDTH as f32, PADDLE_HEIGHT as f32),
    );
    let left = paddle.add_control_map(KeyCode::Left, true);
    let right = paddle.add_control_map(KeyCode::Right, true);
    let serve = paddle.add_control_map(KeyCode::Space, true);
    let paddle = game.add_paddle(paddle);

    let score = game.add_score(Score::new(0, Vec2::new(0.0, (HEIGHT - 22) as f32)));

    // define paddle controls
    game.events.add_ctrl_events(
        EngineCtrlEvent::ServePressed(paddle, serve),
        vec![
            EngineAction::SetBallVel(ball, Vec2::splat(1.0)),
            EngineAction::SetKeyInvalid(paddle, serve),
        ],
    );

    game.events.add_ctrl_event(
        EngineCtrlEvent::MovePaddle(paddle, left),
        EngineAction::MovePaddleBy(paddle, Vec2::new(-1.0, 0.0)),
    );
    game.events.add_ctrl_event(
        EngineCtrlEvent::MovePaddle(paddle, right),
        EngineAction::MovePaddleBy(paddle, Vec2::new(1.0, 0.0)),
    );

    // bounce ball off everything
    game.events.add_col_events(
        EngineCollisionEvent::Match(EntityMatch::ByID(ball.into()), EntityMatch::All),
        vec![EngineAction::BounceBall(ball, None)],
    );

    // remove box when bounced into
    game.events.add_col_events(
        EngineCollisionEvent::Filter(Box::new(|id1: EntID, id2: EntID| {
            id1.get_col_type() == CollisionEnt::Ball
                && id2.get_col_type() == CollisionEnt::Wall
                && id2.get_wall().unwrap().idx() >= 4
        })),
        vec![
            EngineAction::RemoveEntity(None),
            EngineAction::ChangeScoreBy(score, 1),
        ],
    );

    // reset score when ball hits bottom wall
    game.events.add_col_events(
        EngineCollisionEvent::Match(
            EntityMatch::ByID(ball.into()),
            EntityMatch::ByID(bottom_wall.into()),
        ),
        vec![EngineAction::ChangeScore(score, 0)],
    );

    // reset score when score == 40
    game.events.add_rsrc_event(
        EngineRsrcEvent::ScoreEquals(score, 40),
        EngineAction::ChangeScore(score, 0),
    );

    // score reset
    // clear board
    game.events.add_rsrc_event(
        EngineRsrcEvent::ScoreReset(score),
        EngineAction::RemoveEntity(Some(EntityMatch::Filter(Box::new(|ent: EntID| {
            ent.get_type() == EntType::Wall && ent.get_wall().unwrap().idx() >= 4
        })))),
    );

    // add a million blocks
    game.events.add_rsrc_events(
        EngineRsrcEvent::ScoreReset(score),
        add_blocks()
            .iter()
            .map(|wall| EngineAction::AddEntity(Ent::Wall(*wall)))
            .collect(),
    );

    // reset ball
    game.events.add_rsrc_events(
        EngineRsrcEvent::ScoreReset(score),
        vec![
            EngineAction::SetBallPos(
                ball,
                Vec2::new(
                    WIDTH as f32 / 2.0 - BALL_SIZE as f32 / 2.0,
                    HEIGHT as f32 - PADDLE_OFF_X as f32 * 2.0,
                ),
            ),
            EngineAction::SetBallVel(ball, Vec2::ZERO),
            EngineAction::SetKeyValid(paddle, serve),
        ],
    );
}

fn add_blocks() -> Vec<Wall> {
    let block_size = Vec2::new(32.0, 16.0);
    (0..5)
        .flat_map(|y| {
            (0..8).map(move |x| Wall::new(Vec2::new(x as f32 * 32.0, y as f32 * 16.0), block_size))
        })
        .collect()
}
