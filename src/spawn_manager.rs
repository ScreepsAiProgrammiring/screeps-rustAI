use screeps::*;
use log::{warn, debug, info};

pub struct SpawnManager;

impl SpawnManager {
    pub fn run_spawns() {
        for spawn in game::spawns().values() {
            debug!("running spawn {}", spawn.name());
            
            // Проверяем, есть ли энергия для создания крипа
            let body = [Part::Move, Part::Carry, Part::Carry, Part::Carry, Part::Work, Part::Work, Part::Work];
            let body_cost: u32 = body.iter().map(|p| p.cost()).sum();
            
            if spawn.room().unwrap().energy_available() >= body_cost {
                // Создаём универсального крипа без фиксированной роли
                let name = format!("Worker-{}", game::time());
                
                match spawn.spawn_creep(&body, &name) {
                    Ok(()) => {
                        info!("Spawned universal worker: {}", name);
                    }
                    Err(e) => warn!("couldn't spawn: {:?}", e),
                }
            }
        }
    }
} 