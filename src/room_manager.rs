use screeps::*;
use std::collections::HashMap;
use log::info;
use crate::command_system::ActionBoard;

pub struct RoomManager {
    pub action_boards: HashMap<RoomName, ActionBoard>,
}

impl RoomManager {
    pub fn new() -> Self {
        Self {
            action_boards: HashMap::new(),
        }
    }

    pub fn initialize_room(&mut self, room_name: &RoomName) {
        if !self.action_boards.contains_key(room_name) {
            let action_board = ActionBoard::new(*room_name);
            self.action_boards.insert(*room_name, action_board);
            info!("Initialized action board for room {}", room_name);
        }
    }

    pub fn get_action_board(&mut self, room_name: &RoomName) -> &mut ActionBoard {
        self.initialize_room(room_name);
        self.action_boards.get_mut(room_name).unwrap()
    }

    pub fn run_rooms(&mut self) {
        for spawn in game::spawns().values() {
            if let Some(room) = spawn.room() {
                self.initialize_room(&room.name());
            }
        }
    }

    pub fn cleanup_empty_rooms(&mut self) {
        let mut to_remove = Vec::new();
        for room_name in self.action_boards.keys() {
            let has_spawns = game::spawns().values().any(|spawn| {
                spawn.room().map(|room| room.name() == *room_name).unwrap_or(false)
            });
            if !has_spawns {
                to_remove.push(room_name.clone());
            }
        }
        for room_name in to_remove {
            self.action_boards.remove(&room_name);
            info!("Removed action board for room {} (no spawns)", room_name);
        }
    }
} 