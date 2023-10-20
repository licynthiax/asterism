//! # Entity-state Logics
//!
//! Entity-state logics communicate that game entities act in different ways or have different capabilities at different times, in ways that are intrinsic to each such entity. They govern the finite, discrete states of a set of game characters or other entities, update states when necessary, and condition the operators of other logics on entities' discrete states.

use crate::graph::StateMachine;
use crate::{Event, EventType, LendingIterator, Logic, Reaction};

/// An entity-state logic for flat entity state machines.
pub struct FlatEntityState<ID: Copy + Eq> {
    /// A vec of state machines
    pub graphs: Vec<StateMachine<ID>>,
    pub just_traversed: Vec<bool>,
    events: Vec<EntityEvent>,
}

impl<ID: Copy + Eq + 'static> FlatEntityState<ID> {
    pub fn new() -> Self {
        Self {
            graphs: Vec::new(),
            just_traversed: Vec::new(),
            events: Vec::new(),
        }
    }

    /// Updates the entity-state logic.
    ///
    /// Check the status of all the links from the current node in the condition table. If any of those links are `true`, i.e. that node can be moved to, move the current position.
    pub fn update(&mut self) {
        self.just_traversed.fill(false);
        self.events.clear();

        for (i, (graph, traversed)) in self
            .graphs
            .iter_mut()
            .zip(self.just_traversed.iter_mut())
            .enumerate()
        {
            let mut activated = graph
                .conditions
                .iter()
                .enumerate()
                .filter_map(|(node, activated)| {
                    if *activated {
                        Some(EntityEvent {
                            graph: i,
                            node,
                            event_type: EntityEventType::Activated,
                        })
                    } else {
                        None
                    }
                })
                .collect();
            self.events.append(&mut activated);

            for edge in graph.graph.get_edges(graph.current_node) {
                if graph.conditions[edge] {
                    *traversed = true;
                    self.events.push(EntityEvent {
                        graph: i,
                        node: edge,
                        event_type: EntityEventType::Traversed(graph.current_node),
                    });
                    graph.current_node = edge;
                    break;
                }
            }
        }
    }

    /// Gets the current state of the entity by its index
    pub fn get_id_for_entity(&self, ent: <Self as Logic>::Ident) -> ID {
        self.graphs[ent].get_current_node()
    }

    /// Adds a map of nodes to the logic.
    ///
    /// `starting_pos` is where the node the graph traversal starts on. `edges` is a list of adjacency lists. All conditions are set to false.
    pub fn add_graph<const NUM_NODES: usize>(
        &mut self,
        starting_pos: usize,
        edges: [(ID, &[ID]); NUM_NODES],
    ) {
        let mut graph = StateMachine::new();
        let (ids, edges): (Vec<_>, Vec<_>) = edges.iter().cloned().unzip();
        graph.add_nodes(ids.as_slice());
        graph.current_node = starting_pos;
        for (from, node_edges) in edges.iter().enumerate() {
            for to in node_edges.iter() {
                graph
                    .graph
                    .add_edge(from, ids.iter().position(|id| to == id).unwrap());
            }
        }
        self.graphs.push(graph);
        self.just_traversed.push(false);
    }
}

/// A representation of a map of states.
pub struct StateMap<ID> {
    pub states: Vec<State<ID>>,
}

/// A state in a state machine.
pub struct State<ID> {
    pub id: ID,
    /// The edges to the states that the entity can move to from the current state.
    pub edges: Vec<usize>,
}

#[derive(Copy, Clone)]
pub struct EntityEvent {
    pub graph: usize,
    pub node: usize,
    event_type: EntityEventType,
}

#[derive(Copy, Clone)]
pub enum EntityEventType {
    Activated,
    Traversed(usize), // last node (which edge)
}
impl EventType for EntityEventType {}

impl Event for EntityEvent {
    type EventType = EntityEventType;
    fn get_type(&self) -> &Self::EventType {
        &self.event_type
    }
}

pub enum EntityReaction {
    Activate(usize, usize),
    Traverse(usize, usize),
}

impl Reaction for EntityReaction {}

impl<ID: Copy + Eq + 'static> Logic for FlatEntityState<ID> {
    type Event = EntityEvent;
    type Reaction = EntityReaction;

    /// index of graph
    type Ident = usize;
    /// current position in logic
    type IdentData<'a> = &'a mut ID where Self: 'a;

    type DataIter<'logic> = FesDataIter<'logic, ID> where Self: 'logic;

    fn handle_predicate(&mut self, reaction: &Self::Reaction) {
        match reaction {
            EntityReaction::Activate(graph, node) => self.graphs[*graph].conditions[*node] = true,
            EntityReaction::Traverse(graph, node) => {
                self.graphs[*graph].set_current_node(*node);
                self.just_traversed[*graph] = true;
            }
        }
    }

    fn get_ident_data(&mut self, ident: Self::Ident) -> Self::IdentData<'_> {
        let graph = &mut self.graphs[ident];
        &mut graph.graph.nodes[graph.current_node]
    }

    fn data_iter(&mut self) -> Self::DataIter<'_> {
        Self::DataIter {
            ent_state: self,
            count: 0,
        }
    }
    fn events(&self) -> &[Self::Event] {
        &self.events
    }
}

pub struct FesDataIter<'logic, ID>
where
    ID: Copy + Eq + 'static,
{
    ent_state: &'logic mut FlatEntityState<ID>,
    count: usize,
}

impl<'logic, ID> LendingIterator for FesDataIter<'logic, ID>
where
    ID: Copy + Eq + 'static,
{
    type Item<'a> = (
        <FlatEntityState<ID> as Logic>::Ident,
        <FlatEntityState<ID> as Logic>::IdentData<'a>
    )
    where
        Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        self.count += 1;
        if self.count == self.ent_state.graphs.len() {
            None
        } else {
            Some((
                self.count - 1,
                self.ent_state.get_ident_data(self.count - 1),
            ))
        }
    }
}

pub struct FesEventIter<'logic, ID>
where
    ID: Copy + Eq + 'static,
{
    ent_state: &'logic FlatEntityState<ID>,
    count: usize,
}

impl<'logic, ID> LendingIterator for FesEventIter<'logic, ID>
where
    ID: Copy + Eq + 'static,
{
    type Item<'a> = &'a EntityEvent
    where
        Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        self.count += 1;
        if self.count == self.ent_state.events.len() {
            None
        } else {
            Some(&self.ent_state.events[self.count - 1])
        }
    }
}
