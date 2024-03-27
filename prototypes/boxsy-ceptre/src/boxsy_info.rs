#![allow(unused)]
/// types that describe the boxsy game and functions for facilitating working with them
use std::collections::BTreeSet;

#[derive(Debug)]
pub enum GameType {
    Character,
    Tile,
    Player,
    Rsrc,
    Room,
}

impl std::fmt::Display for GameType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            GameType::Character => fmt.write_str("GameType::Character"),
            GameType::Tile => fmt.write_str("GameType::Tile"),
            GameType::Player => fmt.write_str("GameType::Player"),
            GameType::Rsrc => fmt.write_str("GameType::Rsrc"),
            GameType::Room => fmt.write_str("GameType::Room"),
        }
    }
}

impl GameType {
    pub fn associated_logics(&self) -> BTreeSet<Logic> {
        match self {
            GameType::Player => BTreeSet::from([Logic::Collision, Logic::Control, Logic::Resource]),
            GameType::Tile => BTreeSet::from([Logic::Collision, Logic::Linking]),
            GameType::Character => {
                BTreeSet::from([Logic::Collision, Logic::Resource, Logic::Linking])
            }
            GameType::Rsrc => BTreeSet::from([Logic::Resource]),
            GameType::Room => BTreeSet::from([Logic::Linking]),
        }
    }
}

#[derive(serde::Deserialize, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum Logic {
    Collision,
    Control,
    Linking,
    Resource,
}

pub enum Event {
    ChangeResource,
    MoveRoom,
}

impl Event {
    pub fn associated_logics(&self) -> BTreeSet<Logic> {
        match self {
            Event::ChangeResource => {
                BTreeSet::from_iter([Logic::Resource, Logic::Collision, Logic::Control])
            }
            Event::MoveRoom => {
                BTreeSet::from_iter([Logic::Linking, Logic::Collision, Logic::Control])
            }
        }
    }
}

impl<'a> TryFrom<&'a str> for Logic {
    type Error = &'a str;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "collision" => Ok(Logic::Collision),
            "control" => Ok(Logic::Control),
            "linking" => Ok(Logic::Linking),
            "resource" => Ok(Logic::Resource),
            _ => Err("logic not in this list"),
        }
    }
}

impl std::fmt::Display for Logic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Logic::Collision => "Logic::Collision",
            Logic::Control => "Logic::Control",
            Logic::Linking => "Logic::Linking",
            Logic::Resource => "Logic::Resource",
        };
        f.write_str(s)
    }
}
