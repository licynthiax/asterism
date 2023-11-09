//! # Asterism
//!
//! An asterism is a pattern people can see in stars, and while there is a fixed set of true constellations we can come up with as many asterisms as we like.
//!
//! Asterism is a project in operationalizing operational logics to the extent that they can be composed to form game engines. This means that instead of a monolithic `update()` function that combines different logics and extremely concrete instantiations of abstract processes, the game loop arbitrates its rules by configuring and calling out to a variety of logics.
//!
//! The descriptions of logics in the modules are lightly modified from Prof Osborn's dissertation.
//!
//! Requires at least Rust 1.68---if this doesn't compile, update your rustc.

#![allow(clippy::new_without_default)]
#![allow(clippy::upper_case_acronyms)]
pub mod collision;
pub mod control;
pub mod entity_state;
pub mod linking;
pub mod physics;
pub mod resources;

pub mod graph;
pub mod graphics;
pub mod lending_iterator;

pub use lending_iterator::LendingIterator;

/// An operational logic
pub trait Logic {
    /// the events that this logic can generate
    type Event: Event + Copy;
    /// the reactions that this logic can act on
    type Reaction: Reaction;

    /// a single unit/entity within the logic
    type Ident: Copy;
    /// the data of the logic associated with its identity (`<Self as Logic>::Ident`).
    type IdentData<'logic>
    where
        Self: 'logic;

    type DataIter<'logic>: LendingIterator<
        Item<'logic> = (<Self as Logic>::Ident, <Self as Logic>::IdentData<'logic>),
    >
    where
        Self: 'logic;

    /// processes the reaction if a condition is met
    fn handle_predicate(&mut self, reaction: &Self::Reaction);

    /// exposes the data associated with a particular ""entity"" of the logic.
    fn get_ident_data(&mut self, ident: Self::Ident) -> Self::IdentData<'_>;

    /// returns a [lending iterator]()
    fn data_iter(&mut self) -> Self::DataIter<'_>;
    fn events(&self) -> &[Self::Event];
}

/// An event produced by the logic. Holds both the data associated with the event and information about what the event is---these should be separated for easier matching.
pub trait Event {
    type EventType: EventType;
    fn get_type(&self) -> &Self::EventType;
}

pub trait EventType {}

pub trait Reaction {}
