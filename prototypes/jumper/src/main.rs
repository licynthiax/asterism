#![deny(clippy::all)]
#![forbid(unsafe_code)]
#![allow(dead_code)]

use pixels::{wgpu::Surface, Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use ultraviolet::{Vec2, Vec3, geometry::Aabb};

const WIDTH: u8 = 255;
const HEIGHT: u8 = 255;
const PLAYER_SIZE: u8 = 10;

trait Input {
    fn min(&self) -> f32;
    fn max(&self) -> f32;
}

#[derive(Clone)]
enum KeyInput {
    Single(VirtualKeyCode),
    Pair(VirtualKeyCode, VirtualKeyCode)
}

impl Input for KeyInput {
    fn min(&self) -> f32 { 0.0 }

    fn max(&self) -> f32 { 1.0 }
}

#[derive(Clone)]
enum InputState {
    On, Off, Pressed, Released
}

enum ActionType {
    Instant(f32),
    Continuous(f32),
    Axis(f32, f32)
}

impl Default for ActionType {
    fn default() -> Self { Self::Instant(0.0) }
}

#[derive(Default)]
struct Action<ID: Copy + Eq> {
    id: ID,
    action_type: ActionType
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ActionID {
    Move(Player),
    Jump(Player),
}

impl Default for ActionID {
    fn default() -> Self { Self::Move(Player::P1) }
}

struct InputMap<I: Input, ID: Copy + Eq> {
    inputs: Vec<(I, InputState)>,
    actions: Vec<Action<ID>>
        // Invariants: inputs.len() == actions.actions.len()
}

struct WinitKeyboardControl<ID: Copy + Eq> {
    mapping: Vec<InputMap<KeyInput, ID>>,
    values: Vec<Vec<f32>> // vector of values per mapping.
        // Invariants: mapping.len() == values.len(), mapping[i].inputs.len() == values[i].len() 
}

impl<ID: Copy + Eq> WinitKeyboardControl<ID> {
    fn new() -> Self {
        Self {
            mapping: Vec::new(),
            values: Vec::new()
        }
    }

    fn update(&mut self, events: &WinitInputHelper) {
        self.values.resize_with(self.mapping.len(), Default::default);
        for (map, vals) in self.mapping.iter().zip(self.values.iter_mut()) {
            vals.resize_with(map.inputs.len(), Default::default);
            for (action_map, value) in map.inputs.iter().zip(map.actions.iter()).zip(vals.iter_mut()) {
                let ((input, input_state), action) = action_map;
                let is_activated = |keycode: VirtualKeyCode| {
                    match input_state {
                        InputState::On => events.key_held(keycode),
                        InputState::Off => !events.key_held(keycode),
                        InputState::Pressed => events.key_pressed(keycode),
                        InputState::Released => events.key_released(keycode)
                    }
                };
                match (&action.action_type, input) {
                    (ActionType::Instant(val), KeyInput::Single(keycode)) |
                        (ActionType::Continuous(val), KeyInput::Single(keycode)) => {
                        if is_activated(*keycode) {
                            *value = input.max() * val;
                        } else {
                            *value = input.min();
                        }
                    }
                    (ActionType::Axis(axis_min, axis_max), KeyInput::Pair(keycode_min, keycode_max)) => {
                        *value = input.min();
                        if is_activated(*keycode_min) {
                            *value += input.max() * axis_min;
                        }
                        if is_activated(*keycode_max) {
                            *value += input.max() * axis_max;
                        }
                    }
                    (ActionType::Axis(axis_min, axis_max), KeyInput::Single(keycode)) => {
                        if is_activated(*keycode) {
                            *value = input.min() * axis_min;
                        } else {
                            *value = input.max() * axis_max;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn get_action_by_index(&self, action_set: usize, idx: usize) -> f32 {
        self.values[action_set][idx]
    }

    // This gets the value of the first action whose `id` is `id`.
    pub fn get_action(&self, id: ID) -> Option<f32> {
        for (i, set) in self.mapping.iter().enumerate() {
            if let Some(j) = set.actions.iter().position(|act| act.id == id) {
                return Some(self.values[i][j]);
            }
        }
        None
    }

    pub fn get_action_in_set(&self, action_set: usize, id: ID) -> Option<f32> {
        if let Some(idx) = self.mapping[action_set].actions.iter().position(|act| act.id == id) {
            return Some(self.get_action_by_index(action_set, idx));
        }
        None
    }
}

struct JumperPhysics {
    positions: Vec<Vec2>,
    velocities: Vec<Vec2>,
    accelerations: Vec<Vec2>,
}

impl JumperPhysics {
    fn new() -> Self {
        Self {
            positions: Vec::new(),
            velocities: Vec::new(),
            accelerations: Vec::new(),
        }
    }

    fn update(&mut self) {
        for (pos, (vel, acc)) in self.positions.iter_mut().zip(self.velocities.iter_mut().zip(self.accelerations.iter())) {
            *vel += *acc;
            *pos += *vel;
        }
    }
}

struct AabbCollision<ID: Copy + Eq> {
    bodies: Vec<Aabb>,
    velocities: Vec<Vec2>,
    metadata: Vec<CollisionData<ID>>,
    contacts: Vec<(usize, usize)>,
}

#[derive(Default, Clone, Copy)]
struct CollisionData<ID: Copy + Eq> {
    solid: bool, // true = participates in restitution, false = no
    fixed: bool, // collision system cannot move it
    id: ID
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CollisionID {
    Player(Player),
    TopWall,
    BottomWall,
    LeftWall,
    RightWall,
    Ground,
    Wall,
}

impl Default for CollisionID {
    fn default() -> Self { Self::LeftWall }
}

impl<ID: Copy + Eq> AabbCollision<ID> {
    pub fn new() -> Self {
        Self {
            bodies: Vec::new(),
            metadata: Vec::new(),
            velocities: Vec::new(),
            contacts: Vec::new(),
        }
    }

    pub fn update(&mut self) {
        self.contacts.clear();
        for (i, body) in self.bodies.iter().enumerate() {
            for (j, body2) in self.bodies[i + 1..].iter().enumerate() {
                if body.intersects(body2) {
                    self.contacts.push((i, j + i + 1));
                }
            }
        }

        for (i, j) in self.contacts.iter() {
            let CollisionData { solid: i_solid, fixed: i_fixed, .. } =
                self.metadata[*i];
            let CollisionData { solid: j_solid, fixed: j_fixed, .. } =
                self.metadata[*j];

            if !(i_solid && j_solid) || (i_fixed && j_fixed) {
                continue;
            }

            if !i_fixed && !j_fixed {
                let Vec2 { x: vel_i_x, y: vel_i_y } = self.velocities[*i];
                let Vec2 { x: vel_j_x, y: vel_j_y } = self.velocities[*j];
                let Aabb { min: Vec3 { x: min_i_x, y: min_i_y, .. },
                    max: Vec3 { x: max_i_x, y: max_i_y, ..} } = self.bodies[*i];
                let Aabb { min: Vec3 { x: min_j_x, y: min_j_y, .. },
                    max: Vec3 { x: max_j_x, y: max_j_y, ..} } = self.bodies[*j];

                let ( i_displace, j_displace ) = {
                    let vel_i_x = vel_i_x / (vel_i_x.abs() + vel_j_x.abs());
                    let vel_i_y = vel_i_y / (vel_i_y.abs() + vel_j_y.abs());
                    let vel_j_x = vel_j_x / (vel_i_x.abs() + vel_j_x.abs());
                    let vel_j_y = vel_j_y / (vel_i_y.abs() + vel_j_y.abs());

                    let displacement_x = Self::get_displacement(min_i_x, max_i_x, min_j_x, max_j_x);
                    let displacement_y = Self::get_displacement(min_i_y, max_i_y, min_j_y, max_j_y);

                    ( Vec3::new(displacement_x * vel_i_x, displacement_y * vel_i_y, 0.0),
                        Vec3::new(displacement_x * vel_j_x, displacement_y * vel_j_y, 0.0) )
                };

                self.bodies[*i].min += i_displace;
                self.bodies[*i].max += i_displace;
                self.bodies[*j].min += j_displace;
                self.bodies[*j].max += j_displace;
            } else {
                let i_swap = if !j_fixed { j } else { i };
                let j_swap = if !j_fixed { i } else { j };
                let Aabb { min: Vec3 { x: min_i_x, y: min_i_y, .. },
                    max: Vec3 { x: max_i_x, y: max_i_y, ..} } = self.bodies[*i_swap];
                let Aabb { min: Vec3 { x: min_j_x, y: min_j_y, .. },
                    max: Vec3 { x: max_j_x, y: max_j_y, ..} } = self.bodies[*j_swap];
                let displace = {
                    let displacement_x = Self::get_displacement(min_i_x, max_i_x, min_j_x, max_j_x);
                    let displacement_y = Self::get_displacement(min_i_y, max_i_y, min_j_y, max_j_y);

                    if displacement_x == displacement_y {
                        Vec3::new(displacement_x, displacement_y, 0.0)
                    } else if displacement_x < displacement_y {
                        if min_i_x < min_j_x {
                            Vec3::new(-displacement_x, 0.0, 0.0)
                        } else {
                            Vec3::new(displacement_x, 0.0, 0.0)
                        }
                    } else {
                        if min_i_y < min_j_y {
                            Vec3::new(0.0, -displacement_y, 0.0)
                        } else {
                            Vec3::new(0.0, displacement_y, 0.0)
                        }
                    }
                };

                self.bodies[*i_swap].min += displace;
                self.bodies[*i_swap].max += displace;
            }
        }
    }

    fn get_displacement(min_i: f32, max_i: f32, min_j: f32, max_j: f32)
        -> f32 {
            if max_i - min_j < max_j - min_i {
                max_i - min_j
            } else {
                max_j - min_i
            }
    }
}



struct JumperEntityState<ID: Copy + Eq> {
    maps: Vec<StateMap<ID>>,
    conditions: Vec<Vec<bool>>,
    states: Vec<usize>
}

// 1. create condition table.
// 2. update condition table in project.
// 3. use condition table to change state.
// 4. ???
// 5. profit
impl<ID: Copy + Eq> JumperEntityState<ID> {
    pub fn new() -> Self {
        Self {
            // one map per entity
            maps: Vec::new(),
            conditions: Vec::new(),
            states: Vec::new()
        }
    }

    pub fn update(&mut self) {
        // update states
        for (i, state_idx) in self.states.iter_mut().enumerate() {
            for edge in &self.maps[i].states[*state_idx].edges {
                if self.conditions[i][*edge] {
                    *state_idx = *edge;
                }
            }
        }
    }

    pub fn get_id_for_entity(&self, ent: usize) -> ID {
        self.maps[ent].states[self.states[ent]].id
    }
}

struct StateMap<ID> {
    states: Vec<State<ID>>,
}

struct State<ID> {
    id: ID,
    edges: Vec<usize>
}

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
enum StateID {
    PlatformLeft,
    PlatformRight,
    PlayerGrounded,
    PlayerWalk,
    PlayerJump,
    PlayerFall,
}

struct Logics {
    control: WinitKeyboardControl<ActionID>,
    physics: JumperPhysics,
    collision: AabbCollision<CollisionID>,
    entity_state: JumperEntityState<StateID>,
}

impl Logics {
    fn new() -> Self {
        Self {
            control: {
                let mut control = WinitKeyboardControl::new();
                control.mapping.push(
                    InputMap {
                        inputs: vec![(KeyInput::Pair(VirtualKeyCode::Left, VirtualKeyCode::Right),
                                    InputState::On),
                        ],
                        actions: vec![Action {
                            id: ActionID::Move(Player::P1),
                            action_type: ActionType::Axis(-1.0, 1.0)
                        }]
                    }
                );
                control
            },
            physics: JumperPhysics::new(),
            collision: {
                let mut collision = AabbCollision::new();
                collision.bodies.push(Aabb::new(
                    Vec3::new(-1.0, 0.0, 0.0),
                    Vec3::new(0.0, HEIGHT as f32, 0.0)
                ));
                collision.bodies.push(Aabb::new(
                    Vec3::new(WIDTH as f32, 0.0, 0.0),
                    Vec3::new(WIDTH as f32 + 1.0, HEIGHT as f32, 0.0)
                ));
                collision.bodies.push(Aabb::new(
                    Vec3::new(0.0, -1.0, 0.0),
                    Vec3::new(WIDTH as f32, 0.0, 0.0)
                ));
                collision.bodies.push(Aabb::new(
                    Vec3::new(0.0, HEIGHT as f32, 0.0),
                    Vec3::new(WIDTH as f32, HEIGHT as f32 + 1.0, 0.0)
                ));
                for _ in 0..4 {
                    collision.velocities.push(Vec2::new(0.0, 0.0));
                }
                collision.metadata.push(CollisionData{ solid: true, fixed: true, id: CollisionID::RightWall });
                collision.metadata.push(CollisionData{ solid: true, fixed: true, id: CollisionID::LeftWall });
                collision.metadata.push(CollisionData{ solid: true, fixed: true, id: CollisionID::TopWall });
                collision.metadata.push(CollisionData{ solid: true, fixed: true, id: CollisionID::BottomWall });
                collision
            },
            entity_state: {
                let mut entity_state = JumperEntityState::new();
                entity_state.maps.push(StateMap {
                    states: {
                        let mut states = Vec::new();
                        states.push(State {
                            id: StateID::PlatformLeft,
                            edges: vec![1],
                        });
                        states.push(State {
                            id: StateID::PlatformRight,
                            edges: vec![0],
                        });
                        states
                    }
                });
                entity_state.maps.push(StateMap {
                    states: vec![State {
                            id: StateID::PlayerGrounded,
                            edges: vec![1, 2],
                        }, State {
                            id: StateID::PlayerWalk,
                            edges: vec![0, 2],
                        }, State {
                            id: StateID::PlayerJump,
                            edges: vec![3],
                        }, State {
                            id: StateID::PlayerFall,
                            edges: vec![0, 1],
                        }]
                });
                for map in &mut entity_state.maps {
                    entity_state.conditions.push(vec![false; map.states.len()]);
                }
                entity_state.states = vec![0, 3];
                entity_state
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Player {
    P1
}


struct World {
    player: (u8, u8),
    player_vel: Vec2,
    player_err: Vec2,
    ground: [u8; 4],
    platform: [u8; 4],
    platform_vel: Vec2,
}


fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("paddles")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };
    let mut hidpi_factor = window.scale_factor();

    let mut pixels = {
        let surface = Surface::create(&window);
        let surface_texture = SurfaceTexture::new(WIDTH as u32, HEIGHT as u32, surface);
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture)?
    };
    let mut world = World::new();
    let mut logics = Logics::new();

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.get_frame());
            if pixels
                .render()
                    .map_err(|e| panic!("pixels.render() failed: {}", e))
                    .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Adjust high DPI factor
            if let Some(factor) = input.scale_factor_changed() {
                hidpi_factor = factor;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize(size.width, size.height);
            }

            // Update internal state and request a redraw
            world.update(&mut logics, &input);
            window.request_redraw();
        }
    });
}

impl World {
    fn new() -> Self {
        Self {
            player: (50, 70),
            player_vel: Vec2::new(0.0, 0.0),
            player_err: Vec2::new(0.0, 0.0),
            ground: [0, 200, WIDTH, 55],
            platform: [25, 50, 55, 9],
            platform_vel: Vec2::new(-1.0, 0.0),
        }
    }

    fn update(&mut self, logics: &mut Logics, input: &WinitInputHelper) {
        self.project_control(&mut logics.control, &logics.entity_state);
        logics.control.update(input);
        self.unproject_control(&logics.control, &logics.entity_state);

        self.project_physics(&mut logics.physics);
        logics.physics.update();
        self.unproject_physics(&logics.physics);

        self.project_collision(&mut logics.collision);
        logics.collision.update();
        self.unproject_collision(&logics.collision);

        self.project_entity_state(&mut logics.entity_state, &logics.collision);
        logics.entity_state.update();
        self.unproject_entity_state(&logics.entity_state);
    }

    fn project_control(&self, control: &mut WinitKeyboardControl<ActionID>, entity_state: &JumperEntityState<StateID>) {
        for map in control.mapping.iter_mut() {
            map.inputs.resize(1, (KeyInput::Single(VirtualKeyCode::H), InputState::On));
            map.actions.resize_with(1, Action::default);
        }

        match entity_state.get_id_for_entity(1) {
            StateID::PlayerGrounded | StateID::PlayerWalk => {
                control.mapping[0].inputs.push(
                    (KeyInput::Single(VirtualKeyCode::Space), InputState::Pressed));
                control.mapping[0].actions.push(
                    Action { id: ActionID::Jump(Player::P1),
                    action_type: ActionType::Continuous(-1.0) });
            }
            _ => {}
        }
    }

    fn unproject_control(&mut self, control: &WinitKeyboardControl<ActionID>, _entity_state: &JumperEntityState<StateID>) {
        self.player_vel.x = control.values[0][0];
        if control.values[0].len() > 1 {
            self.player_vel.y = control.values[0][1];
        }
    }

    fn project_physics(&self, physics: &mut JumperPhysics) {
        physics.accelerations.resize_with(2, Vec2::default);
        physics.positions.resize_with(2, Vec2::default);
        physics.velocities.resize_with(2, Vec2::default);
        physics.accelerations[0] = Vec2::new(0.0, 0.03);
        physics.accelerations[1] = Vec2::new(0.0, 0.0);
        physics.positions[0] = Vec2::new(self.player.0 as f32, self.player.1 as f32);
        physics.positions[1] = Vec2::new(self.platform[0] as f32, self.platform[1] as f32);
        physics.velocities[0] = self.player_vel;
        physics.velocities[1] = self.platform_vel;
    }

    fn unproject_physics(&mut self, physics: &JumperPhysics) {
        self.player.0 = physics.positions[0].x.trunc().max(0.0).min((WIDTH - PLAYER_SIZE) as f32) as u8;
        self.player.1 = physics.positions[0].y.trunc().max(0.0).min((HEIGHT - PLAYER_SIZE) as f32) as u8;
        self.player_err = physics.positions[0] - Vec2::new(self.player.0 as f32, self.player.1 as f32);
        self.player_vel = physics.velocities[0];
        self.platform[0] = physics.positions[1].x.trunc().max(0.0).min((WIDTH - self.platform[2]) as f32) as u8;
        self.platform[1] = physics.positions[1].y.trunc().max(0.0).min((HEIGHT - self.platform[3]) as f32) as u8;
        self.platform_vel = physics.velocities[1];
    }

    fn project_collision(&self, collision: &mut AabbCollision<CollisionID>) {
        collision.bodies.resize_with(4, Aabb::default);
        collision.velocities.resize_with(4, Default::default);
        collision.metadata.resize_with(4, CollisionData::default);
        collision.bodies.push(Aabb::new(
                Vec3::new(self.player.0 as f32, self.player.1 as f32, 0.0),
                Vec3::new(self.player.0 as f32 + PLAYER_SIZE as f32, self.player.1 as f32 + PLAYER_SIZE as f32, 0.0)));
        collision.bodies.push(Aabb::new(
                Vec3::new(self.ground[0] as f32, self.ground[1] as f32, 0.0),
                Vec3::new((self.ground[0] + self.ground[2]) as f32,
                    (self.ground[1] + self.ground[3]) as f32, 0.0)));
        collision.bodies.push(Aabb::new(
                Vec3::new(self.platform[0] as f32, self.platform[1] as f32, 0.0),
                Vec3::new((self.platform[0] + self.platform[2]) as f32,
                    (self.platform[1] + self.platform[3]) as f32, 0.0)));

        collision.velocities.push(self.player_vel);
        collision.velocities.push(Vec2::new(0.0, 0.0));
        collision.velocities.push(self.platform_vel);

        collision.metadata.push(CollisionData { solid: true, fixed: false, id: CollisionID::Player(Player::P1) });
        collision.metadata.push(CollisionData { solid: true, fixed: true, id: CollisionID::Ground });
        collision.metadata.push(CollisionData { solid: true, fixed: true, id: CollisionID::Wall });
    }

    fn unproject_collision(&mut self, collision: &AabbCollision<CollisionID>) {
        self.player.0 = collision.bodies[4].min.x.trunc() as u8;
        self.player.1 = collision.bodies[4].min.y.trunc() as u8;
    }


    fn project_entity_state(&self, entity_state: &mut JumperEntityState<StateID>, collision: &AabbCollision<CollisionID>) {
        // update condition table
        for state_conditions in entity_state.conditions.iter_mut() {
            state_conditions.clear();
        }
        entity_state.conditions[0].resize(2, false);
        entity_state.conditions[1].resize(4, false);
        if self.platform[0] < 30 {
            entity_state.conditions[0][1] = true;
        }
        if self.platform[0] > 150 {
            entity_state.conditions[0][0] = true;
        }
        for contact in collision.contacts.iter() {
            match (collision.metadata[contact.0].id,
                collision.metadata[contact.1].id) {
                (CollisionID::Player(..), CollisionID::Ground) => {
                    if self.player_vel.x == 0.0 {
                        entity_state.conditions[1][0] = true;
                    } else {
                        entity_state.conditions[1][1] = true;
                    }
                }
                _ => {
                    if self.player_vel.y < 0.0 {
                        entity_state.conditions[1][2] = true;
                    } else {
                        entity_state.conditions[1][3] = true;
                    }
                }
            }
        }
    }

    fn unproject_entity_state(&mut self, entity_state: &JumperEntityState<StateID>) {
        for (map, state) in entity_state.maps.iter().zip(entity_state.states.iter()) {
            match map.states[*state].id {
                StateID::PlatformLeft => {
                    self.platform_vel.x = -1.0;
                }
                StateID::PlatformRight => {
                    self.platform_vel.x = 1.0;
                }
                _ => {}
            }
        }
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: [`wgpu::TextureFormat::Rgba8UnormSrgb`]
    fn draw(&self, frame: &mut [u8]) {
        for pixel in frame.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[128, 128, 255, 255]);
        }
        draw_rect(self.player.0, self.player.1, PLAYER_SIZE, PLAYER_SIZE, [0, 0, 0, 255], frame);
        draw_rect(self.ground[0], self.ground[1], self.ground[2], self.ground[3], [64, 64, 64, 255], frame);
        draw_rect(self.platform[0], self.platform[1], self.platform[2], self.platform[3], [64, 64, 64, 255], frame);
    }
}

fn draw_rect(x: u8, y: u8, w: u8, h: u8, color: [u8;4], frame: &mut [u8]) {
    let x = x.min(WIDTH-1) as usize;
    let w = (w as usize).min(WIDTH as usize-x);
    let y = y.min(HEIGHT-1) as usize;
    let h = (h as usize).min(HEIGHT as usize-y);
    for row in 0..h {
        let row_start = (WIDTH as usize)*4*(y+row);
        let slice = &mut frame[(row_start+x*4)..(row_start+(x+w)*4)];
        for pixel in slice.chunks_exact_mut(4) {
            pixel.copy_from_slice(&color);
        }
    }
}
