use log::{warn};
use screeps::*;
use screeps::action_error_codes::{UpgradeControllerErrorCode, BuildErrorCode, TransferErrorCode, CreepRepairErrorCode};
use screeps::enums::StructureObject;

// TODO: returning status (with errors) instead bool "task done"

pub fn upgrade_controller(creep: &Creep, controller_id: &ObjectId<StructureController>) -> bool {
    let mut task_done = true;
    if creep.store().get_used_capacity(Some(ResourceType::Energy)) > 0 {
        if let Some(controller) = controller_id.resolve() {
            task_done = false;
            creep
                .upgrade_controller(&controller)
                .unwrap_or_else(|e| match e {
                    UpgradeControllerErrorCode::NotInRange => {
                        creep.move_to(&controller);
                    }
                    _ => {
                        warn!("couldn't upgrade: {:?}", e);
                        task_done = true;
                    }
                });
        } else {
            warn!("couldn't resolve controller!");
            task_done = true;
        }
    };

    task_done
}

pub fn harvest_energy(creep: &Creep, source_id: &ObjectId<Source>) -> bool {
    let mut task_done = true;
    if creep.store().get_free_capacity(Some(ResourceType::Energy)) > 0 {
        if let Some(source) = source_id.resolve() {
            task_done = false;
            if creep.pos().is_near_to(source.pos()) {
                creep.harvest(&source).unwrap_or_else(|e| {
                    warn!("couldn't harvest: {:?}", e);
                    task_done = true;
                });
            } else {
                creep.move_to(&source);
            }
        }
    };

    task_done
}

pub fn build_construction(creep: &Creep, construction_id: &ObjectId<ConstructionSite>) -> bool {
    let mut task_done = true;
    if creep.store().get_used_capacity(Some(ResourceType::Energy)) > 0 {
        if let Some(construction_site) = construction_id.resolve() {
            task_done = false;
            creep.build(&construction_site).unwrap_or_else(|e| match e {
                BuildErrorCode::NotInRange => {
                    let _ = creep.move_to(&construction_site);
                }
                _ => {
                    warn!("couldn't build: {:?}", e);
                    task_done = true;
                }
            });
        } else {
            warn!("couldn't resolve construction site!");
            task_done = true;
        }
    };

    task_done
}

pub fn repair_structure(creep: &Creep, structure_obj: &StructureObject) -> bool {
    // Если нет энергии — задача завершена, надо выбрать новую цель
    if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
        return true;
    }
    let structure = structure_obj.as_structure();
    if structure.hits() < structure.hits_max() {
        let repair_result = match structure_obj {
            StructureObject::StructureWall(wall) => creep.repair(wall),
            StructureObject::StructureRampart(rampart) => creep.repair(rampart),
            StructureObject::StructureRoad(road) => creep.repair(road),
            StructureObject::StructureSpawn(spawn) => creep.repair(spawn),
            StructureObject::StructureExtension(extension) => creep.repair(extension),
            StructureObject::StructureContainer(container) => creep.repair(container),
            StructureObject::StructureTower(tower) => creep.repair(tower),
            StructureObject::StructureStorage(storage) => creep.repair(storage),
            StructureObject::StructureLink(link) => creep.repair(link),
            StructureObject::StructureLab(lab) => creep.repair(lab),
            StructureObject::StructureFactory(factory) => creep.repair(factory),
            StructureObject::StructureTerminal(terminal) => creep.repair(terminal),
            StructureObject::StructureNuker(nuker) => creep.repair(nuker),
            StructureObject::StructureObserver(observer) => creep.repair(observer),
            StructureObject::StructurePowerSpawn(power_spawn) => creep.repair(power_spawn),
            _ => {
                warn!("structure cannot be repaired!");
                return true;
            }
        };
        repair_result.unwrap_or_else(|e| match e {
            CreepRepairErrorCode::NotInRange => {
                let _ = creep.move_to(structure);
            }
            _ => {
                warn!("couldn't repair: {:?}", e);
            }
        });
        // Если после ремонта энергия закончилась — задача завершена
        if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
            return true;
        }
        // Если структура полностью отремонтирована — задача завершена
        if structure.hits() == structure.hits_max() {
            return true;
        }
        // Иначе продолжаем чинить ту же цель
        return false;
    } else {
        // Структура полностью отремонтирована
        return true;
    }
}

pub fn transfer_energy(
    creep: &Creep,
    structure_id: &ObjectId<impl Transferable + wasm_bindgen::JsCast + screeps::MaybeHasId>,
) -> bool {
    let mut task_done = true;
    if creep.store().get_used_capacity(Some(ResourceType::Energy)) > 0 {
        if let Some(structure) = structure_id.resolve() {
            task_done = false;
            creep
                .transfer(&structure, ResourceType::Energy, None)
                .unwrap_or_else(|e| match e {
                    TransferErrorCode::NotInRange => {
                        creep.move_to(&structure);
                    }
                    _ => {
                        warn!("couldn't transfer: {:?}", e);
                        task_done = true;
                    }
                });
        } else {
            warn!("couldn't resolve structure!");
            task_done = true;
        }
    };

    task_done
}
