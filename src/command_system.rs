use screeps::*;

// Трейт для действий крипов
pub trait CreepAction: std::fmt::Debug {
    fn execute(&self, creep: &Creep) -> bool;
    fn get_description(&self) -> String;
}

// Трейт для действий башен
pub trait TowerAction: std::fmt::Debug {
    fn execute(&self, tower: &StructureTower) -> bool;
    fn get_description(&self) -> String;
}

// Enum для ремонтируемых структур
#[derive(Clone, Debug)]
pub enum RepairableStructure {
    Extension(screeps::objects::StructureExtension),
    Spawn(screeps::objects::StructureSpawn),
    Container(screeps::objects::StructureContainer),
    Road(screeps::objects::StructureRoad),
    Wall(screeps::objects::StructureWall),
    Rampart(screeps::objects::StructureRampart),
}

impl RepairableStructure {
    pub fn hits(&self) -> u32 {
        match self {
            RepairableStructure::Extension(s) => s.hits(),
            RepairableStructure::Spawn(s) => s.hits(),
            RepairableStructure::Container(s) => s.hits(),
            RepairableStructure::Road(s) => s.hits(),
            RepairableStructure::Wall(s) => s.hits(),
            RepairableStructure::Rampart(s) => s.hits(),
        }
    }

    pub fn hits_max(&self) -> u32 {
        match self {
            RepairableStructure::Extension(s) => s.hits_max(),
            RepairableStructure::Spawn(s) => s.hits_max(),
            RepairableStructure::Container(s) => s.hits_max(),
            RepairableStructure::Road(s) => s.hits_max(),
            RepairableStructure::Wall(s) => s.hits_max(),
            RepairableStructure::Rampart(s) => s.hits_max(),
        }
    }

    pub fn pos(&self) -> screeps::Position {
        match self {
            RepairableStructure::Extension(s) => s.pos(),
            RepairableStructure::Spawn(s) => s.pos(),
            RepairableStructure::Container(s) => s.pos(),
            RepairableStructure::Road(s) => s.pos(),
            RepairableStructure::Wall(s) => s.pos(),
            RepairableStructure::Rampart(s) => s.pos(),
        }
    }

    // Общая функция ремонта для любого исполнителя
    pub fn repair_with<T>(&self, repairer: &T) -> Result<(), String> 
    where 
        T: Repairer,
    {
        repairer.repair_structure(self)
    }
}

// Трейт для исполнителей ремонта
pub trait Repairer {
    fn repair_structure(&self, structure: &RepairableStructure) -> Result<(), String>;
}

// Реализация для крипов
impl Repairer for Creep {
    fn repair_structure(&self, structure: &RepairableStructure) -> Result<(), String> {
        match structure {
            RepairableStructure::Extension(s) => self.repair(s).map_err(|e| format!("{:?}", e)),
            RepairableStructure::Spawn(s) => self.repair(s).map_err(|e| format!("{:?}", e)),
            RepairableStructure::Container(s) => self.repair(s).map_err(|e| format!("{:?}", e)),
            RepairableStructure::Road(s) => self.repair(s).map_err(|e| format!("{:?}", e)),
            RepairableStructure::Wall(s) => self.repair(s).map_err(|e| format!("{:?}", e)),
            RepairableStructure::Rampart(s) => self.repair(s).map_err(|e| format!("{:?}", e)),
        }
    }
}

// Реализация для башен
impl Repairer for StructureTower {
    fn repair_structure(&self, structure: &RepairableStructure) -> Result<(), String> {
        match structure {
            RepairableStructure::Extension(s) => self.repair(s).map_err(|e| format!("{:?}", e)),
            RepairableStructure::Spawn(s) => self.repair(s).map_err(|e| format!("{:?}", e)),
            RepairableStructure::Container(s) => self.repair(s).map_err(|e| format!("{:?}", e)),
            RepairableStructure::Road(s) => self.repair(s).map_err(|e| format!("{:?}", e)),
            RepairableStructure::Wall(s) => self.repair(s).map_err(|e| format!("{:?}", e)),
            RepairableStructure::Rampart(s) => self.repair(s).map_err(|e| format!("{:?}", e)),
        }
    }
}



// Конкретные типы действий
#[derive(Clone, Debug)]
pub enum EnergySourceTarget {
    Source(screeps::objects::Source),
}

#[derive(Clone, Debug)]
pub struct HarvestEnergyAction {
    pub target: EnergySourceTarget,
}

impl CreepAction for HarvestEnergyAction {
    fn execute(&self, creep: &Creep) -> bool {
        if creep.store().get_free_capacity(Some(ResourceType::Energy)) == 0 {
            return true;
        }
        match &self.target {
            EnergySourceTarget::Source(source) => {
                if creep.pos().is_near_to(source.pos()) {
                    match creep.harvest(source) {
                        Ok(_) => false,
                        Err(e) => {
                            log::warn!("Couldn't harvest: {:?}", e);
                            true
                        }
                    }
                } else {
                    let _ = creep.move_to(&source);
                    false
                }
            }
        }
    }

    fn get_description(&self) -> String {
        "Harvesting energy from source".to_string()
    }
}

// Общий трейт для передачи энергии
pub trait TransferEnergyAction {
    fn get_target_id(&self) -> &str;
    fn get_target_description(&self) -> &str;
}

impl<T: TransferEnergyAction + std::fmt::Debug> CreepAction for T {
    fn execute(&self, creep: &Creep) -> bool {
        // Проверяем, есть ли энергия для передачи
        if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
            return true; // Команда завершена - нет энергии
        }

        // Пытаемся передать энергию
        if let Ok(target_id) = self.get_target_id().parse::<screeps::local::ObjectId<screeps::objects::StructureSpawn>>() {
            if let Some(target) = target_id.resolve() {
                // Проверяем, нужна ли энергия в цели
                if target.store().get_free_capacity(Some(ResourceType::Energy)) == 0 {
                    return true; // Команда завершена - цель полная
                }
                
                match creep.transfer(&target, ResourceType::Energy, None) {
                    Ok(_) => false, // Команда продолжается - продолжаем передавать
                    Err(e) => match e {
                        screeps::action_error_codes::TransferErrorCode::NotInRange => {
                            let _ = creep.move_to(&target);
                            false // Команда продолжается
                        }
                        _ => {
                            log::warn!("Couldn't transfer energy: {:?}", e);
                            true // Команда завершена с ошибкой
                        }
                    }
                }
            } else {
                log::warn!("Couldn't resolve target!");
                true // Команда завершена
            }
        } else {
            log::warn!("Invalid target ID: {}", self.get_target_id());
            true // Команда завершена
        }
    }

    fn get_description(&self) -> String {
        format!("Refilling {} [{}]", self.get_target_description(), &self.get_target_id()[..5])
    }
}

#[derive(Clone, Debug)]
pub struct TransferEnergyToSpawnAction {
    pub target_id: String,
}

impl TransferEnergyAction for TransferEnergyToSpawnAction {
    fn get_target_id(&self) -> &str {
        &self.target_id
    }
    
    fn get_target_description(&self) -> &str {
        "spawn"
    }
}

#[derive(Clone, Debug)]
pub struct TransferEnergyToExtensionAction {
    pub target_id: String,
}

impl TransferEnergyAction for TransferEnergyToExtensionAction {
    fn get_target_id(&self) -> &str {
        &self.target_id
    }
    
    fn get_target_description(&self) -> &str {
        "extension"
    }
}

#[derive(Clone, Debug)]
pub struct TransferEnergyToTowerAction {
    pub target_id: String,
}

impl TransferEnergyAction for TransferEnergyToTowerAction {
    fn get_target_id(&self) -> &str {
        &self.target_id
    }
    
    fn get_target_description(&self) -> &str {
        "tower"
    }
}

#[derive(Clone, Debug)]
pub struct UpgradeControllerAction {
    pub target_id: String,
}

impl CreepAction for UpgradeControllerAction {
    fn execute(&self, creep: &Creep) -> bool {
        // Проверяем, есть ли энергия для улучшения
        if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
            return true; // Команда завершена - нет энергии
        }

        // Пытаемся улучшить контроллер
        if let Ok(controller_id) = self.target_id.parse::<screeps::local::ObjectId<screeps::objects::StructureController>>() {
            if let Some(controller) = controller_id.resolve() {
                match creep.upgrade_controller(&controller) {
                    Ok(_) => false, // Команда продолжается - продолжаем улучшать
                    Err(e) => match e {
                        screeps::action_error_codes::UpgradeControllerErrorCode::NotInRange => {
                            let _ = creep.move_to(&controller);
                            false // Команда продолжается
                        }
                        _ => {
                            log::warn!("Couldn't upgrade controller: {:?}", e);
                            true // Команда завершена с ошибкой
                        }
                    }
                }
            } else {
                log::warn!("Couldn't resolve controller!");
                true // Команда завершена
            }
        } else {
            log::warn!("Invalid controller ID: {}", self.target_id);
            true // Команда завершена
        }
    }

    fn get_description(&self) -> String {
        "Upgrading controller".to_string()
    }
}

#[derive(Clone, Debug)]
pub struct BuildAction {
    pub target_id: String,
}

impl CreepAction for BuildAction {
    fn execute(&self, creep: &Creep) -> bool {
        // Проверяем, есть ли энергия для строительства
        if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
            return true; // Команда завершена - нет энергии
        }

        // Пытаемся строить
        if let Ok(construction_id) = self.target_id.parse::<screeps::local::ObjectId<screeps::objects::ConstructionSite>>() {
            if let Some(construction_site) = construction_id.resolve() {
                match creep.build(&construction_site) {
                    Ok(_) => false, // Команда продолжается - продолжаем строить
                    Err(e) => match e {
                        screeps::action_error_codes::BuildErrorCode::NotInRange => {
                            let _ = creep.move_to(&construction_site);
                            false // Команда продолжается
                        }
                        _ => {
                            log::warn!("Couldn't build: {:?}", e);
                            true // Команда завершена с ошибкой
                        }
                    }
                }
            } else {
                log::warn!("Couldn't resolve construction site!");
                true // Команда завершена
            }
        } else {
            log::warn!("Invalid construction site ID: {}", self.target_id);
            true // Команда завершена
        }
    }

    fn get_description(&self) -> String {
        format!("Building site [{}]", &self.target_id[..5])
    }
}

#[derive(Clone, Debug)]
pub struct RepairAction {
    pub target_id: String,
}

impl CreepAction for RepairAction {
    fn execute(&self, creep: &Creep) -> bool {
        // Проверяем, есть ли энергия для ремонта
        if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
            return true; // Команда завершена - нет энергии
        }

        // Пытаемся найти и отремонтировать структуру
        if let Some(structure) = Self::try_resolve_structure(&self.target_id) {
            return Self::repair_structure(creep, &structure);
        }

        log::warn!("Invalid structure ID: {}", self.target_id);
        true // Команда завершена
    }

    fn get_description(&self) -> String {
        format!("Repairing structure [{}]", &self.target_id[..5])
    }

}

impl RepairAction {
    fn try_resolve_structure(target_id: &str) -> Option<RepairableStructure> {
        // Пробуем разные типы структур
        if let Ok(structure_id) = target_id.parse::<screeps::local::ObjectId<screeps::objects::StructureExtension>>() {
            if let Some(structure) = structure_id.resolve() {
                return Some(RepairableStructure::Extension(structure));
            }
        }
        
        if let Ok(structure_id) = target_id.parse::<screeps::local::ObjectId<screeps::objects::StructureSpawn>>() {
            if let Some(structure) = structure_id.resolve() {
                return Some(RepairableStructure::Spawn(structure));
            }
        }
        
        if let Ok(structure_id) = target_id.parse::<screeps::local::ObjectId<screeps::objects::StructureContainer>>() {
            if let Some(structure) = structure_id.resolve() {
                return Some(RepairableStructure::Container(structure));
            }
        }
        
        if let Ok(structure_id) = target_id.parse::<screeps::local::ObjectId<screeps::objects::StructureRoad>>() {
            if let Some(structure) = structure_id.resolve() {
                return Some(RepairableStructure::Road(structure));
            }
        }
        
        if let Ok(structure_id) = target_id.parse::<screeps::local::ObjectId<screeps::objects::StructureWall>>() {
            if let Some(structure) = structure_id.resolve() {
                return Some(RepairableStructure::Wall(structure));
            }
        }
        
        if let Ok(structure_id) = target_id.parse::<screeps::local::ObjectId<screeps::objects::StructureRampart>>() {
            if let Some(structure) = structure_id.resolve() {
                return Some(RepairableStructure::Rampart(structure));
            }
        }
        
        None
    }

    fn repair_structure(creep: &Creep, structure: &RepairableStructure) -> bool {
        // Проверяем, нужен ли ремонт
        let hits = structure.hits();
        let hits_max = structure.hits_max();
        
        if hits >= hits_max {
            return true; // Команда завершена - структура здорова
        }

        match structure.repair_with(creep) {
            Ok(_) => false, // Команда продолжается - продолжаем чинить
            Err(e) => {
                if e.contains("NotInRange") {
                    // Крип должен подойти к структуре
                    let pos = structure.pos();
                    let _ = creep.move_to(pos);
                    false // Команда продолжается
                } else {
                    log::warn!("Couldn't repair: {}", e);
                    true // Команда завершена с ошибкой
                }
            }
        }
    }
}



pub struct ActionQueue {
    pub actions: Vec<Box<dyn CreepAction>>,
    pub current_index: usize,
    pub completed: bool,
}

impl ActionQueue {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
            current_index: 0,
            completed: false,
        }
    }
    
    pub fn add_action(&mut self, action: Box<dyn CreepAction>) {
        self.actions.push(action);
    }

    pub fn current_action(&self) -> Option<&dyn CreepAction> {
        if self.current_index < self.actions.len() {
            Some(self.actions[self.current_index].as_ref())
        } else {
            None
        }
    }
    
    pub fn next_action(&mut self) -> bool {
        self.current_index += 1;
        if self.current_index >= self.actions.len() {
            self.completed = true;
            false
        } else {
            true
        }
    }
}

pub struct ActionQueueFactory;

impl ActionQueueFactory {
    pub fn transfer_energy_sequence(energy_target: EnergySourceTarget, target_id: String) -> ActionQueue {
        let mut queue = ActionQueue::new();
        queue.add_action(Box::new(HarvestEnergyAction { target: energy_target.clone() }));
        queue.add_action(Box::new(TransferEnergyToSpawnAction { target_id }));
        queue
    }
    
    pub fn transfer_energy_to_extension_sequence(energy_target: EnergySourceTarget, target_id: String) -> ActionQueue {
        let mut queue = ActionQueue::new();
        queue.add_action(Box::new(HarvestEnergyAction { target: energy_target.clone() }));
        queue.add_action(Box::new(TransferEnergyToExtensionAction { target_id }));
        queue
    }
    
    pub fn transfer_energy_to_tower_sequence(energy_target: EnergySourceTarget, target_id: String) -> ActionQueue {
        let mut queue = ActionQueue::new();
        queue.add_action(Box::new(HarvestEnergyAction { target: energy_target.clone() }));
        queue.add_action(Box::new(TransferEnergyToTowerAction { target_id }));
        queue
    }
    
    pub fn upgrade_controller_sequence(energy_target: EnergySourceTarget, target_id: String) -> ActionQueue {
        let mut queue = ActionQueue::new();
        queue.add_action(Box::new(HarvestEnergyAction { target: energy_target.clone() }));
        queue.add_action(Box::new(UpgradeControllerAction { target_id }));
        queue
    }
    
    pub fn build_sequence(energy_target: EnergySourceTarget, target_id: String) -> ActionQueue {
        let mut queue = ActionQueue::new();
        queue.add_action(Box::new(HarvestEnergyAction { target: energy_target.clone() }));
        queue.add_action(Box::new(BuildAction { target_id }));
        queue
    }
    
    pub fn repair_sequence(energy_target: EnergySourceTarget, target_id: String) -> ActionQueue {
        let mut queue = ActionQueue::new();
        queue.add_action(Box::new(HarvestEnergyAction { target: energy_target.clone() }));
        queue.add_action(Box::new(RepairAction { target_id }));
        queue
    }
    
    pub fn repair_walls_sequence(energy_target: EnergySourceTarget, target_id: String) -> ActionQueue {
        let mut queue = ActionQueue::new();
        queue.add_action(Box::new(HarvestEnergyAction { target: energy_target.clone() }));
        queue.add_action(Box::new(RepairAction { target_id }));
        queue
    }
}

pub struct ActionBoard {
    pub room_name: RoomName,
}

impl ActionBoard {
    pub fn new(room_name: RoomName) -> Self {
        Self {
            room_name,
        }
    }
}

// Действие для башен - ремонт структур
#[derive(Clone, Debug)]
pub struct TowerRepairAction {
    pub target_id: String,
}

impl TowerAction for TowerRepairAction {
    fn execute(&self, tower: &StructureTower) -> bool {
        // Проверяем, есть ли энергия в башне
        if tower.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
            return true; // Нет энергии - действие завершено
        }

        // Пытаемся найти и отремонтировать структуру
        if let Some(structure) = Self::try_resolve_structure(&self.target_id) {
            return Self::repair_structure(tower, &structure);
        }

        log::warn!("Invalid structure ID for tower repair: {}", self.target_id);
        true // Действие завершено
    }

    fn get_description(&self) -> String {
        format!("Tower repairing structure [{}]", &self.target_id[..5])
    }
}

impl TowerRepairAction {
    fn try_resolve_structure(target_id: &str) -> Option<RepairableStructure> {
        // Пробуем разные типы структур
        if let Ok(structure_id) = target_id.parse::<screeps::local::ObjectId<screeps::objects::StructureExtension>>() {
            if let Some(structure) = structure_id.resolve() {
                return Some(RepairableStructure::Extension(structure));
            }
        }
        
        if let Ok(structure_id) = target_id.parse::<screeps::local::ObjectId<screeps::objects::StructureSpawn>>() {
            if let Some(structure) = structure_id.resolve() {
                return Some(RepairableStructure::Spawn(structure));
            }
        }
        
        if let Ok(structure_id) = target_id.parse::<screeps::local::ObjectId<screeps::objects::StructureContainer>>() {
            if let Some(structure) = structure_id.resolve() {
                return Some(RepairableStructure::Container(structure));
            }
        }
        
        if let Ok(structure_id) = target_id.parse::<screeps::local::ObjectId<screeps::objects::StructureRoad>>() {
            if let Some(structure) = structure_id.resolve() {
                return Some(RepairableStructure::Road(structure));
            }
        }
        
        if let Ok(structure_id) = target_id.parse::<screeps::local::ObjectId<screeps::objects::StructureWall>>() {
            if let Some(structure) = structure_id.resolve() {
                return Some(RepairableStructure::Wall(structure));
            }
        }
        
        if let Ok(structure_id) = target_id.parse::<screeps::local::ObjectId<screeps::objects::StructureRampart>>() {
            if let Some(structure) = structure_id.resolve() {
                return Some(RepairableStructure::Rampart(structure));
            }
        }
        
        None
    }

    fn repair_structure(tower: &StructureTower, structure: &RepairableStructure) -> bool {
        // Проверяем, нужен ли ремонт
        let hits = structure.hits();
        let hits_max = structure.hits_max();
        
        if hits >= hits_max {
            return true; // Действие завершено - структура здорова
        }

        match structure.repair_with(tower) {
            Ok(_) => false, // Действие продолжается - продолжаем чинить
            Err(e) => {
                log::warn!("Tower couldn't repair: {}", e);
                true // Действие завершено с ошибкой
            }
        }
    }
} 