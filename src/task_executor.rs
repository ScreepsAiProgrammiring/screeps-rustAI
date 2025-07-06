use screeps::*;
use screeps::local::ObjectId;
use log::warn;
use crate::task_system::{Task, TaskType};

pub struct TaskExecutor;

impl TaskExecutor {
    pub fn execute_task(creep: &Creep, task: &Task) -> bool {
        // Получаем флаг "выполняю работу" из памяти крипа
        let mut mem = creep.memory();
        let is_working = js_sys::Reflect::get(&mem, &wasm_bindgen::JsValue::from_str("is_working"))
            .map(|v| v.as_bool().unwrap_or(false))
            .unwrap_or(false);

        // Проверяем, есть ли энергия
        let energy_available = creep.store().get_used_capacity(Some(ResourceType::Energy));
        let is_full = creep.store().get_free_capacity(Some(ResourceType::Energy)) == 0;

        if !is_working {
            // Флаг false - режим сбора энергии
            if is_full {
                // Бак полный - переключаемся на работу
                let _ = js_sys::Reflect::set(&mut mem, &wasm_bindgen::JsValue::from_str("is_working"), &wasm_bindgen::JsValue::from_bool(true));
                warn!("{} switching to work mode", creep.name());
                return false; // Задача продолжается, но теперь в режиме работы
            } else {
                // Бак неполный - продолжаем собирать
                warn!("{} harvesting", creep.name());
                return Self::harvest_energy_if_needed(creep);
            }
        } else {
            // Флаг true - режим выполнения задачи
            if energy_available == 0 {
                // Бак пустой - завершаем задачу
                let _ = js_sys::Reflect::set(&mut mem, &wasm_bindgen::JsValue::from_str("is_working"), &wasm_bindgen::JsValue::from_bool(false));
                warn!("{} out of energy, completing task", creep.name());
                return true; // Задача завершена
            }

            // Есть энергия - выполняем основную задачу
            warn!("{} {:?} -> {} ({})", creep.name(), task.task_type, task.target_id, energy_available);
            let task_completed = match &task.task_type {
                TaskType::TransferEnergyToSpawn => Self::execute_transfer_energy(creep, &task.target_id),
                TaskType::TransferEnergyToExtension => Self::execute_transfer_energy(creep, &task.target_id),
                TaskType::UpgradeController => Self::execute_upgrade_controller(creep, &task.target_id),
                TaskType::Build => Self::execute_build(creep, &task.target_id),
                TaskType::Repair => Self::execute_repair(creep, &task.target_id),
                TaskType::RepairWalls => Self::execute_repair(creep, &task.target_id), // Используем ту же логику ремонта
            };

            // Если задача завершена - сбрасываем флаг и завершаем
            if task_completed {
                let _ = js_sys::Reflect::set(&mut mem, &wasm_bindgen::JsValue::from_str("is_working"), &wasm_bindgen::JsValue::from_bool(false));
                warn!("{} completed {:?}", creep.name(), task.task_type);
                return true; // Задача завершена
            } else {
                return false; // Задача продолжается
            }
        }
    }

    fn harvest_energy_if_needed(creep: &Creep) -> bool {
        // Ищем ближайший источник энергии
        if let Some(room) = creep.room() {
            if let Some(source) = room.find(find::SOURCES_ACTIVE, None).first() {
                if creep.pos().is_near_to(source.pos()) {
                    match creep.harvest(source) {
                        Ok(_) => false, // Задача продолжается - продолжаем собирать
                        Err(e) => {
                            warn!("Couldn't harvest: {:?}", e);
                            true // Задача завершена с ошибкой
                        }
                    }
                } else {
                    let _ = creep.move_to(&source);
                    false // Задача продолжается - идём к источнику
                }
            } else {
                warn!("No active sources found!");
                true // Задача завершена - нет источников
            }
        } else {
            warn!("Creep has no room!");
            true // Задача завершена
        }
    }

    fn execute_transfer_energy(creep: &Creep, target_id: &str) -> bool {
        // Проверяем, есть ли энергия для передачи
        if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
            return true; // Задача завершена - нет энергии
        }

        // Пытаемся передать энергию в Spawn
        if let Ok(spawn_id) = target_id.parse::<ObjectId<StructureSpawn>>() {
            if let Some(spawn) = spawn_id.resolve() {
                // Проверяем, нужна ли энергия в спавне
                if spawn.store().get_free_capacity(Some(ResourceType::Energy)) == 0 {
                    return true; // Задача завершена - спавн полный
                }
                
                match creep.transfer(&spawn, ResourceType::Energy, None) {
                    Ok(_) => false, // Задача продолжается - продолжаем передавать
                    Err(e) => match e {
                        screeps::action_error_codes::TransferErrorCode::NotInRange => {
                            let _ = creep.move_to(&spawn);
                            false // Задача продолжается
                        }
                        _ => {
                            warn!("Couldn't transfer energy to spawn: {:?}", e);
                            true // Задача завершена с ошибкой
                        }
                    }
                }
            } else {
                warn!("Couldn't resolve spawn!");
                true // Задача завершена
            }
        } else if let Ok(extension_id) = target_id.parse::<ObjectId<StructureExtension>>() {
            if let Some(extension) = extension_id.resolve() {
                // Проверяем, нужна ли энергия в extension
                let free_capacity = extension.store().get_free_capacity(Some(ResourceType::Energy));
                if free_capacity == 0 {
                    warn!("Extension {} full", target_id);
                    return true; // Задача завершена - extension полный
                }
                
                warn!("Transfer to {} ({})", target_id, free_capacity);
                match creep.transfer(&extension, ResourceType::Energy, None) {
                    Ok(_) => {
                        warn!("Transferred to {}", target_id);
                        false // Задача продолжается - продолжаем передавать
                    }
                    Err(e) => match e {
                        screeps::action_error_codes::TransferErrorCode::NotInRange => {
                            warn!("Moving to {}", target_id);
                            let _ = creep.move_to(&extension);
                            false // Задача продолжается
                        }
                        _ => {
                            warn!("Couldn't transfer energy to extension: {:?}", e);
                            true // Задача завершена с ошибкой
                        }
                    }
                }
            } else {
                warn!("Couldn't resolve extension!");
                true // Задача завершена
            }
        } else {
            warn!("Invalid structure ID: {}", target_id);
            true // Задача завершена
        }
    }

    fn execute_upgrade_controller(creep: &Creep, target_id: &str) -> bool {
        // Проверяем, есть ли энергия для улучшения
        if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
            return true; // Задача завершена - нет энергии
        }

        // Пытаемся улучшить контроллер
        if let Ok(controller_id) = target_id.parse::<ObjectId<StructureController>>() {
            if let Some(controller) = controller_id.resolve() {
                match creep.upgrade_controller(&controller) {
                    Ok(_) => false, // Задача продолжается - продолжаем улучшать
                    Err(e) => match e {
                        screeps::action_error_codes::UpgradeControllerErrorCode::NotInRange => {
                            let _ = creep.move_to(&controller);
                            false // Задача продолжается
                        }
                        _ => {
                            warn!("Couldn't upgrade controller: {:?}", e);
                            true // Задача завершена с ошибкой
                        }
                    }
                }
            } else {
                warn!("Couldn't resolve controller!");
                true // Задача завершена
            }
        } else {
            warn!("Invalid controller ID: {}", target_id);
            true // Задача завершена
        }
    }

    fn execute_build(creep: &Creep, target_id: &str) -> bool {
        // Проверяем, есть ли энергия для строительства
        if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
            return true; // Задача завершена - нет энергии
        }

        // Пытаемся строить
        if let Ok(construction_id) = target_id.parse::<ObjectId<ConstructionSite>>() {
            if let Some(construction_site) = construction_id.resolve() {
                match creep.build(&construction_site) {
                    Ok(_) => false, // Задача продолжается - продолжаем строить
                    Err(e) => match e {
                        screeps::action_error_codes::BuildErrorCode::NotInRange => {
                            let _ = creep.move_to(&construction_site);
                            false // Задача продолжается
                        }
                        _ => {
                            warn!("Couldn't build: {:?}", e);
                            true // Задача завершена с ошибкой
                        }
                    }
                }
            } else {
                warn!("Couldn't resolve construction site!");
                true // Задача завершена
            }
        } else {
            warn!("Invalid construction site ID: {}", target_id);
            true // Задача завершена
        }
    }

    fn execute_repair(creep: &Creep, target_id: &str) -> bool {
        // Проверяем, есть ли энергия для ремонта
        if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
            return true; // Задача завершена - нет энергии
        }

        // Пытаемся ремонтировать структуру (пробуем разные типы)
        if let Ok(structure_id) = target_id.parse::<ObjectId<StructureExtension>>() {
            if let Some(structure) = structure_id.resolve() {
                return Self::repair_structure(creep, &structure);
            }
        } else if let Ok(structure_id) = target_id.parse::<ObjectId<StructureSpawn>>() {
            if let Some(structure) = structure_id.resolve() {
                return Self::repair_structure(creep, &structure);
            }
        } else if let Ok(structure_id) = target_id.parse::<ObjectId<StructureContainer>>() {
            if let Some(structure) = structure_id.resolve() {
                return Self::repair_structure(creep, &structure);
            }
        } else if let Ok(structure_id) = target_id.parse::<ObjectId<StructureRoad>>() {
            if let Some(structure) = structure_id.resolve() {
                return Self::repair_structure(creep, &structure);
            }
        } else if let Ok(structure_id) = target_id.parse::<ObjectId<StructureWall>>() {
            if let Some(structure) = structure_id.resolve() {
                return Self::repair_structure(creep, &structure);
            }
        } else if let Ok(structure_id) = target_id.parse::<ObjectId<StructureRampart>>() {
            if let Some(structure) = structure_id.resolve() {
                return Self::repair_structure(creep, &structure);
            }
        }

        warn!("Invalid structure ID: {}", target_id);
        true // Задача завершена
    }

    fn repair_structure<T: Repairable + HasPosition + AsRef<RoomObject>>(creep: &Creep, structure: &T) -> bool {
        // Проверяем, нужен ли ремонт
        if structure.hits() >= structure.hits_max() {
            return true; // Задача завершена - структура здорова
        }

        match creep.repair(structure) {
            Ok(_) => false, // Задача продолжается - продолжаем чинить
            Err(e) => match e {
                screeps::action_error_codes::CreepRepairErrorCode::NotInRange => {
                    let _ = creep.move_to(structure);
                    false // Задача продолжается
                }
                _ => {
                    warn!("Couldn't repair: {:?}", e);
                    true // Задача завершена с ошибкой
                }
            }
        }
    }
} 