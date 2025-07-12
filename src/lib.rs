use crate::room_manager::RoomManager;
use crate::creep_manager::CreepManager;
use crate::tower_manager::TowerManager;

use std::{
    cell::RefCell,
    collections::HashSet,
};

use js_sys::{JsString, Object, Reflect};
use log::*;
use screeps::{
    game,
    prelude::*,
};
use wasm_bindgen::prelude::*;

mod logging;
mod task_system;
mod task_executor;
mod command_system;
mod command_executor;
mod creep_manager;
mod spawn_manager;
mod room_manager;
mod tower_manager;



// Глобальные менеджеры для управления задачами и крипами
thread_local! {
    static ROOM_MANAGER: RefCell<RoomManager> = RefCell::new(RoomManager::new());
    static CREEP_MANAGER: RefCell<CreepManager> = RefCell::new(CreepManager::new());
}

static INIT_LOGGING: std::sync::Once = std::sync::Once::new();



// add wasm_bindgen to any function you would like to expose for call from js
// to use a reserved name as a function name, use `js_name`:
#[wasm_bindgen(js_name = loop)]
pub fn game_loop() {
    INIT_LOGGING.call_once(|| {
        // show all output of Info level, adjust as needed
        logging::setup_logging(logging::Info);
    });

    debug!("loop starting! CPU: {}", game::cpu::get_used());

    // Запускаем менеджеры
    ROOM_MANAGER.with(|room_manager_refcell| {
        CREEP_MANAGER.with(|creep_manager_refcell| {
            let mut room_manager = room_manager_refcell.borrow_mut();
            let mut creep_manager = creep_manager_refcell.borrow_mut();

            // Обновляем задачи для всех комнат
            room_manager.run_rooms();

            // Запускаем всех крипов
            debug!("running creeps");
            for creep in game::creeps().values() {
                if let Some(room) = creep.room() {
                    let action_board = room_manager.get_action_board(&room.name());
                    creep_manager.run_creep(&creep, action_board);
                }
            }

            // Очищаем мертвых крипов
            creep_manager.cleanup_dead_creeps();
        });
    });

    // Запускаем спавны
    debug!("running spawns");
    spawn_manager::SpawnManager::run_spawns();

    // Запускаем башни
    debug!("running towers");
    TowerManager::run_towers();

    // Очистка памяти
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

        // Очищаем пустые комнаты
        ROOM_MANAGER.with(|room_manager_refcell| {
            let mut room_manager = room_manager_refcell.borrow_mut();
            room_manager.cleanup_empty_rooms();
        });
    }

    info!("done! cpu: {}", game::cpu::get_used())
}


