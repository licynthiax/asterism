use macroquad::{input::KeyCode, math::Vec2};

/// generates identifier structs (i got tired of typing all of them out). example: `id_impl_new!([derive(PartialOrd, Ord)] ScoreID)` expands out to
///
/// ```
/// #[derive(PartialOrd, Ord)]
/// #[derive(Clone, Copy, PartialEq, Eq)]
/// pub struct ScoreID(usize);
/// impl ScoreID {
///     pub fn new(idx: usize) -> Self {
///         Self(idx)
///     }
///     pub fn idx(&self) -> usize {
///         self.0
///     }
/// }
/// ```
macro_rules! id_impl_new {
    ($([$($derive:meta)*] $id_type:ident),*) => {
        $(
            $(#[$derive])*
            #[derive(Clone, Copy, PartialEq, Eq)]
            pub struct $id_type(usize);

            impl $id_type {
                pub fn new(idx: usize) -> Self {
                    Self(idx)
                }

                pub fn idx(&self) -> usize {
                    self.0
                }
            }
        )*
    };
}

id_impl_new!([] PaddleID, [] WallID, [] BallID, [derive(PartialOrd, Ord, Debug)] ScoreID, [derive(PartialOrd, Ord, Debug)] ActionID);

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum QueryType {
    CtrlEvent,
    CtrlIdent,
    ColEvent,
    ColIdent,
    PhysEvent,
    PhysIdent,
    RsrcEvent,
    RsrcIdent,
    BallCol,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CollisionEnt {
    Paddle,
    Wall,
    Ball,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum RsrcPool {
    Score(ScoreID),
}

#[derive(Clone)]
pub struct Paddle {
    pub pos: Vec2,
    pub size: Vec2,
    pub controls: Vec<(ActionID, KeyCode, bool)>,
}

impl Paddle {
    pub fn new(pos: Vec2, size: Vec2) -> Self {
        Self {
            pos,
            size,
            controls: Vec::new(),
        }
    }

    pub fn add_control_map(&mut self, keycode: KeyCode, valid: bool) -> ActionID {
        let act_id = ActionID(self.controls.len());
        self.controls.push((act_id, keycode, valid));
        act_id
    }
}

#[derive(Copy, Clone)]
pub struct Ball {
    pub pos: Vec2,
    pub size: Vec2,
    pub vel: Vec2,
}

impl Ball {
    pub fn new(pos: Vec2, size: Vec2) -> Self {
        Self {
            pos,
            size,
            vel: Vec2::ZERO,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Wall {
    pub pos: Vec2,
    pub size: Vec2,
}

impl Wall {
    pub fn new(pos: Vec2, size: Vec2) -> Self {
        Self { pos, size }
    }
}

#[derive(Copy, Clone)]
pub struct Score {
    pub value: u16,
}

impl Score {
    pub(crate) const MIN: u16 = 0;
    pub(crate) const MAX: u16 = u16::MAX;
    pub fn new(value: u16) -> Self {
        Self { value }
    }
}

use asterism::control::ControlEvent;

pub type CtrlEvent = ControlEvent<ActionID>;
pub type CtrlIdent<'a> = (usize, &'a [asterism::control::Action<ActionID, KeyCode>]);
pub type ColEvent = asterism::collision::Contact;
pub type ColIdent<'a> = (usize, asterism::collision::AabbColData<'a, CollisionEnt>);
pub type RsrcIdent = (RsrcPool, (u16, u16, u16));
pub type RsrcEvent = asterism::resources::ResourceEvent<RsrcPool>;
pub type PhysIdent<'a> = (usize, asterism::physics::PointPhysData<'a>);
pub type PhysEvent = asterism::physics::PhysicsEvent;
