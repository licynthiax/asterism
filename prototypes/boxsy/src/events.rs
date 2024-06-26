#![allow(clippy::type_complexity)]
use crate::types::*;
use crate::*;

pub enum EngineAction {
    /// change resource
    ChangeResource(PoolID, Transaction<i16, PoolID>),
    MoveTile(IVec2, IVec2),
    MoveCharacter(Option<CharacterID>, IVec2),
    /// move room-- original room, and position
    MoveRoom(usize, IVec2),
    /// adds a character in a room (`usize`)
    AddCharacter(Character, usize),
    /// adds a tile with a tile id, in a room (`usize`), and with a position
    AddTile(TileID, usize, IVec2),
    MovePlayer(IVec2),
    MovePlayerBy(IVec2),
}

impl std::fmt::Debug for EngineAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineAction::ChangeResource(_, _) => write!(f, "EngineAction::ChangeResource"),
            EngineAction::MoveTile(_, _) => write!(f, "EngineAction::MoveTile"),
            EngineAction::MoveCharacter(_, _) => write!(f, "EngineAction::MoveCharacter"),
            EngineAction::MoveRoom(_, _) => write!(f, "EngineAction::MoveRoom"),
            EngineAction::AddCharacter(_, _) => write!(f, "EngineAction::AddCharacter"),
            EngineAction::AddTile(_, _, _) => write!(f, "EngineAction::AddTile"),
            EngineAction::MovePlayer(_) => write!(f, "EngineAction::MovePlayer"),
            EngineAction::MovePlayerBy(_) => write!(f, "EngineAction::MovePlayerBy"),
        }
    }
}

impl EngineAction {
    pub fn perform_action(&self, state: &mut State, logics: &mut Logics) {
        match &self {
            Self::ChangeResource(pool, transaction) => {
                logics
                    .resources
                    .handle_predicate(&(pool.clone(), transaction.clone()));
            }
            Self::MoveTile(old_pos, new_pos) => {
                if let Some(id) = logics.collision.tile_at_pos(old_pos) {
                    logics
                        .collision
                        .handle_predicate(&CollisionReaction::SetTileAtPos(*new_pos, *id));
                    logics
                        .collision
                        .handle_predicate(&CollisionReaction::RemoveTileAtPos(*old_pos));
                }
            }
            Self::MoveCharacter(Some(ch), new_pos) => {
                let room = logics.linking.graphs[0].get_current_node();
                let col_idx = state.get_col_idx(*ch, Some(room)).unwrap();
                logics
                    .collision
                    .handle_predicate(&CollisionReaction::SetEntPos(col_idx, *new_pos));
            }
            Self::MoveCharacter(None, _) => {}
            Self::MoveRoom(to, pos) => {
                // this is the current room
                let from = logics.linking.graphs[0].get_current_node();
                logics
                    .linking
                    .handle_predicate(&LinkingReaction::Traverse(0, *to));
                entities::set_current_room(state, logics, from, *to);
                logics
                    .collision
                    .handle_predicate(&CollisionReaction::SetEntPos(0, *pos));
            }
            Self::AddCharacter(ch, room) => {
                state.queue_add(Ent::Character(ch.clone(), *room));
            }
            Self::AddTile(tile_id, room, pos) => {
                state.queue_add(Ent::TileID(*tile_id, *pos, *room))
            }
            Self::MovePlayer(pos) => {
                logics
                    .collision
                    .handle_predicate(&CollisionReaction::SetEntPos(0, *pos));
            }
            Self::MovePlayerBy(delta) => {
                let pos = match logics.collision.get_ident_data(ColIdent::EntIdx(0)) {
                    TileMapColData::Ent { pos, .. } => *pos,
                    _ => unreachable!(),
                };
                logics
                    .collision
                    .handle_predicate(&CollisionReaction::SetEntPos(0, pos + *delta));
                logics
                    .collision
                    .handle_predicate(&CollisionReaction::SetEntVel(0, *delta));
            }
        }
    }
}

pub struct Events {
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

    pub fn add_link_predicate(&mut self, to: usize, from: usize, when_traversed: EngineAction) {
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
