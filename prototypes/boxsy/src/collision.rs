#![allow(clippy::comparison_chain)]
use std::collections::BTreeMap;
use std::fmt::Debug;

use asterism::{Event, LendingIterator, Logic, Reaction};
use macroquad::math::IVec2;

#[derive(Copy, Clone, Debug)]
pub struct CollisionData<ID: Copy> {
    pub solid: bool,
    pub fixed: bool,
    pub id: ID,
}

impl<ID: Copy> CollisionData<ID> {
    pub fn new(solid: bool, fixed: bool, id: ID) -> Self {
        Self { solid, fixed, id }
    }
}

pub struct TileMapCollision<TileID: Debug, EntID: Copy> {
    pub map: Vec<Vec<Option<TileID>>>,
    pub tile_solid: BTreeMap<TileID, bool>,
    pub positions: Vec<IVec2>,
    pub metadata: Vec<CollisionData<EntID>>,
    pub amt_moved: Vec<IVec2>,
    pub contacts: Vec<Contact>,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Contact {
    Ent(usize, usize),
    Tile(usize, IVec2),
}

use asterism::collision::CollisionEventType;
impl Event for Contact {
    type EventType = CollisionEventType;

    fn get_type(&self) -> &Self::EventType {
        &CollisionEventType::Touching
    }
}

pub enum CollisionReaction<TileID, EntID> {
    SetTileAtPos(IVec2, TileID),
    RemoveTileAtPos(IVec2),
    SetEntPos(usize, IVec2),
    SetEntVel(usize, IVec2),
    SetEntData(usize, bool, bool, EntID), // idx, solid, fixed, id
    RemoveEnt(usize),
}

#[derive(Clone, Copy)]
pub enum ColIdent {
    Position(IVec2),
    EntIdx(usize),
}

#[derive(Debug)]
pub enum TileMapColData<'logic, TileID, EntID>
where
    TileID: Debug,
    EntID: Debug,
{
    Position {
        pos: IVec2,
        solid: bool,
        id: &'logic TileID,
    },
    Ent {
        pos: &'logic IVec2,
        amt_moved: IVec2,
        solid: bool,
        fixed: bool,
        id: &'logic EntID,
    },
}

pub enum TileMapColDataMut<'logic, TileID, EntID> {
    Position {
        pos: IVec2,
        solid: bool,
        id: &'logic mut TileID,
    },
    Ent {
        pos: &'logic mut IVec2,
        amt_moved: IVec2,
        solid: bool,
        fixed: bool,
        id: &'logic mut EntID,
    },
}

impl<TileID, EntID> Reaction for CollisionReaction<TileID, EntID> {}

impl<TileID: Copy + Eq + Ord + Debug, EntID: Eq + Copy + Debug> Logic
    for TileMapCollision<TileID, EntID>
{
    type Event = Contact;
    type Reaction = CollisionReaction<TileID, EntID>;

    type Ident = ColIdent;
    type IdentData<'logic> = TileMapColData<'logic, TileID, EntID> where Self: 'logic;
    type IdentDataMut<'logic> = TileMapColDataMut<'logic, TileID, EntID> where Self: 'logic;

    type DataIter<'logic> = ColDataIter<'logic, TileID, EntID> where Self: 'logic;

    fn handle_predicate(&mut self, reaction: &Self::Reaction) {
        match reaction {
            CollisionReaction::SetTileAtPos(pos, id) => {
                self.map[pos.y as usize][pos.x as usize] = Some(*id);
            }
            CollisionReaction::RemoveTileAtPos(pos) => {
                self.map[pos.y as usize][pos.x as usize] = None;
            }
            CollisionReaction::SetEntPos(idx, pos) => {
                self.positions[*idx] = *pos;
            }
            CollisionReaction::SetEntVel(idx, vel) => {
                self.amt_moved[*idx] = *vel;
            }
            CollisionReaction::SetEntData(idx, solid, fixed, id) => {
                self.metadata[*idx].solid = *solid;
                self.metadata[*idx].fixed = *fixed;
                self.metadata[*idx].id = *id;
            }
            CollisionReaction::RemoveEnt(idx) => {
                self.positions.remove(*idx);
                self.amt_moved.remove(*idx);
                self.metadata.remove(*idx);
            }
        };
    }

    fn get_ident_data(&self, ident: Self::Ident) -> Self::IdentData<'_> {
        match ident {
            ColIdent::Position(pos) => {
                if self.map[pos.y as usize][pos.x as usize].is_none() {
                    panic!("no tile at position {}", pos);
                }
                let solid = self.tile_solid(&self.map[pos.y as usize][pos.x as usize].unwrap());
                let id = self.map[pos.y as usize][pos.x as usize].as_ref().unwrap();
                TileMapColData::Position { solid, id, pos }
            }
            ColIdent::EntIdx(idx) => {
                let meta = &self.metadata[idx];
                TileMapColData::Ent {
                    pos: &self.positions[idx],
                    amt_moved: self.amt_moved[idx],
                    solid: meta.solid,
                    fixed: meta.fixed,
                    id: &meta.id,
                }
            }
        }
    }
    fn get_ident_data_mut(&mut self, ident: Self::Ident) -> Self::IdentDataMut<'_> {
        match ident {
            ColIdent::Position(pos) => {
                if self.map[pos.y as usize][pos.x as usize].is_none() {
                    panic!("no tile at position {}", pos);
                }
                let solid = self.tile_solid(&self.map[pos.y as usize][pos.x as usize].unwrap());
                let id = self.map[pos.y as usize][pos.x as usize].as_mut().unwrap();
                TileMapColDataMut::Position { solid, id, pos }
            }
            ColIdent::EntIdx(idx) => {
                let meta = &mut self.metadata[idx];
                TileMapColDataMut::Ent {
                    pos: &mut self.positions[idx],
                    amt_moved: self.amt_moved[idx],
                    solid: meta.solid,
                    fixed: meta.fixed,
                    id: &mut meta.id,
                }
            }
        }
    }

    fn events(&self) -> &[Self::Event] {
        &self.contacts
    }

    fn data_iter(&mut self) -> Self::DataIter<'_> {
        Self::DataIter {
            collision: self,
            tile_count: IVec2::new(0, 0),
            ent_count: 0,
        }
    }
}

impl<TileID: Eq + Ord + Copy + Debug, EntID: Eq + Copy + Debug> TileMapCollision<TileID, EntID> {
    pub fn new(width: usize, height: usize) -> Self {
        let mut collision = Self {
            map: Vec::new(),
            tile_solid: BTreeMap::new(),
            positions: Vec::new(),
            metadata: Vec::new(),
            amt_moved: Vec::new(),
            contacts: Vec::new(),
        };
        collision.clear_and_resize_map(width, height);
        collision
    }

    pub fn clear_and_resize_map(&mut self, width: usize, height: usize) {
        self.map.clear();

        self.map.resize_with(height, || {
            let mut vec = Vec::with_capacity(width);
            vec.resize_with(width, || None);
            vec
        });
    }

    pub fn clear_entities_except(&mut self, id: EntID) {
        let ent_info = self
            .positions
            .iter()
            .zip(self.amt_moved.iter())
            .zip(self.metadata.iter());
        let mut positions = Vec::new();
        let mut amt_moved = Vec::new();
        let mut metadata = Vec::new();
        for ((pos, moved), meta) in ent_info.filter(|(_, meta)| meta.id == id) {
            positions.push(*pos);
            amt_moved.push(*moved);
            metadata.push(*meta);
        }

        std::mem::swap(&mut self.positions, &mut positions);
        std::mem::swap(&mut self.amt_moved, &mut amt_moved);
        std::mem::swap(&mut self.metadata, &mut metadata);
    }

    pub fn clear_tile_data(&mut self) {
        self.tile_solid.clear();
    }

    pub fn update(&mut self) {
        self.contacts.clear();

        // make sure everyone is IN BOUNDS
        for (pos, amt_moved) in self.positions.iter_mut().zip(self.amt_moved.iter_mut()) {
            let width = self.map[0].len() as i32;
            let height = self.map.len() as i32;
            if let Some(oob_direction) =
                TileMapCollision::<TileID, EntID>::in_bounds(width, height, *pos)
            {
                if oob_direction.x < 0 {
                    let delta = 0 - pos.x;
                    pos.x = 0;
                    amt_moved.x -= delta;
                } else if oob_direction.x > 0 {
                    let delta = pos.x - width + 1;
                    pos.x = width - 1;
                    amt_moved.x -= delta;
                }

                if oob_direction.y < 0 {
                    let delta = 0 - pos.y;
                    pos.y = 0;
                    amt_moved.y -= delta;
                } else if oob_direction.y > 0 {
                    let delta = pos.y - (height - 1);
                    pos.y = height - 1;
                    amt_moved.y -= delta;
                }
            }
        }

        // check for contacts
        // ent vs tile
        for (i, pos) in self.positions.iter().enumerate() {
            if self.tile_at_pos(pos).is_some() {
                self.contacts.push(Contact::Tile(i, *pos));
            }
        }

        // ent vs ent
        for (i, (pos_i, meta_i)) in self.positions.iter().zip(self.metadata.iter()).enumerate() {
            for (j, (pos_j, meta_j)) in self
                .positions
                .iter()
                .zip(self.metadata.iter())
                .enumerate()
                .skip(i + 1)
            {
                if pos_i == pos_j {
                    let mut i = i;
                    let mut j = j;

                    if meta_i.fixed && !meta_j.fixed {
                        std::mem::swap(&mut i, &mut j);
                    }

                    self.contacts.push(Contact::Ent(i, j));
                }
            }
        }

        // restitute
        for contact in self.contacts.iter() {
            match contact {
                Contact::Tile(i, pos) => {
                    if self.positions[*i] != *pos {
                        continue;
                    }
                    if !self.metadata[*i].solid || self.metadata[*i].fixed {
                        continue;
                    }
                    let pos = &mut self.positions[*i];
                    let moved = &mut self.amt_moved[*i];
                    if *moved == IVec2::ZERO {
                        return;
                    }
                    let norm_moved = normalize(*moved);
                    while let Some(tile) = self.map[pos.y as usize][pos.x as usize] {
                        if Some(&true) == self.tile_solid.get(&tile) {
                            *pos -= norm_moved;
                            *moved -= norm_moved;
                        } else {
                            break;
                        }
                    }
                }

                Contact::Ent(i, j) => {
                    if self.positions[*i] != self.positions[*j] {
                        continue;
                    }
                    if !self.metadata[*i].solid
                        || self.metadata[*i].fixed
                        || !self.metadata[*j].solid
                    {
                        continue;
                    }
                    if self.amt_moved[*i] == IVec2::ZERO {
                        continue;
                    }

                    if self.metadata[*j].fixed {
                        let pos = &mut self.positions[*i];
                        let moved = &mut self.amt_moved[*i];
                        let norm_moved = normalize(*moved);
                        *pos -= norm_moved;
                        *moved -= norm_moved;
                        continue;
                    }

                    // if both are not fixed
                    if self.amt_moved[*i].as_vec2().length_squared()
                        > self.amt_moved[*i].as_vec2().length_squared()
                    {
                        let norm_moved_i = normalize(self.amt_moved[*i]);
                        let pos_i = &mut self.positions[*i];
                        let moved_i = &mut self.amt_moved[*i];

                        *pos_i -= norm_moved_i;
                        *moved_i -= norm_moved_i;
                    } else {
                        let norm_moved_j = normalize(self.amt_moved[*j]);
                        let pos_j = &mut self.positions[*j];
                        let moved_j = &mut self.amt_moved[*j];
                        *pos_j -= norm_moved_j;
                        *moved_j -= norm_moved_j;
                    }
                }
            }
        }

        self.amt_moved.clear();
        self.amt_moved.resize(self.positions.len(), IVec2::ZERO);
    }

    pub fn tile_at_pos(&self, pos: &IVec2) -> &Option<TileID> {
        &self.map[pos.y as usize][pos.x as usize]
    }

    pub fn tile_at_pos_mut(&mut self, pos: &IVec2) -> &mut Option<TileID> {
        &mut self.map[pos.y as usize][pos.x as usize]
    }

    pub fn tile_solid(&self, tile_id: &TileID) -> bool {
        *self
            .tile_solid
            .get(tile_id)
            .unwrap_or_else(|| panic!("not specified if tile {:?} is solid or not", tile_id))
    }

    fn in_bounds(map_width: i32, map_height: i32, pos: IVec2) -> Option<IVec2> {
        let mut direction = IVec2::ZERO;
        if pos.x >= map_width {
            direction.x = 1;
        } else if pos.x < 0 {
            direction.x = -1;
        }
        if pos.y >= map_height {
            direction.y = 1;
        } else if pos.y < 0 {
            direction.y = -1;
        }
        if direction == IVec2::ZERO {
            None
        } else {
            Some(direction)
        }
    }
}

fn normalize(vec2: IVec2) -> IVec2 {
    let mut vec2 = vec2;
    if vec2.x > 1 {
        vec2.x = 1;
    }
    if vec2.x < -1 {
        vec2.x = -1;
    }
    if vec2.y > 1 {
        vec2.y = 1;
    }
    if vec2.y < -1 {
        vec2.y = -1;
    }
    vec2
}

pub struct ColDataIter<'logic, TileID, EntID>
where
    TileID: Copy + Eq + Ord + Debug,
    EntID: Copy,
{
    collision: &'logic mut TileMapCollision<TileID, EntID>,
    tile_count: IVec2,
    ent_count: usize,
}

impl<'logic, TileID, EntID> LendingIterator for ColDataIter<'logic, TileID, EntID>
where
    TileID: Copy + Eq + Ord + Debug,
    EntID: Copy + Eq + Debug,
{
    type Item<'a> = (ColIdent, TileMapColDataMut<'a, TileID, EntID>) where Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        let inc_pos = |pos: &mut IVec2, len: usize| {
            let IVec2 { x, y } = pos;
            if *x + 1 < len as i32 {
                *x += 1;
            } else {
                *x = 0;
                *y += 1;
            }
        };

        // find next tile in map
        while TileMapCollision::<TileID, EntID>::in_bounds(
            self.collision.map[0].len() as i32,
            self.collision.map.len() as i32,
            self.tile_count,
        )
        .is_none()
            && self.collision.tile_at_pos(&self.tile_count).is_none()
        {
            inc_pos(&mut self.tile_count, self.collision.map.len());
        }

        if TileMapCollision::<TileID, EntID>::in_bounds(
            self.collision.map[0].len() as i32,
            self.collision.map.len() as i32,
            self.tile_count,
        )
        .is_none()
        {
            let id = ColIdent::Position(self.tile_count);
            inc_pos(&mut self.tile_count, self.collision.map.len());
            return Some((id, self.collision.get_ident_data_mut(id)));
        }

        if self.ent_count < self.collision.positions.len() {
            let id = ColIdent::EntIdx(self.ent_count);
            self.ent_count += 1;
            return Some((id, self.collision.get_ident_data_mut(id)));
        }

        None
    }
}
