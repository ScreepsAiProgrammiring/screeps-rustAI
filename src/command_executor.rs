use screeps::*;
use screeps::local::ObjectId;
use log::warn;
use crate::command_system::{Action, ActionType};

pub struct ActionExecutor;

impl ActionExecutor {
    pub fn execute_action(creep: &Creep, action: &Action) -> bool {
        match &action.action_type {
            ActionType::HarvestEnergy => Self::execute_harvest_energy(creep),
            ActionType::TransferEnergyToSpawn => Self::execute_transfer_energy_to_spawn(creep, action),
            ActionType::TransferEnergyToExtension => Self::execute_transfer_energy_to_extension(creep, action),
            ActionType::TransferEnergyToTower => Self::execute_transfer_energy_to_tower(creep, action),
            ActionType::UpgradeController => Self::execute_upgrade_controller(creep, action),
            ActionType::Build => Self::execute_build(creep, action),
            ActionType::Repair => Self::execute_repair(creep, action),
            ActionType::RepairWalls => Self::execute_repair(creep, action), // Используем ту же логику
            ActionType::MoveTo => Self::execute_move_to(creep, action),
            ActionType::Wait => Self::execute_wait(creep, action),
        }
    }

    fn execute_harvest_energy(creep: &Creep) -> bool {
        // Проверяем, есть ли место для энергии
        if creep.store().get_free_capacity(Some(ResourceType::Energy)) == 0 {
            return true; // Бак полный - команда завершена
        }

        // Ищем ближайший источник энергии
        if let Some(room) = creep.room() {
            if let Some(source) = room.find(find::SOURCES_ACTIVE, None).first() {
                if creep.pos().is_near_to(source.pos()) {
                    match creep.harvest(source) {
                        Ok(_) => false, // Команда продолжается - продолжаем собирать
                        Err(e) => {
                            warn!("Couldn't harvest: {:?}", e);
                            true // Команда завершена с ошибкой
                        }
                    }
                } else {
                    let _ = creep.move_to(&source);
                    false // Команда продолжается - идём к источнику
                }
            } else {
                warn!("No active sources found!");
                true // Команда завершена - нет источников
            }
        } else {
            warn!("Creep has no room!");
            true // Команда завершена
        }
    }

    fn execute_transfer_energy_to_spawn(creep: &Creep, action: &Action) -> bool {
        Self::execute_transfer_energy_generic(creep, action, |target_id| {
            target_id.parse::<ObjectId<StructureSpawn>>()
        })
    }

    fn execute_transfer_energy_to_extension(creep: &Creep, action: &Action) -> bool {
        Self::execute_transfer_energy_generic(creep, action, |target_id| {
            target_id.parse::<ObjectId<StructureExtension>>()
        })
    }

    fn execute_transfer_energy_to_tower(creep: &Creep, action: &Action) -> bool {
        Self::execute_transfer_energy_generic(creep, action, |target_id| {
            target_id.parse::<ObjectId<StructureTower>>()
        })
    }

    fn execute_transfer_energy_generic<T, F>(creep: &Creep, action: &Action, parse_fn: F) -> bool 
    where 
        F: Fn(&str) -> Result<ObjectId<T>, screeps::local::RawObjectIdParseError>,
        T: HasStore + HasPosition + AsRef<RoomObject> + wasm_bindgen::JsCast + screeps::MaybeHasId + screeps::Transferable,
    {
        // Проверяем, есть ли энергия для передачи
        if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
            return true; // Команда завершена - нет энергии
        }

        // Получаем ID цели
        let target_id = match &action.target_id {
            Some(id) => id,
            None => {
                warn!("No target ID provided for transfer command");
                return true; // Команда завершена с ошибкой
            }
        };

        // Пытаемся передать энергию
        if let Ok(structure_id) = parse_fn(target_id) {
            if let Some(structure) = structure_id.resolve() {
                // Проверяем, нужна ли энергия в структуре
                if structure.store().get_free_capacity(Some(ResourceType::Energy)) == 0 {
                    return true; // Команда завершена - структура полная
                }
                
                match creep.transfer(&structure, ResourceType::Energy, None) {
                    Ok(_) => false, // Команда продолжается - продолжаем передавать
                    Err(e) => match e {
                        screeps::action_error_codes::TransferErrorCode::NotInRange => {
                            let _ = creep.move_to(&structure);
                            false // Команда продолжается
                        }
                        _ => {
                            warn!("Couldn't transfer energy: {:?}", e);
                            true // Команда завершена с ошибкой
                        }
                    }
                }
            } else {
                warn!("Couldn't resolve structure!");
                true // Команда завершена
            }
        } else {
            warn!("Invalid structure ID: {}", target_id);
            true // Команда завершена
        }
    }

    fn execute_upgrade_controller(creep: &Creep, action: &Action) -> bool {
        // Проверяем, есть ли энергия для улучшения
        if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
            return true; // Команда завершена - нет энергии
        }

        // Получаем ID цели
        let target_id = match &action.target_id {
            Some(id) => id,
            None => {
                warn!("No target ID provided for upgrade command");
                return true; // Команда завершена с ошибкой
            }
        };

        // Пытаемся улучшить контроллер
        if let Ok(controller_id) = target_id.parse::<ObjectId<StructureController>>() {
            if let Some(controller) = controller_id.resolve() {
                match creep.upgrade_controller(&controller) {
                    Ok(_) => false, // Команда продолжается - продолжаем улучшать
                    Err(e) => match e {
                        screeps::action_error_codes::UpgradeControllerErrorCode::NotInRange => {
                            let _ = creep.move_to(&controller);
                            false // Команда продолжается
                        }
                        _ => {
                            warn!("Couldn't upgrade controller: {:?}", e);
                            true // Команда завершена с ошибкой
                        }
                    }
                }
            } else {
                warn!("Couldn't resolve controller!");
                true // Команда завершена
            }
        } else {
            warn!("Invalid controller ID: {}", target_id);
            true // Команда завершена
        }
    }

    fn execute_build(creep: &Creep, action: &Action) -> bool {
        // Проверяем, есть ли энергия для строительства
        if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
            return true; // Команда завершена - нет энергии
        }

        // Получаем ID цели
        let target_id = match &action.target_id {
            Some(id) => id,
            None => {
                warn!("No target ID provided for build command");
                return true; // Команда завершена с ошибкой
            }
        };

        // Пытаемся строить
        if let Ok(construction_id) = target_id.parse::<ObjectId<ConstructionSite>>() {
            if let Some(construction_site) = construction_id.resolve() {
                match creep.build(&construction_site) {
                    Ok(_) => false, // Команда продолжается - продолжаем строить
                    Err(e) => match e {
                        screeps::action_error_codes::BuildErrorCode::NotInRange => {
                            let _ = creep.move_to(&construction_site);
                            false // Команда продолжается
                        }
                        _ => {
                            warn!("Couldn't build: {:?}", e);
                            true // Команда завершена с ошибкой
                        }
                    }
                }
            } else {
                warn!("Couldn't resolve construction site!");
                true // Команда завершена
            }
        } else {
            warn!("Invalid construction site ID: {}", target_id);
            true // Команда завершена
        }
    }

    fn execute_repair(creep: &Creep, action: &Action) -> bool {
        // Проверяем, есть ли энергия для ремонта
        if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
            return true; // Команда завершена - нет энергии
        }

        // Получаем ID цели
        let target_id = match &action.target_id {
            Some(id) => id,
            None => {
                warn!("No target ID provided for repair command");
                return true; // Команда завершена с ошибкой
            }
        };

        warn!("{} attempting to repair structure: {}", creep.name(), target_id);

        // Пытаемся ремонтировать структуру (пробуем разные типы)
        if let Ok(structure_id) = target_id.parse::<ObjectId<StructureExtension>>() {
            if let Some(structure) = structure_id.resolve() {
                warn!("{} repairing Extension", creep.name());
                return Self::repair_structure(creep, &structure);
            }
        } else if let Ok(structure_id) = target_id.parse::<ObjectId<StructureSpawn>>() {
            if let Some(structure) = structure_id.resolve() {
                warn!("{} repairing Spawn", creep.name());
                return Self::repair_structure(creep, &structure);
            }
        } else if let Ok(structure_id) = target_id.parse::<ObjectId<StructureContainer>>() {
            if let Some(structure) = structure_id.resolve() {
                warn!("{} repairing Container", creep.name());
                return Self::repair_structure(creep, &structure);
            }
        } else if let Ok(structure_id) = target_id.parse::<ObjectId<StructureRoad>>() {
            if let Some(structure) = structure_id.resolve() {
                warn!("{} repairing Road", creep.name());
                return Self::repair_structure(creep, &structure);
            }
        } else if let Ok(structure_id) = target_id.parse::<ObjectId<StructureWall>>() {
            if let Some(structure) = structure_id.resolve() {
                warn!("{} repairing Wall", creep.name());
                return Self::repair_structure(creep, &structure);
            }
        } else if let Ok(structure_id) = target_id.parse::<ObjectId<StructureRampart>>() {
            if let Some(structure) = structure_id.resolve() {
                warn!("{} repairing Rampart", creep.name());
                return Self::repair_structure(creep, &structure);
            }
        }

        warn!("{} invalid structure ID: {}", creep.name(), target_id);
        true // Команда завершена
    }

    fn repair_structure<T: Repairable + HasPosition + AsRef<RoomObject>>(creep: &Creep, structure: &T) -> bool {
        // Проверяем, нужен ли ремонт
        let hits = structure.hits();
        let hits_max = structure.hits_max();
        let hits_ratio = hits as f64 / hits_max as f64;
        
        warn!("{} repairing structure: hits={}/{}, ratio={:.2}", creep.name(), hits, hits_max, hits_ratio);
        
        if hits >= hits_max {
            warn!("{} structure is healthy, completing repair", creep.name());
            return true; // Команда завершена - структура здорова
        }

        match creep.repair(structure) {
            Ok(_) => {
                warn!("{} repairing successfully", creep.name());
                false // Команда продолжается - продолжаем чинить
            }
            Err(e) => match e {
                screeps::action_error_codes::CreepRepairErrorCode::NotInRange => {
                    warn!("{} moving to structure for repair", creep.name());
                    let _ = creep.move_to(structure);
                    false // Команда продолжается
                }
                _ => {
                    warn!("{} couldn't repair: {:?}", creep.name(), e);
                    true // Команда завершена с ошибкой
                }
            }
        }
    }

    fn execute_move_to(creep: &Creep, action: &Action) -> bool {
        if let Some(pos) = &action.parameters.move_to_pos {
            let _ = creep.move_to(*pos);
            false // Команда продолжается
        } else {
            warn!("No position provided for move_to command");
            true // Команда завершена с ошибкой
        }
    }

    fn execute_wait(creep: &Creep, action: &Action) -> bool {
        if let Some(wait_ticks) = action.parameters.wait_ticks {
            // Простая реализация ожидания - можно улучшить
            if wait_ticks > 0 {
                // Уменьшаем количество оставшихся тиков
                // В реальной реализации нужно сохранять это в памяти крипа
                warn!("{} waiting for {} ticks", creep.name(), wait_ticks);
                false // Команда продолжается
            } else {
                true // Команда завершена
            }
        } else {
            warn!("No wait_ticks provided for wait command");
            true // Команда завершена с ошибкой
        }
    }
} 