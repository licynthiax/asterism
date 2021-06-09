use std::collections::BTreeMap;

use asterism::control::Values;
use macroquad::{color::*, input::KeyCode, math::IVec2};

/// generates identifier structs (i got tired of typing all of them out)
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

id_impl_new!([derive(PartialOrd, Ord)] PlayerID, [derive(Ord, PartialOrd)] TileID, [derive(Ord, PartialOrd)] CharacterID, [derive(PartialOrd, Ord)] RsrcID, [derive(PartialOrd, Ord)] ActionID);

pub enum Ent {
    Player(Player),
    Tile(Tile),
    Character(Character),
}

#[derive(PartialOrd, Ord, PartialEq, Eq)]
pub enum EntID {
    Player(PlayerID),
    Tile(TileID),
    Character(CharacterID),
}

// players are unfixed
pub struct Player {
    pub pos: IVec2,
    pub color: Color,
    pub inventory: BTreeMap<RsrcID, (u16, u16, u16)>,
    pub controls: Vec<(ActionID, KeyCode, bool, Values)>,
}

impl Player {
    pub fn new() -> Self {
        Self {
            pos: IVec2::ZERO,
            color: WHITE,
            inventory: BTreeMap::new(),
            controls: Vec::new(),
        }
    }

    pub fn add_control_map(&mut self, keycode: KeyCode, valid: bool) -> ActionID {
        let act_id = ActionID(self.controls.len());
        self.controls.push((act_id, keycode, valid, Values::new()));
        act_id
    }
}

// tiles can be solid or not
pub struct Tile {
    pub solid: bool,
    pub color: Color,
}

impl Tile {
    pub fn new() -> Self {
        Self {
            solid: false,
            color: SKYBLUE,
        }
    }
}

// characters are fixed
pub struct Character {
    pub pos: IVec2,
    pub color: Color,
}

impl Character {
    pub fn new() -> Self {
        Self {
            pos: IVec2::ZERO,
            color: LIME,
        }
    }
}

#[derive(Default)]
pub struct Resource {
    pub val: u16,
    pub min: u16,
    pub max: u16,
}

impl Resource {
    pub fn new() -> Self {
        Self::default()
    }
}

use crate::collision::Contact;
use asterism::control::ControlEvent;
use asterism::resources::ResourceEvent;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CollisionEnt {
    Player,
    Character,
}

pub type CtrlEvent = ControlEvent<ActionID>;
pub type ColEvent = Contact;
pub type RsrcEvent = ResourceEvent<RsrcID>;
