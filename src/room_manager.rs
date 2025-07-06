use screeps::*;
use std::collections::HashMap;
use log::info;
use crate::task_system::TaskBoard;

pub struct RoomManager {
    pub task_boards: HashMap<RoomName, TaskBoard>,
}

impl RoomManager {
    pub fn new() -> Self {
        Self {
            task_boards: HashMap::new(),
        }
    }

    pub fn initialize_room(&mut self, room_name: &RoomName) {
        if !self.task_boards.contains_key(room_name) {
            let task_board = TaskBoard::new(*room_name);
            self.task_boards.insert(*room_name, task_board);
            info!("Initialized task board for room {}", room_name);
        }
    }

    pub fn get_task_board(&mut self, room_name: &RoomName) -> &mut TaskBoard {
        self.initialize_room(room_name);
        self.task_boards.get_mut(room_name).unwrap()
    }

    pub fn run_rooms(&mut self) {
        // Инициализируем доски задач для всех комнат с spawn'ами
        for spawn in game::spawns().values() {
            if let Some(room) = spawn.room() {
                self.initialize_room(&room.name());
            }
        }
    }

    pub fn cleanup_empty_rooms(&mut self) {
        let mut to_remove = Vec::new();
        
        for room_name in self.task_boards.keys() {
            // Проверяем, есть ли в комнате spawn'ы
            let has_spawns = game::spawns().values().any(|spawn| {
                spawn.room().map(|room| room.name() == *room_name).unwrap_or(false)
            });
            
            if !has_spawns {
                to_remove.push(room_name.clone());
            }
        }
        
        for room_name in to_remove {
            self.task_boards.remove(&room_name);
            info!("Removed task board for room {} (no spawns)", room_name);
        }
    }
} 