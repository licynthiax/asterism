//! # Linking logics
//!
//! Linking logics present the idea that some things, in some context, are connected to each other. They maintain, enumerate, and follow/activate directed connections between concepts.
//!
//! Linking logics are incredibly broad and have a wide range of uses.
use crate::graph::StateMachine;
use crate::{Event, EventType, LendingIterator, Logic, Reaction};

/// A generic linking logic. See [StateMachine][crate::graph::StateMachine] documentation for more information.
///
/// I think this is the exact same code as FlatEntityState actually. The difference might make become more clear when rendering?
pub struct GraphedLinking<NodeID: Copy + Eq> {
    /// A vec of state machines
    pub graphs: Vec<StateMachine<NodeID>>,
    /// If the state machine has just traversed an edge or not
    pub just_traversed: Vec<bool>,
    events: Vec<LinkingEvent>,
}

impl<NodeID: Copy + Eq + 'static> GraphedLinking<NodeID> {
    pub fn new() -> Self {
        Self {
            graphs: Vec::new(),
            just_traversed: Vec::new(),
            events: Vec::new(),
        }
    }

    /// Updates the linking logic.
    ///
    /// Check the status of all the links from the current node in the condition table. If any of those links are `true`, i.e. that node can be moved to, move the current position.
    pub fn update(&mut self) {
        self.just_traversed.fill(false);
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
                        Some(LinkingEvent {
                            graph: i,
                            node,
                            event_type: LinkingEventType::Activated,
                        })
                    } else {
                        None
                    }
                })
                .collect();
            self.events.append(&mut activated);

            for edge in graph.graph.get_edges(graph.current_node) {
                if graph.conditions[i] {
                    *traversed = true;
                    self.events.push(LinkingEvent {
                        graph: i,
                        node: edge,
                        event_type: LinkingEventType::Traversed(graph.current_node),
                    });
                    graph.current_node = i;
                    break;
                }
            }
        }
    }

    /// Adds a map of nodes to the logic.
    ///
    /// `starting_pos` is where the node the graph traversal starts on. `edges` is a list of adjacency lists. All conditions are set to false.
    ///
    /// const generics <3
    pub fn add_graph<const NUM_NODES: usize>(
        &mut self,
        starting_pos: usize,
        edges: [(NodeID, &[NodeID]); NUM_NODES],
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

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct LinkingEvent {
    pub graph: usize,
    pub node: usize,
    pub event_type: LinkingEventType,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum LinkingEventType {
    Activated,
    Traversed(usize), // last node (which edge)
}
impl EventType for LinkingEventType {}

impl Event for LinkingEvent {
    type EventType = LinkingEventType;
    fn get_type(&self) -> &Self::EventType {
        &self.event_type
    }
}

pub enum LinkingReaction {
    Activate(usize, usize),
    Traverse(usize, usize),
    // AddNode(usize),
    // AddEdge(usize, (usize, usize))
    // RemoveNode(usize),
    // RemoveEdge(usize, (usize, usize)),
}

impl Reaction for LinkingReaction {}

impl<NodeID: Copy + Eq + 'static> Logic for GraphedLinking<NodeID> {
    type Event = LinkingEvent;
    type Reaction = LinkingReaction;

    /// index of graph
    type Ident = usize;
    /// list of graph nodes and edges
    type IdentData<'a> = &'a mut NodeID where Self: 'a;

    type DataIter<'logic> = LinkingDataIter<'logic, NodeID> where Self: 'logic;

    fn handle_predicate(&mut self, reaction: &Self::Reaction) {
        match reaction {
            LinkingReaction::Activate(graph, node) => self.graphs[*graph].conditions[*node] = true,
            LinkingReaction::Traverse(graph, node) => {
                self.just_traversed[*graph] = true;
                self.graphs[*graph].set_current_node(*node);
            }
        }
    }

    fn get_ident_data(&mut self, ident: Self::Ident) -> Self::IdentData<'_> {
        let graph = &mut self.graphs[ident];
        &mut graph.graph.nodes[graph.current_node]
    }

    fn data_iter(&mut self) -> Self::DataIter<'_> {
        Self::DataIter {
            linking: self,
            count: 0,
        }
    }
    fn events(&self) -> &[Self::Event] {
        &self.events
    }
}

pub struct LinkingDataIter<'logic, ID>
where
    ID: Copy + Eq + 'static,
{
    linking: &'logic mut GraphedLinking<ID>,
    count: usize,
}

impl<'logic, ID> LendingIterator for LinkingDataIter<'logic, ID>
where
    ID: Copy + Eq + 'static,
{
    type Item<'a> = (
        <GraphedLinking<ID> as Logic>::Ident,
        <GraphedLinking<ID> as Logic>::IdentData<'a>
    )
    where
        Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        self.count += 1;
        if self.count == self.linking.graphs.len() {
            None
        } else {
            Some((self.count - 1, self.linking.get_ident_data(self.count - 1)))
        }
    }
}

pub struct LinkingEventIter<'logic, ID>
where
    ID: Copy + Eq + 'static,
{
    linking: &'logic GraphedLinking<ID>,
    count: usize,
}

impl<'logic, ID> LendingIterator for LinkingEventIter<'logic, ID>
where
    ID: Copy + Eq + 'static,
{
    type Item<'a> = &'a LinkingEvent
    where
        Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        self.count += 1;
        if self.count == self.linking.events.len() {
            None
        } else {
            Some(&self.linking.events[self.count - 1])
        }
    }
}
