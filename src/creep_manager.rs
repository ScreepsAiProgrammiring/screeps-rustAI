use screeps::*;
use std::collections::HashMap;
use crate::task_system::{TaskBoard, Task, TaskType};
use crate::task_executor::TaskExecutor;

pub struct CreepManager {
    pub assigned_tasks: HashMap<String, Task>, // creep_name -> task
}

impl CreepManager {
    pub fn new() -> Self {
        Self {
            assigned_tasks: HashMap::new(),
        }
    }

    pub fn run_creep(&mut self, creep: &Creep, task_board: &mut TaskBoard) {
        if creep.spawning() {
            return;
        }

        let creep_name = creep.name();
        
        // Проверяем, есть ли у крипа назначенная задача
        if let Some(task) = self.assigned_tasks.get_mut(&creep_name) {
            // Выполняем задачу
            let task_completed = TaskExecutor::execute_task(creep, task);
            
            if task_completed {
                // Задача завершена, освобождаем крипа
                self.assigned_tasks.remove(&creep_name);
            }
        }

        // Если у крипа нет задачи, генерируем новую
        if !self.assigned_tasks.contains_key(&creep_name) {
            self.generate_new_task(creep, task_board);
        }
    }

    fn generate_new_task(&mut self, creep: &Creep, task_board: &mut TaskBoard) {
        let creep_name = creep.name();
        
        // Генерируем новую задачу с использованием взвешенного рандома
        if let Some(task) = Self::generate_weighted_task(task_board) {
            // Записываем описание задачи в память крипа
            let description = match task.task_type {
                TaskType::TransferEnergyToSpawn => format!("Refilling spawn [{}]", &task.target_id[..5]),
                TaskType::TransferEnergyToExtension => format!("Refilling extension [{}]", &task.target_id[..5]),
                TaskType::UpgradeController => "Upgrading controller".to_string(),
                TaskType::Build => format!("Building site [{}]", &task.target_id[..5]),
                TaskType::Repair => format!("Repairing structure [{}]", &task.target_id[..5]),
                TaskType::RepairWalls => format!("Repairing wall [{}]", &task.target_id[..5]),
            };
            let mut mem = creep.memory();
            let _ = js_sys::Reflect::set(&mut mem, &wasm_bindgen::JsValue::from_str("current_task"), &wasm_bindgen::JsValue::from_str(&description));
            self.assigned_tasks.insert(creep_name, task);
        }
    }

    fn generate_weighted_task(task_board: &TaskBoard) -> Option<Task> {
        if let Some(room) = game::rooms().get(task_board.room_name) {
            // Определяем доступные типы задач и их веса
            let mut available_tasks: Vec<(TaskType, String, f64)> = Vec::new();
            
            // Проверяем возможность передачи энергии в Spawn
            for structure in room.find(find::STRUCTURES, None).iter() {
                if let screeps::enums::StructureObject::StructureSpawn(spawn) = structure {
                    if spawn.store().get_free_capacity(Some(ResourceType::Energy)) > 0 {
                        available_tasks.push((
                            TaskType::TransferEnergyToSpawn,
                            spawn.id().to_string(),
                            Self::get_task_weight(TaskType::TransferEnergyToSpawn)
                        ));
                        break; // Только один Spawn
                    }
                }
            }

            // Проверяем возможность передачи энергии в Extension
            for structure in room.find(find::STRUCTURES, None).iter() {
                if let screeps::enums::StructureObject::StructureExtension(extension) = structure {
                    if extension.store().get_free_capacity(Some(ResourceType::Energy)) > 0 {
                        available_tasks.push((
                            TaskType::TransferEnergyToExtension,
                            extension.id().to_string(),
                            Self::get_task_weight(TaskType::TransferEnergyToExtension)
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
                        available_tasks.push((
                            TaskType::UpgradeController,
                            controller.id().to_string(),
                            Self::get_task_weight(TaskType::UpgradeController)
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
                available_tasks.push((
                    TaskType::Build,
                    "construction_sites".to_string(), // Заглушка, реальная стройплощадка выберется позже
                    Self::get_task_weight(TaskType::Build)
                ));
            }

            // Проверяем возможность ремонта структур (кроме стен)
            let mut most_damaged_structure: Option<(screeps::enums::StructureObject, f64)> = None;
            
            for structure in room.find(find::STRUCTURES, None).iter() {
                let structure_ref = structure.as_structure();
                if structure_ref.structure_type() != StructureType::Controller 
                   && structure_ref.structure_type() != StructureType::Wall
                   && structure_ref.structure_type() != StructureType::Rampart {
                    let hits_ratio = structure_ref.hits() as f64 / structure_ref.hits_max() as f64;
                    if hits_ratio < 0.5 {
                        if let Some((_, current_ratio)) = most_damaged_structure {
                            if hits_ratio < current_ratio {
                                most_damaged_structure = Some((structure.clone(), hits_ratio));
                            }
                        } else {
                            most_damaged_structure = Some((structure.clone(), hits_ratio));
                        }
                    }
                }
            }
            
            if let Some((structure, _damage_ratio)) = most_damaged_structure {
                available_tasks.push((
                    TaskType::Repair,
                    "damaged_structure".to_string(), // Заглушка, реальная структура выберется позже
                    Self::get_task_weight(TaskType::Repair)
                ));
            }

            // Проверяем возможность ремонта стен
            let mut most_damaged_wall: Option<(screeps::enums::StructureObject, f64)> = None;
            
            for structure in room.find(find::STRUCTURES, None).iter() {
                let structure_ref = structure.as_structure();
                if structure_ref.structure_type() == StructureType::Wall 
                   || structure_ref.structure_type() == StructureType::Rampart {
                    let hits_ratio = structure_ref.hits() as f64 / structure_ref.hits_max() as f64;
                    if hits_ratio < 0.5 {
                        if let Some((_, current_ratio)) = most_damaged_wall {
                            if hits_ratio < current_ratio {
                                most_damaged_wall = Some((structure.clone(), hits_ratio));
                            }
                        } else {
                            most_damaged_wall = Some((structure.clone(), hits_ratio));
                        }
                    }
                }
            }
            
            if let Some((wall, _damage_ratio)) = most_damaged_wall {
                available_tasks.push((
                    TaskType::RepairWalls,
                    "damaged_wall".to_string(), // Заглушка, реальная стена выберется позже
                    Self::get_task_weight(TaskType::RepairWalls)
                ));
            }

            // Выбираем задачу с использованием правильного взвешенного рандома
            if !available_tasks.is_empty() {
                let total_weight: f64 = available_tasks.iter().map(|(_, _, weight)| weight).sum();
                
                if total_weight > 0.0 {
                    // Используем Math.random() для правильного рандома
                    let random_value = js_sys::Math::random() * total_weight;
                    let mut current_weight = 0.0;
                    
                    for (task_type, target_id, weight) in &available_tasks {
                        current_weight += weight;
                        if random_value <= current_weight {
                            // Выбираем конкретную цель для задач, которые требуют дополнительного выбора
                            let final_target_id = match task_type {
                                TaskType::Build => Self::select_random_construction_site(&room),
                                TaskType::Repair => Self::select_most_damaged_structure(&room),
                                TaskType::RepairWalls => Self::select_most_damaged_wall(&room),
                                _ => Some(target_id.clone()),
                            };
                            
                            if let Some(final_id) = final_target_id {
                                return Some(Task::new(task_type.clone(), final_id));
                            }
                        }
                    }
                }
                
                // Если что-то пошло не так, возвращаем первую доступную задачу
                let (task_type, target_id, _weight) = &available_tasks[0];
                let final_target_id = match task_type {
                    TaskType::Build => Self::select_random_construction_site(&room),
                    TaskType::Repair => Self::select_most_damaged_structure(&room),
                    TaskType::RepairWalls => Self::select_most_damaged_wall(&room),
                    _ => Some(target_id.clone()),
                };
                
                if let Some(final_id) = final_target_id {
                    return Some(Task::new(task_type.clone(), final_id));
                }
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

    /// Выбирает самую повреждённую структуру (кроме стен)
    fn select_most_damaged_structure(room: &Room) -> Option<String> {
        let mut most_damaged_structure: Option<(String, f64)> = None;
        
        for structure in room.find(find::STRUCTURES, None).iter() {
            let structure_ref = structure.as_structure();
            if structure_ref.structure_type() != StructureType::Controller 
               && structure_ref.structure_type() != StructureType::Wall
               && structure_ref.structure_type() != StructureType::Rampart {
                let hits_ratio = structure_ref.hits() as f64 / structure_ref.hits_max() as f64;
                if hits_ratio < 0.5 {
                    if let Some((_, current_ratio)) = most_damaged_structure {
                        if hits_ratio < current_ratio {
                            most_damaged_structure = Some((structure_ref.id().to_string(), hits_ratio));
                        }
                    } else {
                        most_damaged_structure = Some((structure_ref.id().to_string(), hits_ratio));
                    }
                }
            }
        }
        
        most_damaged_structure.map(|(id, _)| id)
    }

    /// Выбирает самую повреждённую стену
    fn select_most_damaged_wall(room: &Room) -> Option<String> {
        let mut most_damaged_wall: Option<(String, f64)> = None;
        
        for structure in room.find(find::STRUCTURES, None).iter() {
            let structure_ref = structure.as_structure();
            if structure_ref.structure_type() == StructureType::Wall 
               || structure_ref.structure_type() == StructureType::Rampart {
                let hits_ratio = structure_ref.hits() as f64 / structure_ref.hits_max() as f64;
                if hits_ratio < 0.5 {
                    if let Some((_, current_ratio)) = most_damaged_wall {
                        if hits_ratio < current_ratio {
                            most_damaged_wall = Some((structure_ref.id().to_string(), hits_ratio));
                        }
                    } else {
                        most_damaged_wall = Some((structure_ref.id().to_string(), hits_ratio));
                    }
                }
            }
        }
        
        most_damaged_wall.map(|(id, _)| id)
    }

    /// Возвращает базовый вес для каждого типа задачи
    /// Чем выше вес, тем выше вероятность выбора этой задачи
    fn get_task_weight(task_type: TaskType) -> f64 {
        match task_type {
            TaskType::TransferEnergyToSpawn => 1.0,    // Высший приоритет - Spawn должен быть заполнен
            TaskType::TransferEnergyToExtension => 0.8, // Средний приоритет
            TaskType::UpgradeController => 0.7,         // Средний приоритет
            TaskType::Build => 0.6,                     // Низкий приоритет
            TaskType::Repair => 0.4,                    // Ремонт структур
            TaskType::RepairWalls => 0.3,               // Ремонт стен (самый низкий приоритет)
        }
    }

    pub fn cleanup_dead_creeps(&mut self) {
        let mut to_remove = Vec::new();
        
        for creep_name in self.assigned_tasks.keys() {
            if !game::creeps().keys().any(|name| name == *creep_name) {
                to_remove.push(creep_name.clone());
            }
        }
        
        for creep_name in to_remove {
            self.assigned_tasks.remove(&creep_name);
        }
    }
} 