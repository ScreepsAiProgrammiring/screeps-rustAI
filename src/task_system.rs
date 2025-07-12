use screeps::*;

#[derive(Clone, Debug, PartialEq)]
pub enum TaskType {
    TransferEnergyToSpawn,      // Передача энергии в Spawn
    TransferEnergyToExtension,  // Передача энергии в Extension
    TransferEnergyToTower,      // Передача энергии в Tower
    UpgradeController,          // Улучшение контроллера
    Build,                      // Строительство (общая задача)
    Repair,                     // Ремонт структур (общая задача)
    RepairWalls,                // Ремонт стен (общая задача)
}

#[derive(Clone, Debug)]
pub struct Task {
    pub task_type: TaskType,
    pub target_id: String,      // ID цели (source, structure, etc.)
}

impl Task {
    pub fn new(task_type: TaskType, target_id: String) -> Self {
        Self {
            task_type,
            target_id,
        }
    }
}

pub struct TaskBoard {
    pub room_name: RoomName,
}

impl TaskBoard {
    pub fn new(room_name: RoomName) -> Self {
        Self {
            room_name,
        }
    }
} 