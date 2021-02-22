//! # Physics logics
//!
//! Physics logics communicate that physical laws govern the movement of some in-game entities.
//! They update and honor objects' physical properties like position, velocity, density, etc.,
//! according to physical laws integrated over time.

use crate::{collision::Vec2, Logic, LogicType};

/// A physics logic using 2d points.
pub struct PointPhysics<V2: Vec2> {
    pub positions: Vec<V2>,
    pub velocities: Vec<V2>,
    pub accelerations: Vec<V2>,
}

impl<V2: Vec2> Logic for PointPhysics<V2> {
    fn logic_type(&self) -> LogicType {
        LogicType::Physics
    }

    /// Update the physics logic: changes the velocities of entities based on acceleration, then
    /// changes entities' positions based on updated velocities.
    fn update(&mut self) {
        for (pos, (vel, acc)) in self
            .positions
            .iter_mut()
            .zip(self.velocities.iter_mut().zip(self.accelerations.iter()))
        {
            *vel += *acc;
            *pos += *vel;
        }
    }
}

impl<V2: Vec2> PointPhysics<V2> {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            velocities: Vec::new(),
            accelerations: Vec::new(),
        }
    }

    /// Adds a physics entity to the logic with the given position, velocity, and acceleration.
    pub fn add_physics_entity(&mut self, pos: V2, vel: V2, acc: V2) {
        self.positions.push(pos);
        self.velocities.push(vel);
        self.accelerations.push(acc);
    }
}
