use log::warn;
use num_derive::*; // 0.2.4 (the derive)
use screeps::*;
use wasm_bindgen::prelude::*;

use strum::IntoEnumIterator; // 0.17.1
use strum_macros::EnumIter; // 0.17.1

#[derive(PartialEq, Copy, Clone, FromPrimitive, EnumIter)]
pub enum Role {
    Unknown = 0,
    Harvester = 1,
    Upgrader = 2,
    Builder = 3,
    Repairer = 4,
}
impl Role {
    pub fn to_int(self) -> i32 {
        self as i32
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreepMemory {
    pub role: i32,
}

pub fn get_creep_role(creep: &Creep) -> Role {
    match num::FromPrimitive::from_i32(_get_creep_role(creep)) {
        Some(role) => role,
        None => Role::Unknown,
    }
}

#[wasm_bindgen]
pub fn _get_creep_role(creep: &Creep) -> i32 {
    match serde_wasm_bindgen::from_value::<CreepMemory>(creep.memory()) {
        Ok(memo) => memo.role,
        Err(_) => {
            0 // !!!  чезаноль?
        }
    }
}

pub fn set_creep_role(creep: &Creep, new_role: Role) -> bool {
    _set_creep_role(creep, new_role.to_int())
}

#[wasm_bindgen]
pub fn _set_creep_role(creep: &Creep, new_role: i32) -> bool {
    let creep_memory = CreepMemory { role: new_role };

    match serde_wasm_bindgen::to_value(&creep_memory) {
        Ok(value) => {
            creep.set_memory(&value);
            true
        }
        Err(_) => {
            warn!("Cant setup screep's role!");
            false
        }
    }
}
// Result<JsValue, serde_wasm_bindgen::Error>

pub fn set_creep_role_2(new_role: Role) -> SpawnOptions {
    //TODO: fiiiix dupl

    match _set_creep_role_2(new_role.to_int()) {
        Ok(value) => {
            let spawn_opitons: SpawnOptions = Default::default();
            spawn_opitons.memory(value)
        }
        Err(_) => Default::default(),
    }
}

#[wasm_bindgen]
pub fn _set_creep_role_2(new_role: i32) -> Result<JsValue, serde_wasm_bindgen::Error> {
    let creep_memory = CreepMemory { role: new_role };

    serde_wasm_bindgen::to_value(&creep_memory)
}

#[inline]
pub fn get_expected_count(role: Role) -> i32 {
    match role {
        Role::Unknown => 0,
        Role::Harvester => 2,
        Role::Upgrader => 3,
        Role::Builder => 3,
        //Role::Repairer => 2
        _ => 0,
    }
}
