use macroquad::{color::*, input::KeyCode, math::IVec2};

/// generates identifier structs (i got tired of typing all of them out)
macro_rules! id_impl_new {
    ($([$($derive:meta)*] $id_type:ident),*) => {
        $(
            $(#[$derive])*
            #[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

id_impl_new!([derive(Hash, Ord, PartialOrd)] TileID, [derive(Hash, Ord, PartialOrd)] CharacterID);

#[derive(Hash, Ord, PartialOrd, Clone, Copy, PartialEq, Eq, Debug)]
pub struct PoolID {
    pub(crate) attached_to: EntID,
    pub(crate) rsrc: RsrcID,
}

impl PoolID {
    pub fn new(attached_to: EntID, rsrc: RsrcID) -> Self {
        Self { attached_to, rsrc }
    }
}

#[derive(Hash, Ord, PartialOrd, Clone, Copy, PartialEq, Eq, Debug)]
pub struct RsrcID {
    idx: usize,
    name: &'static str,
}

impl RsrcID {
    pub fn new(idx: usize, name: &'static str) -> Self {
        Self { idx, name }
    }
    pub fn idx(&self) -> usize {
        self.idx
    }
    pub fn name(&self) -> &'static str {
        self.name
    }
}

pub enum Ent {
    /// tile id of tile to add, position, room
    TileID(TileID, IVec2, usize),
    Character(Character, usize),
}

// the stonks meme but it says derive
#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum EntID {
    Player,
    Tile(TileID),
    Character(CharacterID),
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum ActionID {
    Left,
    Right,
    Up,
    Down,
}

// players are unfixed
pub struct Player {
    pub pos: IVec2,
    pub amt_moved: IVec2,
    pub color: Color,
    pub inventory: Vec<(RsrcID, i16)>,
    pub controls: Vec<(ActionID, KeyCode, bool)>,
}

impl Player {
    pub fn new() -> Self {
        Self {
            pos: IVec2::ZERO,
            amt_moved: IVec2::ZERO,
            color: WHITE,
            inventory: Vec::new(),
            controls: vec![
                (ActionID::Up, KeyCode::Up, true),
                (ActionID::Down, KeyCode::Down, true),
                (ActionID::Left, KeyCode::Left, true),
                (ActionID::Right, KeyCode::Right, true),
            ],
        }
    }

    pub fn set_control_map(&mut self, action: ActionID, keycode: KeyCode, valid: bool) {
        let (_, keycode_old, valid_old) = self
            .controls
            .iter_mut()
            .find(|(act_id, ..)| *act_id == action)
            .unwrap();
        *keycode_old = keycode;
        *valid_old = valid;
    }

    pub fn add_inventory_item(&mut self, id: RsrcID, val: i16) {
        self.inventory.push((id, val));
    }
}

// tiles can be solid or not
#[derive(Clone, Copy)]
pub struct Tile {
    pub solid: bool,
    pub color: Color,
}

impl Tile {
    pub fn new() -> Self {
        Self {
            solid: false,
            // randomly generate tile color using hsl
            color: {
                use macroquad::rand::gen_range;
                hsl_to_rgb(
                    gen_range(0.0, 1.0),
                    gen_range(0.7, 1.0),
                    gen_range(0.3, 0.7),
                )
            },
        }
    }
}

// characters are fixed
#[derive(Clone)]
pub struct Character {
    /// resource id and starting value
    pub inventory: Vec<(RsrcID, i16)>,
    pub pos: IVec2,
    pub color: Color,
}

impl Character {
    pub fn new() -> Self {
        Self {
            inventory: Vec::new(),
            pos: IVec2::ZERO,
            color: LIME,
        }
    }

    pub fn add_inventory_item(&mut self, id: RsrcID, val: i16) {
        self.inventory.push((id, val));
    }
}

use crate::collision::Contact;
use asterism::control::ControlEvent;
use asterism::resources::ResourceEvent;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ColEntType {
    Player,
    Character,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CollisionEnt {
    Player,
    Tile(IVec2),
    Character(CharacterID),
}

pub type CtrlEvent = ControlEvent<ActionID>;
pub type ColEvent = (usize, Contact); // usize is the current room number
pub type RsrcEvent = ResourceEvent<PoolID, i16>;
