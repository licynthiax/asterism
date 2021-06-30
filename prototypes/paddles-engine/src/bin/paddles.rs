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
    let mut ball = Ball::new();
    ball.set_pos(Vec2::new(
        WIDTH as f32 / 2.0 - BALL_SIZE as f32 / 2.0,
        HEIGHT as f32 / 2.0 - BALL_SIZE as f32 / 2.0,
    ));
    ball.set_size(Vec2::new(BALL_SIZE as f32, BALL_SIZE as f32));
    ball.set_vel(Vec2::new(1.0, 0.0));
    let ball = game.add_ball(ball);

    // walls
    // left
    let mut wall = Wall::new();
    wall.set_pos(Vec2::new(-1.0, 0.0));
    wall.set_size(Vec2::new(1.0, HEIGHT as f32));
    let left_wall = game.add_wall(wall);
    // right
    let mut wall = Wall::new();
    wall.set_pos(Vec2::new(WIDTH as f32, 0.0));
    wall.set_size(Vec2::new(1.0, HEIGHT as f32));
    let right_wall = game.add_wall(wall);
    // top
    let mut wall = Wall::new();
    wall.set_pos(Vec2::new(0.0, -1.0));
    wall.set_size(Vec2::new(WIDTH as f32, 1.0));
    let top_wall = game.add_wall(wall);
    // bottom
    let mut wall = Wall::new();
    wall.set_pos(Vec2::new(0.0, HEIGHT as f32));
    wall.set_size(Vec2::new(WIDTH as f32, 1.0));
    let bottom_wall = game.add_wall(wall);

    // paddle 1
    let mut paddle1 = Paddle::new();
    let action_q = paddle1.add_control_map(KeyCode::Q, true);
    let action_a = paddle1.add_control_map(KeyCode::A, true);
    let action_w = paddle1.add_control_map(KeyCode::W, true);
    paddle1.set_pos(Vec2::new(
        PADDLE_OFF_X as f32,
        HEIGHT as f32 / 2.0 - PADDLE_HEIGHT as f32 / 2.0,
    ));
    paddle1.set_size(Vec2::new(PADDLE_WIDTH as f32, PADDLE_HEIGHT as f32));
    game.add_paddle(paddle1);

    // paddle 2
    let mut paddle2 = Paddle::new();
    let action_o = paddle2.add_control_map(KeyCode::O, true);
    let action_l = paddle2.add_control_map(KeyCode::L, true);
    let action_i = paddle2.add_control_map(KeyCode::I, false);
    paddle2.set_pos(Vec2::new(
        WIDTH as f32 - PADDLE_OFF_X as f32 - PADDLE_WIDTH as f32,
        HEIGHT as f32 / 2.0 - PADDLE_HEIGHT as f32 / 2.0,
    ));
    paddle2.set_size(Vec2::new(PADDLE_WIDTH as f32, PADDLE_HEIGHT as f32));

    game.add_paddle(paddle2);

    game.add_score(Score::new());
    game.add_score(Score::new());

    let action_q = game.add_ctrl_query(CtrlEvent {
        set: 0,
        action_id: action_q,
        event_type: ControlEventType::KeyHeld,
    });
    let action_l = game.add_ctrl_query(CtrlEvent {
        set: 0,
        action_id: action_l,
        event_type: ControlEventType::KeyHeld,
    });

    let action_o = game.add_ctrl_query(CtrlEvent {
        set: 1,
        action_id: action_o,
        event_type: ControlEventType::KeyHeld,
    });
    let action_a = game.add_ctrl_query(CtrlEvent {
        set: 1,
        action_id: action_a,
        event_type: ControlEventType::KeyHeld,
    });

    let paddle_up = Compose::Or(
        Box::new(Compose::Just(action_q, ProcessOutput::IfAny)),
        Box::new(Compose::Just(action_o, ProcessOutput::IfAny)),
    );

    let paddle_up_condition =
        game.add_compose(paddle_up, Box::new(move |state, logics, compose| {}));

    // paddle movement
    // Box::new(|_: &mut State, logics: &mut Logics, event: &CtrlEvent| {
    //     let mut paddle_col = logics.collision.get_synthesis(event.set);
    //     paddle_col.center.y -= 1.0;
    //     paddle_col.vel.y = (paddle_col.vel.y.abs() - 1.0).max(-1.0);
    //     logics.collision.update_synthesis(event.set, paddle_col);
    // }),
    //
    // Box::new(|_: &mut State, logics: &mut Logics, event: &CtrlEvent| {
    //     let mut paddle_col = logics.collision.get_synthesis(event.set);
    //     paddle_col.center.y += 1.0;
    //     paddle_col.vel.y = (paddle_col.vel.y.abs() + 1.0).min(1.0);
    //     logics.collision.update_synthesis(event.set, paddle_col);
    // }),

    let _inc_score = move |state: &mut State, logics: &mut Logics, event: &AColEvent| {
        let change_score = if event.1 == state.get_col_idx(left_wall.idx(), CollisionEnt::Wall) {
            1
        } else if event.1 == state.get_col_idx(right_wall.idx(), CollisionEnt::Wall) {
            0
        } else {
            unreachable!("{}", event.1)
        };
        logics.resources.handle_predicate(&(
            RsrcPool::Score(ScoreID::new(change_score)),
            Transaction::Change(1),
        ));
    };

    game.add_collision_query(ColEvent::ByIdx(
        game.state.get_col_idx(ball.idx(), CollisionEnt::Ball),
        game.state.get_col_idx(left_wall.idx(), CollisionEnt::Wall),
    ));

    game.add_collision_query(ColEvent::ByIdx(
        game.state.get_col_idx(ball.idx(), CollisionEnt::Ball),
        game.state.get_col_idx(right_wall.idx(), CollisionEnt::Wall),
    ));

    let _reset_ball = |state: &mut State, logics: &mut Logics, event: &ARsrcEvent| {
        let RsrcPool::Score(score_id) = event.pool;
        logics
            .physics
            .handle_predicate(&PhysicsReaction::SetVel(0, Vec2::ZERO));
        logics
            .collision
            .handle_predicate(&CollisionReaction::SetPos(
                state.get_col_idx(0, CollisionEnt::Ball),
                Vec2::new(
                    WIDTH as f32 / 2.0 - BALL_SIZE as f32 / 2.0,
                    WIDTH as f32 / 2.0 - BALL_SIZE as f32 / 2.0,
                ),
            ));
        logics
            .control
            .handle_predicate(&ControlReaction::SetKeyValid(
                match score_id.idx() {
                    0 => 1,
                    1 => 0,
                    _ => unreachable!(),
                },
                ActionID::new(2), // eh....
            ));
    };

    game.add_rsrc_query(RsrcEvent { success: true });

    let _bounce_ball = |state: &mut State, logics: &mut Logics, (i, j): &AColEvent| {
        let id = state.get_id(*i);
        if let EntID::Ball(ball_id) = id {
            let sides_touched = logics.collision.sides_touched(*i, *j);
            let mut vals = logics.physics.get_synthesis(ball_id.idx());
            if sides_touched.y != 0.0 {
                vals.vel.y *= -1.0;
            }
            if sides_touched.x != 0.0 {
                vals.vel.x *= -1.0;
            }
            logics.physics.update_synthesis(ball_id.idx(), vals);
        }
    };

    game.add_collision_query(ColEvent::ByIdx(
        game.state.get_col_idx(ball.idx(), CollisionEnt::Ball),
        game.state.get_col_idx(top_wall.idx(), CollisionEnt::Wall),
    ));

    game.add_collision_query(ColEvent::ByIdx(
        game.state.get_col_idx(ball.idx(), CollisionEnt::Ball),
        game.state
            .get_col_idx(bottom_wall.idx(), CollisionEnt::Wall),
    ));

    game.add_collision_query(ColEvent::ByType(CollisionEnt::Ball, CollisionEnt::Paddle));

    let _serve_ball = |_: &mut State, logics: &mut Logics, event: &CtrlEvent| {
        let vel = match event.set {
            0 => Vec2::new(1.0, 1.0),
            1 => Vec2::new(-1.0, -1.0),
            _ => unreachable!(),
        };
        logics
            .physics
            .handle_predicate(&PhysicsReaction::SetVel(0, vel));
        logics
            .control
            .handle_predicate(&ControlReaction::SetKeyInvalid(event.set, event.action_id));
    };

    game.add_ctrl_query(CtrlEvent {
        set: 0,
        action_id: action_w,
        event_type: ControlEventType::KeyPressed,
    });

    game.add_ctrl_query(CtrlEvent {
        set: 1,
        action_id: action_i,
        event_type: ControlEventType::KeyPressed,
    });
}
