use screeps::*;
use log::{warn, info};
use wasm_bindgen::JsCast;

pub struct TowerManager;

impl TowerManager {
    pub fn run_towers() {
        for room in game::rooms().values() {
            // Ищем все башни в комнате
            for structure in room.find(find::STRUCTURES, None).iter() {
                if let screeps::enums::StructureObject::StructureTower(tower) = structure {
                    Self::run_tower(&tower);
                }
            }
        }
    }

    fn run_tower(tower: &StructureTower) {
        // Проверяем, есть ли энергия в башне
        let energy = tower.store().get_used_capacity(Some(ResourceType::Energy));
        if energy == 0 {
            return; // Нет энергии - ничего не делаем
        }

        // Ищем вражеские существа в комнате
        if let Some(room) = tower.room() {
            let hostiles = room.find(find::HOSTILE_CREEPS, None);
            
            if !hostiles.is_empty() {
                // Атакуем первого найденного врага
                if let Some(target) = hostiles.first() {
                    match tower.attack(target) {
                        Ok(_) => {
                            info!("Tower {} attacking {}", tower.id(), target.name());
                        }
                        Err(e) => {
                            warn!("Tower {} couldn't attack: {:?}", tower.id(), e);
                        }
                    }
                }
            } else {
                // Если нет врагов, можно использовать башню для ремонта
                Self::repair_with_tower(tower);
            }
        }
    }



    fn repair_with_tower(tower: &StructureTower) {
        // Ищем повреждённые структуры в радиусе действия башни
        if let Some(room) = tower.room() {
            for structure in room.find(find::STRUCTURES, None).iter() {
                let structure_ref = structure.as_structure();
                let hits = structure_ref.hits();
                let hits_max = structure_ref.hits_max();
                
                // Ищем структуры с повреждениями, но не стены
                if hits > 0 && hits < hits_max && 
                   structure_ref.structure_type() != StructureType::Wall &&
                   structure_ref.structure_type() != StructureType::Rampart {
                    
                    // Пытаемся отремонтировать структуру в зависимости от типа
                    let repair_result = match structure {
                        screeps::enums::StructureObject::StructureExtension(ext) => tower.repair(ext),
                        screeps::enums::StructureObject::StructureSpawn(spawn) => tower.repair(spawn),
                        screeps::enums::StructureObject::StructureContainer(container) => tower.repair(container),
                        screeps::enums::StructureObject::StructureRoad(road) => tower.repair(road),
                        screeps::enums::StructureObject::StructureStorage(storage) => tower.repair(storage),
                        screeps::enums::StructureObject::StructureTower(tower_target) => tower.repair(tower_target),
                        screeps::enums::StructureObject::StructureObserver(observer) => tower.repair(observer),
                        screeps::enums::StructureObject::StructurePowerSpawn(power_spawn) => tower.repair(power_spawn),
                        screeps::enums::StructureObject::StructureExtractor(extractor) => tower.repair(extractor),
                        screeps::enums::StructureObject::StructureLab(lab) => tower.repair(lab),
                        screeps::enums::StructureObject::StructureTerminal(terminal) => tower.repair(terminal),
                        screeps::enums::StructureObject::StructureLink(link) => tower.repair(link),
                        screeps::enums::StructureObject::StructureFactory(factory) => tower.repair(factory),
                        screeps::enums::StructureObject::StructureNuker(nuker) => tower.repair(nuker),
                        _ => continue, // Пропускаем другие типы структур
                    };
                    
                    match repair_result {
                        Ok(_) => {
                            info!("Tower {} repairing {}", tower.id(), structure_ref.id());
                            return; // Ремонтируем только одну структуру за раз
                        }
                        Err(e) => {
                            warn!("Tower {} couldn't repair: {:?}", tower.id(), e);
                        }
                    }
                }
            }
        }
    }
} 