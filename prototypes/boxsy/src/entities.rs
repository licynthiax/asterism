use crate::*;
use asterism::resources::PoolValues;

impl Game {
    pub fn set_background(&mut self, color: Color) {
        self.draw.background_color = color;
    }

    pub fn set_player(&mut self, player: Player) {
        self.draw.colors.insert(EntID::Player, player.color);
        self.logics.consume_player(player);

        self.state.player = true;
    }

    pub fn add_character(&mut self, character: Character, room: usize) -> CharacterID {
        let id = CharacterID::new(self.state.char_id_max);

        for (rsrc_id, val) in character.inventory.iter() {
            self.logics.resources.items.insert(
                PoolID::new(EntID::Character(id), rsrc_id.clone()),
                asterism::resources::PoolValues {
                    val: *val,
                    min: 0,
                    max: i16::MAX,
                },
            );
        }

        self.state.rooms[room].chars.push((id, character.pos));

        self.draw
            .colors
            .insert(EntID::Character(id), character.color);

        self.state.char_id_max += 1;
        id
    }

    pub fn log_rsrc(&mut self, name: String) -> RsrcID {
        let id = RsrcID::new(self.state.rsrc_id_max, name);
        self.state.rsrc_id_max += 1;
        self.state.resources.push(id.clone());
        id
    }

    pub fn add_link(&mut self, from: (usize, CollisionEnt), to: (usize, IVec2)) {
        self.logics.linking.graphs[0].graph.add_edge(from.0, to.0);

        match from.1 {
            CollisionEnt::Player => unreachable!(),
            CollisionEnt::Tile(pos) => {
                self.add_collision_predicate(
                    (from.0, CollisionEnt::Player, CollisionEnt::Tile(pos)),
                    EngineAction::MoveRoom(to.0, to.1),
                );
            }
            CollisionEnt::Character(id) => {
                self.add_collision_predicate(
                    (from.0, CollisionEnt::Player, CollisionEnt::Character(id)),
                    EngineAction::MoveRoom(to.0, to.1),
                );
            }
        }
    }

    pub fn set_num_rooms(&mut self, rooms: usize) {
        self.state.rooms.resize_with(rooms, Room::default);
    }

    /// Loads a tilemap with maximum 10 different kinds of tiles (numbers 0-9). A space (' ') marks a place on the map without any tiles. The tile types are read from in the parameter `tiles`.
    ///
    /// # Example
    ///
    /// ```
    /// let map = r#"0000000
    /// 0     0
    /// 0   1 0
    /// 0 1   0
    /// 0   2 0
    /// 0     0
    /// 0     0
    /// 0000000"#;
    ///
    /// game.add_room_from_str(map);
    /// ```
    pub fn add_room_from_str(&mut self, map: &str) -> Result<usize, String> {
        let map_length = WORLD_SIZE * WORLD_SIZE + WORLD_SIZE - 1;
        #[allow(clippy::comparison_chain)]
        if map.len() > map_length {
            return Err("map is too big".to_string());
        } else if map.len() < map_length {
            return Err("map is too small".to_string());
        }

        let room = self.state.rooms.len();
        self.state.rooms.push(Room::default());

        let mut x = 0;
        let mut y = 0;

        for ch in map.chars() {
            if ch.is_ascii_digit() {
                let tile_idx = ch.to_string().parse::<usize>().unwrap();
                if tile_idx > self.state.tile_type_count {
                    return Err(format!("tile {} not found", tile_idx));
                }
                self.add_tile_at_pos(TileID::new(tile_idx), room, IVec2::new(x, y));
                x += 1;
            } else if ch == ' ' {
                x += 1;
            } else if ch == '\n' {
                y += 1;
                x = 0;
            } else {
                return Err(format!("unrecognized character: '{}'", ch));
            }
        }
        self.logics.linking.graphs[0].add_node(room);

        Ok(self.state.rooms.len() - 1)
    }

    pub fn log_tile_info(&mut self, tile: Tile) -> TileID {
        let id = TileID::new(self.state.tile_type_count);
        self.state.tile_type_count += 1;
        self.draw.colors.insert(EntID::Tile(id), tile.color);

        self.logics.collision.tile_solid.insert(id, tile.solid);

        id
    }

    pub fn add_tile_at_pos(&mut self, tile: TileID, room: usize, pos: IVec2) {
        self.state.rooms[room].map[pos.y as usize][pos.x as usize] = Some(tile);
    }

    pub(crate) fn remove_player(&mut self) {
        self.logics
            .collision
            .handle_predicate(&CollisionReaction::RemoveEnt(0));

        let mut remove = Vec::new();
        for (idx, ((_, ent1, ent2), _)) in self.events.collision.iter_mut().enumerate() {
            if *ent1 == CollisionEnt::Player || *ent2 == CollisionEnt::Player {
                remove.push(idx);
            }
        }

        for i in remove.iter().rev() {
            let _ = self.events.collision.remove(*i);
        }
        self.state.player = false;
    }

    pub(crate) fn remove_character(&mut self, character: CharacterID) {
        let current_room = self.get_current_room();
        let mut ent_idx = None;
        for (i, room) in self.state.rooms.iter().enumerate() {
            if let Some(idx) = room.chars.iter().position(|cid| cid.0 == character) {
                ent_idx = Some((idx, i));
                if i == current_room {
                    self.logics
                        .collision
                        .handle_predicate(&CollisionReaction::RemoveEnt(
                            self.state
                                .get_col_idx(character, Some(current_room))
                                .unwrap(),
                        ));
                }
                break;
            }
        }
        let (ent_idx, room) =
            ent_idx.unwrap_or_else(|| panic!("character with id {:?} not found", character));

        let mut remove = Vec::new();
        for (idx, ((_, ent1, ent2), _)) in self.events.collision.iter_mut().enumerate() {
            if let CollisionEnt::Character(ch) = *ent1 {
                if ch == character {
                    remove.push(idx);
                }
            } else if let CollisionEnt::Character(ch) = *ent2 {
                if ch == character {
                    remove.push(idx);
                }
            }
        }
        for i in remove.iter().rev() {
            let _ = self.events.collision.remove(*i);
        }
        self.state.rooms[room].chars.remove(ent_idx);
    }

    // unsure if this is needed????
    pub(crate) fn remove_tile_at_pos(&mut self, room: usize, pos: IVec2) {
        self.state.rooms[room].map[pos.y as usize][pos.x as usize] = None;
        let current_room = self.get_current_room();

        if room == current_room {
            self.logics
                .collision
                .handle_predicate(&CollisionReaction::RemoveTileAtPos(pos));
        }

        let mut remove = Vec::new();
        for (idx, ((event_room, ent1, ent2), _)) in self.events.collision.iter_mut().enumerate() {
            if *event_room == room {
                if let CollisionEnt::Tile(tile_pos) = *ent1 {
                    if tile_pos == pos {
                        remove.push(idx);
                    }
                } else if let CollisionEnt::Tile(tile_pos) = *ent2 {
                    if tile_pos == pos {
                        remove.push(idx);
                    }
                }
            }
        }
        for i in remove.iter() {
            let _ = self.events.collision.remove(*i);
        }
    }

    #[allow(unused)]
    pub(crate) fn remove_rsrc(&mut self, pool: PoolID) {
        let ent_i = self
            .state
            .resources
            .iter()
            .position(|rid| *rid == pool.rsrc)
            .unwrap();
        self.logics.resources.items.remove(&pool);

        let mut remove = Vec::new();
        for (idx, (rsrc_event, _)) in self.events.resource_event.iter().enumerate() {
            if pool == rsrc_event.pool {
                remove.push(idx);
            }
        }
        for i in remove.into_iter() {
            let _ = self.events.resource_event.remove(i);
        }
        self.state.resources.remove(ent_i);
    }
}

impl Logics {
    pub fn consume_player(&mut self, player: Player) {
        self.collision.positions.insert(0, player.pos);
        self.collision.amt_moved.insert(0, player.amt_moved);
        self.collision
            .metadata
            .insert(0, CollisionData::new(true, false, ColEntType::Player));

        if !self.control.mapping.is_empty() {
            self.control.mapping[0].clear();
            self.control.values[0].clear();
        }

        for (act_id, keycode, valid) in player.controls {
            self.control.add_key_map(0, keycode, act_id, valid);
        }

        for (id, rsrc) in player.inventory.into_iter() {
            self.resources.items.insert(
                PoolID::new(EntID::Player, id),
                PoolValues {
                    val: rsrc,
                    min: i16::MIN,
                    max: i16::MAX,
                },
            );
        }
    }
}

pub fn load_room(state: &mut State, logics: &mut Logics, room: usize) {
    logics
        .collision
        .clear_and_resize_map(WORLD_SIZE, WORLD_SIZE);
    logics.collision.clear_entities_except(ColEntType::Player);

    for (row, col_row) in state.rooms[room]
        .map
        .iter()
        .zip(logics.collision.map.iter_mut())
    {
        for (tile, col_tile) in row.iter().zip(col_row.iter_mut()) {
            *col_tile = *tile;
        }
    }

    for (id, pos) in state.rooms[room].chars.iter() {
        logics.collision.positions.push(*pos);
        logics.collision.amt_moved.push(IVec2::ZERO);
        logics
            .collision
            .metadata
            .push(CollisionData::new(true, true, ColEntType::Character(*id)));
    }
}

pub fn set_current_room(state: &mut State, logics: &mut Logics, from_room: usize, to_room: usize) {
    for (row, col_row) in state.rooms[from_room]
        .map
        .iter_mut()
        .zip(logics.collision.map.iter())
    {
        for (tile, col_tile) in row.iter_mut().zip(col_row.iter()) {
            *tile = *col_tile;
        }
    }

    for ((_, pos), col_pos) in state.rooms[from_room]
        .chars
        .iter_mut()
        .zip(logics.collision.positions.iter().skip(1))
    {
        *pos = *col_pos;
    }

    load_room(state, logics, to_room);
}
