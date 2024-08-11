use log::{info, warn};
use screeps::*;

// TODO: returning status (with errors) instead bool "task done"

pub fn upgrade_controller(creep: &Creep, controller_id: &ObjectId<StructureController>) -> bool {
    let mut task_done = true;
    if creep.store().get_used_capacity(Some(ResourceType::Energy)) > 0 {
        if let Some(controller) = controller_id.resolve() {
            task_done = false;
            creep
                .upgrade_controller(&controller)
                .unwrap_or_else(|e| match e {
                    ErrorCode::NotInRange => {
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
                ErrorCode::NotInRange => {
                    let _ = creep.move_to(&construction_site);
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
                    ErrorCode::NotInRange => {
                        creep.move_to(&structure);
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
