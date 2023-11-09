use std::collections::BTreeMap;
use std::fmt::Debug;

use asterism::{Event, LendingIterator, Logic, Reaction};
use macroquad::math::IVec2;

pub struct CollisionData<ID> {
    pub solid: bool,
    pub fixed: bool,
    pub id: ID,
}

impl<ID> CollisionData<ID> {
    pub fn new(solid: bool, fixed: bool, id: ID) -> Self {
        Self { solid, fixed, id }
    }
}

pub struct TileMapCollision<TileID: Debug, EntID> {
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

pub enum TileMapColData<'logic, TileID, EntID> {
    Position {
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

impl<TileID: Copy + Eq + Ord + Debug, EntID: Copy> Logic for TileMapCollision<TileID, EntID> {
    type Event = Contact;
    type Reaction = CollisionReaction<TileID, EntID>;

    type Ident = ColIdent;
    type IdentData<'logic> = TileMapColData<'logic, TileID, EntID> where Self: 'logic;

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

    fn get_ident_data(&mut self, ident: Self::Ident) -> Self::IdentData<'_> {
        match ident {
            ColIdent::Position(pos) => {
                if self.map[pos.y as usize][pos.x as usize].is_none() {
                    panic!("no tile at position {}", pos);
                }
                let solid = self.tile_solid(&self.map[pos.y as usize][pos.x as usize].unwrap());
                let id = self.map[pos.y as usize][pos.x as usize].as_mut().unwrap();
                TileMapColData::Position { solid, id }
            }
            ColIdent::EntIdx(idx) => {
                let meta = &mut self.metadata[idx];
                TileMapColData::Ent {
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

impl<TileID: Eq + Ord + Copy + Debug, EntID> TileMapCollision<TileID, EntID> {
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

    pub fn clear_entities(&mut self) {
        self.positions.clear();
        self.amt_moved.clear();
        self.metadata.clear();
    }

    pub fn clear_tile_data(&mut self) {
        self.tile_solid.clear();
    }

    pub fn update(&mut self) {
        self.contacts.clear();

        // check for contacts
        // ent vs tile
        for (i, pos_i) in self.positions.iter().enumerate() {
            if self.tile_at_pos(pos_i).is_some() {
                self.contacts.push(Contact::Tile(i, *pos_i));
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
                    if let Some(tile_id) = self.tile_at_pos(pos) {
                        if self.tile_solid(tile_id) {
                            let moved = normalize(self.amt_moved[*i]);
                            let mut pos = self.positions[*i];
                            pos -= moved;
                            self.restitute_ent(&mut pos, moved);
                            self.positions[*i] = pos;
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
                    let moved = normalize(self.amt_moved[*i]);
                    let mut pos = self.positions[*i];
                    pos -= moved;
                    self.restitute_ent(&mut pos, moved);
                    self.positions[*i] = pos;

                    if !self.metadata[*j].fixed {
                        let moved = normalize(self.amt_moved[*j]);
                        let mut pos = self.positions[*j];
                        self.restitute_ent(&mut pos, moved);
                        self.positions[*j] = pos;
                    }
                }
            }
        }
    }

    fn restitute_ent(&self, pos: &mut IVec2, moved: IVec2) {
        if moved == IVec2::ZERO {
            // this is miserable
            if !self.in_bounds(*pos - IVec2::new(0, 1)) {
                *pos -= IVec2::new(0, 1);
            } else if !self.in_bounds(*pos + IVec2::new(0, 1)) {
                *pos += IVec2::new(0, 1);
            } else if !self.in_bounds(*pos - IVec2::new(1, 0)) {
                *pos -= IVec2::new(1, 0);
            } else if !self.in_bounds(*pos + IVec2::new(1, 0)) {
                *pos += IVec2::new(1, 0);
            }
        }
        let mut new_pos = *pos;
        // check collision against map
        while let Some(tile_id) = self.map[new_pos.y as usize][new_pos.x as usize] {
            if self.tile_solid(&tile_id) {
                new_pos -= moved;
                if !self.in_bounds(new_pos) {
                    break;
                }
            } else {
                *pos = new_pos;
                break;
            }
        }
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

    pub fn in_bounds(&self, pos: IVec2) -> bool {
        pos.x < self.map[0].len() as i32
            && pos.y < self.map.len() as i32
            && pos.x >= 0
            && pos.y >= 0
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
    EntID: Copy,
{
    type Item<'a> = (ColIdent, TileMapColData<'a, TileID, EntID>) where Self: 'a;

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
        while self.collision.in_bounds(self.tile_count)
            && self.collision.tile_at_pos(&self.tile_count).is_none()
        {
            inc_pos(&mut self.tile_count, self.collision.map.len());
        }

        if self.collision.in_bounds(self.tile_count) {
            let id = ColIdent::Position(self.tile_count);
            inc_pos(&mut self.tile_count, self.collision.map.len());
            return Some((id, self.collision.get_ident_data(id)));
        }

        if self.ent_count < self.collision.positions.len() {
            let id = ColIdent::EntIdx(self.ent_count);
            self.ent_count += 1;
            return Some((id, self.collision.get_ident_data(id)));
        }

        None
    }
}
