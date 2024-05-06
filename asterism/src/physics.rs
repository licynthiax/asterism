//! # Physics logics
//!
//! Physics logics communicate that physical laws govern the movement of some in-game entities. They update and honor objects' physical properties like position, velocity, density, etc., according to physical laws integrated over time.

use crate::{Event, EventType, LendingIterator, Logic, Reaction};
use macroquad::math::Vec2;

/// A physics logic using 2d points.
pub struct PointPhysics {
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub accelerations: Vec<Vec2>,
    pub events: Vec<PhysicsEvent>,
}

pub struct PointPhysData<'data> {
    pub pos: &'data Vec2,
    pub vel: &'data Vec2,
    pub acc: &'data Vec2,
}
pub struct PointPhysDataMut<'data> {
    pub pos: &'data mut Vec2,
    pub vel: &'data mut Vec2,
    pub acc: &'data mut Vec2,
}

impl Logic for PointPhysics {
    type Reaction = PhysicsReaction;
    type Event = PhysicsEvent;

    type Ident = usize;
    type IdentData<'a> = PointPhysData<'a> where Self: 'a;
    type IdentDataMut<'a> = PointPhysDataMut<'a> where Self: 'a;

    type DataIter<'a> = PtPhysicsDataIter<'a> where Self: 'a;

    fn handle_predicate(&mut self, reaction: &Self::Reaction) {
        match reaction {
            PhysicsReaction::SetPos(idx, pos) => {
                self.positions[*idx] = *pos;
            }
            PhysicsReaction::SetVel(idx, vel) => {
                self.velocities[*idx] = *vel;
            }
            PhysicsReaction::SetAcc(idx, acc) => {
                self.accelerations[*idx] = *acc;
            }
            PhysicsReaction::RemoveBody(idx) => {
                self.positions.remove(*idx);
                self.velocities.remove(*idx);
                self.accelerations.remove(*idx);
            }
            PhysicsReaction::AddBody { pos, vel, acc } => {
                self.add_physics_entity(*pos, *vel, *acc);
            }
        }
    }

    fn get_ident_data(&self, ident: Self::Ident) -> Self::IdentData<'_> {
        PointPhysData {
            pos: &self.positions[ident],
            vel: &self.velocities[ident],
            acc: &self.accelerations[ident],
        }
    }
    fn get_ident_data_mut(&mut self, ident: Self::Ident) -> Self::IdentDataMut<'_> {
        PointPhysDataMut {
            pos: &mut self.positions[ident],
            vel: &mut self.velocities[ident],
            acc: &mut self.accelerations[ident],
        }
    }

    fn data_iter(&mut self) -> Self::DataIter<'_> {
        PtPhysicsDataIter {
            physics: self,
            count: 0,
        }
    }
    fn events(&self) -> &[Self::Event] {
        &self.events
    }
}

impl PointPhysics {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            velocities: Vec::new(),
            accelerations: Vec::new(),
            events: Vec::new(),
        }
    }
    /// Update the physics logic: changes the velocities of entities based on acceleration, then changes entities' positions based on updated velocities.
    pub fn update(&mut self) {
        self.events.clear();

        for (i, ((pos, vel), acc)) in self
            .positions
            .iter_mut()
            .zip(self.velocities.iter_mut())
            .zip(self.accelerations.iter())
            .enumerate()
        {
            if acc.x.abs() > 0.0001 && acc.y.abs() > 0.0001 {
                self.events.push(PhysicsEvent {
                    ent: i,
                    event_type: PhysicsEventType::VelChange,
                });
            }
            if vel.x.abs() > 0.0001 && vel.y.abs() > 0.0001 {
                self.events.push(PhysicsEvent {
                    ent: i,
                    event_type: PhysicsEventType::PosChange,
                });
            }

            *vel += *acc;
            *pos += *vel;
        }
    }

    /// Adds a physics entity to the logic with the given position, velocity, and acceleration.
    pub fn add_physics_entity(&mut self, pos: Vec2, vel: Vec2, acc: Vec2) {
        self.positions.push(pos);
        self.velocities.push(vel);
        self.accelerations.push(acc);
    }

    /// Clears vecs from last frame
    pub fn clear(&mut self) {
        self.positions.clear();
        self.velocities.clear();
        self.accelerations.clear();
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum PhysicsReaction {
    SetPos(usize, Vec2),
    SetVel(usize, Vec2),
    SetAcc(usize, Vec2),
    RemoveBody(usize),
    AddBody { pos: Vec2, vel: Vec2, acc: Vec2 },
}
impl Reaction for PhysicsReaction {}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct PhysicsEvent {
    ent: usize,
    event_type: PhysicsEventType,
}

impl Event for PhysicsEvent {
    type EventType = PhysicsEventType;
    fn get_type(&self) -> &Self::EventType {
        &self.event_type
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PhysicsEventType {
    VelChange,
    PosChange,
}
impl EventType for PhysicsEventType {}

pub struct PtPhysicsDataIter<'phys> {
    physics: &'phys mut PointPhysics,
    count: usize,
}

impl<'phys> LendingIterator for PtPhysicsDataIter<'phys> {
    type Item<'a> = (<PointPhysics as Logic>::Ident, <PointPhysics as Logic>::IdentDataMut<'a>) where Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        if self.count == self.physics.positions.len() {
            None
        } else {
            self.count += 1;
            Some((
                self.count - 1,
                self.physics.get_ident_data_mut(self.count - 1),
            ))
        }
    }
}

pub struct PtPhysicsEventIter<'phys> {
    physics: &'phys PointPhysics,
    count: usize,
}

impl<'phys> LendingIterator for PtPhysicsEventIter<'phys> {
    type Item<'a> = &'a PhysicsEvent where Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        self.count += 1;
        if self.count == self.physics.events.len() {
            None
        } else {
            Some(&self.physics.events[self.count - 1])
        }
    }
}
