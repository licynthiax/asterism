#![allow(clippy::type_complexity)]
use crate::types::*;
use crate::*;

pub enum EngineAction {
    ChangeResource(Transaction<u16>),
    MoveTile(IVec2, IVec2),
    MoveEnt(Option<usize>, IVec2),
    MoveRoom(LinkID),
    AddEnt(Ent, IVec2),
    AddTile(usize, IVec2),
    AddPlayer(IVec2),
    MovePlayer(IVec2),
    MovePlayerBy(IVec2),
}

impl EngineAction {
    pub fn perform_action(&self, state: &mut State, logics: &mut Logics) {
        todo!()
    }
}

pub(crate) struct Events {
    pub control: Vec<(CtrlEvent, EngineAction)>,
    pub collision: Vec<(ColEvent, EngineAction)>,
    pub linking: Vec<(LinkingEvent, EngineAction)>,
    pub resource_event: Vec<(RsrcEvent, EngineAction)>,
}

impl Events {
    pub fn new() -> Self {
        Self {
            control: Vec::new(),
            collision: Vec::new(),
            linking: Vec::new(),
            resource_event: Vec::new(),
        }
    }
}

impl Game {
    pub fn add_ctrl_predicate(
        &mut self,
        action: ActionID,
        key_event: ControlEventType,
        on_key_event: EngineAction,
    ) {
        let key_event = CtrlEvent {
            event_type: key_event,
            action_id: action,
            set: 0,
        };
        self.events.control.push((key_event, on_key_event));
    }

    pub fn add_link_predicate(&mut self, from: LinkID, to: LinkID, when_traversed: EngineAction) {
        let to = self.logics.linking.graphs[0].graph.node_idx(&to).unwrap();
        let from = self.logics.linking.graphs[0].graph.node_idx(&from).unwrap();
        let event = LinkingEvent {
            graph: 0,
            node: to,
            event_type: LinkingEventType::Traversed(from),
        };

        self.events.linking.push((event, when_traversed));
    }

    pub fn add_collision_predicate(&mut self, col_event: ColEvent, on_collide: EngineAction) {
        self.events.collision.push((col_event, on_collide));
    }

    pub fn add_rsrc_predicate(&mut self, rsrc_event: RsrcEvent, on_rsrc_event: EngineAction) {
        self.events.resource_event.push((rsrc_event, on_rsrc_event));
    }
}
