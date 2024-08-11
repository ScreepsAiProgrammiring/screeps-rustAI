use crate::basic_tasks::{build_construction, harvest_energy, transfer_energy, upgrade_controller};
use crate::creep_roles::*;

use std::{
    cell::RefCell,
    collections::{hash_map::Entry, HashMap, HashSet},
    //marker::UnsizedConstParamTy,
};

use js_sys::{JsString, Object, Reflect};
use log::*;
use screeps::{
    constants::{ErrorCode, Part, ResourceType},
    enums::StructureObject,
    find, game,
    local::ObjectId,
    objects::{Creep, Source, StructureController},
    prelude::*,
    ConstructionSite, SpawnOptions, Structure,
};
use screeps::{StructureExtension, StructureSpawn, spawn};
use wasm_bindgen::prelude::*;

mod logging;

mod basic_tasks;
mod creep_roles;

use strum::IntoEnumIterator;

// this is one way to persist data between ticks within Rust's memory, as opposed to
// keeping state in memory on game objects - but will be lost on global resets!
thread_local! {
    static CREEP_TARGETS: RefCell<HashMap<String, CreepTarget>> = RefCell::new(HashMap::new());
}

static INIT_LOGGING: std::sync::Once = std::sync::Once::new();

// this enum will represent a creep's lock on a specific target object, storing a js reference
// to the object id so that we can grab a fresh reference to the object each successive tick,
// since screeps game objects become 'stale' and shouldn't be used beyond the tick they were fetched
#[derive(Clone)]
enum CreepTarget {
    Harvest(ObjectId<Source>),
    Upgrade(ObjectId<StructureController>),
    Build(ObjectId<ConstructionSite>),
    TransferEnergyToSpawn(ObjectId<StructureSpawn>), //TODO: think about better solution
    TransferEnergyToExtention(ObjectId<StructureExtension>),
}

// add wasm_bindgen to any function you would like to expose for call from js
// to use a reserved name as a function name, use `js_name`:
#[wasm_bindgen(js_name = loop)]
pub fn game_loop() {
    INIT_LOGGING.call_once(|| {
        // show all output of Info level, adjust as needed
        logging::setup_logging(logging::Info);
    });

    debug!("loop starting! CPU: {}", game::cpu::get_used());

    // mutably borrow the creep_targets refcell, which is holding our creep target locks
    // in the wasm heap
    CREEP_TARGETS.with(|creep_targets_refcell| {
        let mut creep_targets = creep_targets_refcell.borrow_mut();
        debug!("running creeps");
        for creep in game::creeps().values() {
            run_creep(&creep, &mut creep_targets);
        }
    });

    debug!("running spawns");
    for spawn in game::spawns().values() {
        debug!("running spawn {}", spawn.name());

        let body = [Part::Move, Part::Move, Part::Carry, Part::Work];
        if spawn.room().unwrap().energy_available() >= body.iter().map(|p| p.cost()).sum() {
            for role in Role::iter() {
                let expected_count = get_expected_count(role);
                let mut curr_count: i32 = 0;

                for creep in game::creeps().values() {
                    // TODO: full check over every role! -> get rid of "for"
                    if get_creep_role(&creep) == role {
                        curr_count += 1;
                    }
                }

                if curr_count < expected_count {
                    let name = format!("{}-{}", role.to_int(), game::time());

                    let spawn_options: SpawnOptions = set_creep_role_2(role);

                    match spawn.spawn_creep_with_options(&body, &name, &spawn_options) {
                        Ok(()) => (),
                        Err(e) => warn!("couldn't spawn: {:?}", e),
                    }
                    break;
                }
            }
        }
    }

    // memory cleanup; memory gets created for all creeps upon spawning, and any time move_to
    // is used; this should be removed if you're using RawMemory/serde for persistence
    if game::time() % 1000 == 0 {
        info!("running memory cleanup");
        let mut alive_creeps = HashSet::new();
        // add all living creep names to a hashset
        for creep_name in game::creeps().keys() {
            alive_creeps.insert(creep_name);
        }

        // grab `Memory.creeps` (if it exists)
        if let Ok(memory_creeps) = Reflect::get(&screeps::memory::ROOT, &JsString::from("creeps")) {
            // convert from JsValue to Object
            let memory_creeps: Object = memory_creeps.unchecked_into();
            // iterate memory creeps
            for creep_name_js in Object::keys(&memory_creeps).iter() {
                // convert to String (after converting to JsString)
                let creep_name = String::from(creep_name_js.dyn_ref::<JsString>().unwrap());

                // check the HashSet for the creep name, deleting if not alive
                if !alive_creeps.contains(&creep_name) {
                    info!("deleting memory for dead creep {}", creep_name);
                    let _ = Reflect::delete_property(&memory_creeps, &creep_name_js);
                }
            }
        }
    }

    info!("done! cpu: {}", game::cpu::get_used())
}

fn run_creep(creep: &Creep, creep_targets: &mut HashMap<String, CreepTarget>) {
    if creep.spawning() {
        return;
    }
    let name = creep.name();
    debug!("running creep {}", name);

    let target: Entry<'_, String, CreepTarget> = creep_targets.entry(name);
    match target {
        Entry::Occupied(entry) => {
            let creep_target = entry.get();
            let task_done = match creep_target {
                CreepTarget::Harvest(source_id) => harvest_energy(&creep, &source_id),
                CreepTarget::Upgrade(controller_id) => upgrade_controller(&creep, &controller_id),
                CreepTarget::Build(construction_id) => build_construction(&creep, &construction_id),
                CreepTarget::TransferEnergyToSpawn(spawn_id) => transfer_energy(&creep, &spawn_id),
                CreepTarget::TransferEnergyToExtention(extention_id) => {
                    transfer_energy(&creep, &extention_id)
                }
            };
            if task_done {
                entry.remove();
            }
        }
        Entry::Vacant(entry) => {
            // no target, let's set one depending on role / if we have energy
            let room = creep.room().expect("couldn't resolve creep room");

            match get_creep_role(creep) {
                Role::Upgrader => {
                    if creep.store().get_used_capacity(Some(ResourceType::Energy)) > 0 {
                        for structure in room.find(find::STRUCTURES, None).iter() {
                            if let StructureObject::StructureController(controller) = structure {
                                entry.insert(CreepTarget::Upgrade(controller.id()));
                                break;
                            }
                        }
                    } else if let Some(source) = room.find(find::SOURCES_ACTIVE, None).first() {
                        entry.insert(CreepTarget::Harvest(source.id()));
                    }
                }
                Role::Builder => {
                    if creep.store().get_used_capacity(Some(ResourceType::Energy)) > 0 {
                        for construction_site in room.find(find::CONSTRUCTION_SITES, None).iter() {
                            if let Some(id) = construction_site.try_id() {
                                entry.insert(CreepTarget::Build(id));
                                break;
                            }
                        }
                    } else if let Some(source) = room.find(find::SOURCES_ACTIVE, None).first() {
                        entry.insert(CreepTarget::Harvest(source.id()));
                    }
                }
                Role::Harvester => {
                    if creep.store().get_used_capacity(Some(ResourceType::Energy)) > 0 {
                        for structure in room.find(find::STRUCTURES, None).iter() {
                            //TODO: filter instead of None
                            if let StructureObject::StructureSpawn(spawn) = structure {
                                if spawn.store().get_free_capacity(Some(ResourceType::Energy)) > 0 {
                                    entry.insert(CreepTarget::TransferEnergyToSpawn(spawn.id()));
                                    break;
                                }
                            };

                            if let StructureObject::StructureExtension(extention) = structure {
                                if extention.store().get_free_capacity(Some(ResourceType::Energy)) > 0 {
                                entry.insert(CreepTarget::TransferEnergyToExtention(extention.id()));
                                break;
                                }
                            };
                        }
                    } else if let Some(source) = room.find(find::SOURCES_ACTIVE, None).first() {
                        entry.insert(CreepTarget::Harvest(source.id()));
                    }
                }
                _ => {
                    warn!("Creep just suicided!!");
                    creep.suicide();
                }
            }
        }
    }
}
