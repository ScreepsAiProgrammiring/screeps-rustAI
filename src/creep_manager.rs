use screeps::*;
use std::collections::HashMap;
use log::warn;
use crate::command_system::{ActionBoard, ActionQueue, ActionQueueFactory};

pub struct CreepManager {
    pub assigned_queues: HashMap<String, ActionQueue>, // creep_name -> action_queue
}

impl CreepManager {
    pub fn new() -> Self {
        Self {
            assigned_queues: HashMap::new(),
        }
    }

    pub fn run_creep(&mut self, creep: &Creep, action_board: &mut ActionBoard) {
        if creep.spawning() {
            return;
        }

        let creep_name = creep.name();
        
        // Проверяем, есть ли у крипа назначенная последовательность команд
        if let Some(queue) = self.assigned_queues.get_mut(&creep_name) {
            // Выполняем текущую команду в последовательности
            if let Some(action) = queue.current_action() {
                let action_completed = action.execute(creep);
                
                if action_completed {
                    // Команда завершена, переходим к следующей
                    let has_more_actions = queue.next_action();
                    
                    if !has_more_actions {
                        // Последовательность завершена, освобождаем крипа
                        self.assigned_queues.remove(&creep_name);
                        warn!("{} completed action queue", creep_name);
                    } else {
                        // Обновляем описание текущей команды
                        if let Some(next_action) = queue.current_action() {
                            let description = next_action.get_description();
                            let mut mem = creep.memory();
                            let _ = js_sys::Reflect::set(&mut mem, &wasm_bindgen::JsValue::from_str("current_action"), &wasm_bindgen::JsValue::from_str(&description));
                        }
                    }
                }
            }
        }

        // Если у крипа нет последовательности команд, генерируем новую
        if !self.assigned_queues.contains_key(&creep_name) {
            self.generate_new_queue(creep, action_board);
        }
    }



    fn generate_new_queue(&mut self, creep: &Creep, action_board: &mut ActionBoard) {
        let creep_name = creep.name();
        
        // Генерируем новую последовательность команд с использованием взвешенного рандома
        if let Some(queue) = Self::generate_weighted_queue(action_board) {
            // Записываем описание первой команды в память крипа
            if let Some(first_action) = queue.current_action() {
                let description = first_action.get_description();
                let mut mem = creep.memory();
                let _ = js_sys::Reflect::set(&mut mem, &wasm_bindgen::JsValue::from_str("current_action"), &wasm_bindgen::JsValue::from_str(&description));
            }
            self.assigned_queues.insert(creep_name, queue);
        }
    }

    fn generate_weighted_queue(action_board: &ActionBoard) -> Option<ActionQueue> {
        if let Some(room) = game::rooms().get(action_board.room_name) {
            // Определяем доступные типы последовательностей и их веса
            let mut available_queues: Vec<(String, f64)> = Vec::new();
            
            // Проверяем возможность передачи энергии в Spawn
            for structure in room.find(find::STRUCTURES, None).iter() {
                if let screeps::enums::StructureObject::StructureSpawn(spawn) = structure {
                    if spawn.store().get_free_capacity(Some(ResourceType::Energy)) > 0 {
                        available_queues.push((
                            spawn.id().to_string(),
                            Self::get_queue_weight("transfer_energy_to_spawn")
                        ));
                        break; // Только один Spawn
                    }
                }
            }

            // Проверяем возможность передачи энергии в Extension
            for structure in room.find(find::STRUCTURES, None).iter() {
                if let screeps::enums::StructureObject::StructureExtension(extension) = structure {
                    if extension.store().get_free_capacity(Some(ResourceType::Energy)) > 0 {
                        available_queues.push((
                            extension.id().to_string(),
                            Self::get_queue_weight("transfer_energy_to_extension")
                        ));
                    }
                }
            }

            // Проверяем возможность передачи энергии в Tower
            for structure in room.find(find::STRUCTURES, None).iter() {
                if let screeps::enums::StructureObject::StructureTower(tower) = structure {
                    if tower.store().get_free_capacity(Some(ResourceType::Energy)) > 0 {
                        available_queues.push((
                            tower.id().to_string(),
                            Self::get_queue_weight("transfer_energy_to_tower")
                        ));
                    }
                }
            }

            // Проверяем возможность улучшения контроллера
            let has_energy_storage = room.find(find::STRUCTURES, None).iter().any(|s| {
                match s {
                    screeps::enums::StructureObject::StructureSpawn(spawn) => {
                        spawn.store().get_used_capacity(Some(ResourceType::Energy)) > 0
                    }
                    screeps::enums::StructureObject::StructureExtension(ext) => {
                        ext.store().get_used_capacity(Some(ResourceType::Energy)) > 0
                    }
                    _ => false
                }
            });
            
            if has_energy_storage {
                for structure in room.find(find::STRUCTURES, None).iter() {
                    if let screeps::enums::StructureObject::StructureController(controller) = structure {
                        available_queues.push((
                            controller.id().to_string(),
                            Self::get_queue_weight("upgrade_controller")
                        ));
                        break; // Только один контроллер
                    }
                }
            }

            // Проверяем возможность строительства
            let construction_sites: Vec<String> = room.find(find::CONSTRUCTION_SITES, None).iter()
                .filter_map(|site| site.try_id().map(|id| id.to_string()))
                .collect();
            
            if !construction_sites.is_empty() {
                if let Some(site_id) = Self::select_random_construction_site(&room) {
                    available_queues.push((
                        site_id,
                        Self::get_queue_weight("build")
                    ));
                }
            }

            // Проверяем возможность ремонта структур (кроме стен)
            if let Some(structure_id) = Self::select_most_damaged_structure(&room) {
                available_queues.push((
                    structure_id,
                    Self::get_queue_weight("repair")
                ));
            }

            // Проверяем возможность ремонта стен
            if let Some(wall_id) = Self::select_most_damaged_wall(&room) {
                available_queues.push((
                    wall_id,
                    Self::get_queue_weight("repair_walls")
                ));
            }

            // Выбираем последовательность с использованием взвешенного рандома
            if !available_queues.is_empty() {
                let total_weight: f64 = available_queues.iter().map(|(_, weight)| weight).sum();
                
                if total_weight > 0.0 {
                    let random_value = js_sys::Math::random() * total_weight;
                    let mut current_weight = 0.0;
                    
                    for (target_id, weight) in &available_queues {
                        current_weight += weight;
                        if random_value <= current_weight {
                            // Определяем тип последовательности по весу
                            let queue_type = Self::get_queue_type_by_weight(*weight);
                            return Self::create_queue_by_type(queue_type, target_id.clone());
                        }
                    }
                }
                
                // Если что-то пошло не так, возвращаем первую доступную последовательность
                let (target_id, weight) = &available_queues[0];
                let queue_type = Self::get_queue_type_by_weight(*weight);
                return Self::create_queue_by_type(queue_type, target_id.clone());
            }
        }
        
        None
    }

    /// Выбирает случайную стройплощадку
    fn select_random_construction_site(room: &Room) -> Option<String> {
        let construction_sites: Vec<String> = room.find(find::CONSTRUCTION_SITES, None).iter()
            .filter_map(|site| site.try_id().map(|id| id.to_string()))
            .collect();
        
        if construction_sites.is_empty() {
            None
        } else {
            let random_index = (js_sys::Math::random() * construction_sites.len() as f64) as usize;
            Some(construction_sites[random_index].clone())
        }
    }

    /// Выбирает случайную повреждённую структуру (кроме стен)
    fn select_most_damaged_structure(room: &Room) -> Option<String> {
        let mut damaged_structures: Vec<(String, f64)> = Vec::new();
        
        for structure in room.find(find::STRUCTURES, None).iter() {
            let structure_ref = structure.as_structure();
            if structure_ref.structure_type() != StructureType::Controller 
               && structure_ref.structure_type() != StructureType::Wall
               && structure_ref.structure_type() != StructureType::Rampart {
                let hits = structure_ref.hits();
                let hits_max = structure_ref.hits_max();
                
                // Исключаем непостроенные и полностью здоровые структуры
                if hits > 0 && hits < hits_max {
                    let hits_ratio = hits as f64 / hits_max as f64;
                    if hits_ratio < 0.5 {
                        damaged_structures.push((structure_ref.id().to_string(), hits_ratio));
                    }
                }
            }
        }
        
        if damaged_structures.is_empty() {
            None
        } else {
            // Выбираем случайную повреждённую структуру
            let random_index = (js_sys::Math::random() * damaged_structures.len() as f64) as usize;
            Some(damaged_structures[random_index].0.clone())
        }
    }

    /// Выбирает случайную повреждённую стену
    fn select_most_damaged_wall(room: &Room) -> Option<String> {
        let mut damaged_walls: Vec<(String, f64)> = Vec::new();
        
        for structure in room.find(find::STRUCTURES, None).iter() {
            let structure_ref = structure.as_structure();
            if structure_ref.structure_type() == StructureType::Wall 
               || structure_ref.structure_type() == StructureType::Rampart {
                let hits = structure_ref.hits();
                let hits_max = structure_ref.hits_max();
                
                // Исключаем стены с 0 хитами (непостроенные) и полностью здоровые
                if hits > 0 && hits < hits_max {
                    let hits_ratio = hits as f64 / hits_max as f64;
                    if hits_ratio < 0.5 {
                        damaged_walls.push((structure_ref.id().to_string(), hits_ratio));
                    }
                }
            }
        }
        
        if damaged_walls.is_empty() {
            None
        } else {
            // Выбираем случайную повреждённую стену
            let random_index = (js_sys::Math::random() * damaged_walls.len() as f64) as usize;
            Some(damaged_walls[random_index].0.clone())
        }
    }

    /// Возвращает базовый вес для каждого типа последовательности
    /// Чем выше вес, тем выше вероятность выбора этой последовательности
    fn get_queue_weight(queue_type: &str) -> f64 {
        match queue_type {
            "transfer_energy_to_spawn" => 0.5,    // Высший приоритет - Spawn должен быть заполнен
            "transfer_energy_to_extension" => 0.7, // Средний приоритет
            "transfer_energy_to_tower" => 0.6,     // Высокий приоритет - башни важны для защиты
            "upgrade_controller" => 1.0,           // Средний приоритет
            "build" => 0.7,                        // Низкий приоритет
            "repair" => 0.4,                       // Ремонт структур
            "repair_walls" => 0.4,                 // Ремонт стен (самый низкий приоритет)
            _ => 0.5,                              // По умолчанию
        }
    }

    /// Определяет тип последовательности по весу
    fn get_queue_type_by_weight(weight: f64) -> &'static str {
        if (weight - 0.5).abs() < 0.01 { "transfer_energy_to_spawn" }
        else if (weight - 0.7).abs() < 0.01 { "transfer_energy_to_extension" }
        else if (weight - 0.6).abs() < 0.01 { "transfer_energy_to_tower" }
        else if (weight - 1.0).abs() < 0.01 { "upgrade_controller" }
        else if (weight - 0.7).abs() < 0.01 { "build" }
        else if (weight - 0.4).abs() < 0.01 { "repair" }
        else { "transfer_energy_to_spawn" } // По умолчанию
    }

    /// Создаёт последовательность команд по типу
    fn create_queue_by_type(queue_type: &str, target_id: String) -> Option<ActionQueue> {
        match queue_type {
            "transfer_energy_to_spawn" => Some(ActionQueueFactory::transfer_energy_sequence(target_id)),
            "transfer_energy_to_extension" => Some(ActionQueueFactory::transfer_energy_to_extension_sequence(target_id)),
            "transfer_energy_to_tower" => Some(ActionQueueFactory::transfer_energy_to_tower_sequence(target_id)),
            "upgrade_controller" => Some(ActionQueueFactory::upgrade_controller_sequence(target_id)),
            "build" => Some(ActionQueueFactory::build_sequence(target_id)),
            "repair" => Some(ActionQueueFactory::repair_sequence(target_id)),
            "repair_walls" => Some(ActionQueueFactory::repair_walls_sequence(target_id)),
            _ => None,
        }
    }

    pub fn cleanup_dead_creeps(&mut self) {
        let mut to_remove = Vec::new();
        
        for creep_name in self.assigned_queues.keys() {
            if !game::creeps().keys().any(|name| name == *creep_name) {
                to_remove.push(creep_name.clone());
            }
        }
        
        for creep_name in to_remove {
            self.assigned_queues.remove(&creep_name);
        }
    }
} 