use screeps::*;

#[derive(Clone, Debug, PartialEq)]
pub enum ActionType {
    HarvestEnergy,
    TransferEnergyToSpawn,
    TransferEnergyToExtension,
    TransferEnergyToTower,
    UpgradeController,
    Build,
    Repair,
    RepairWalls,
    MoveTo,
    Wait,
}

#[derive(Clone, Debug)]
pub struct Action {
    pub action_type: ActionType,
    pub target_id: Option<String>,
    pub parameters: ActionParameters,
}

#[derive(Clone, Debug)]
pub struct ActionParameters {
    pub energy_amount: Option<u32>,
    pub wait_ticks: Option<u32>,
    pub move_to_pos: Option<Position>,
}

impl Action {
    pub fn new(action_type: ActionType, target_id: Option<String>) -> Self {
        Self {
            action_type,
            target_id,
            parameters: ActionParameters::default(),
        }
    }
    pub fn with_parameters(mut self, parameters: ActionParameters) -> Self {
        self.parameters = parameters;
        self
    }
    pub fn with_energy_amount(mut self, amount: u32) -> Self {
        self.parameters.energy_amount = Some(amount);
        self
    }
    pub fn with_wait_ticks(mut self, ticks: u32) -> Self {
        self.parameters.wait_ticks = Some(ticks);
        self
    }
    pub fn with_move_to_pos(mut self, pos: Position) -> Self {
        self.parameters.move_to_pos = Some(pos);
        self
    }
}

impl Default for ActionParameters {
    fn default() -> Self {
        Self {
            energy_amount: None,
            wait_ticks: None,
            move_to_pos: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ActionQueue {
    pub actions: Vec<Action>,
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
    pub fn add_action(&mut self, action: Action) {
        self.actions.push(action);
    }
    pub fn add_actions(&mut self, actions: Vec<Action>) {
        self.actions.extend(actions);
    }
    pub fn current_action(&self) -> Option<&Action> {
        if self.current_index < self.actions.len() {
            Some(&self.actions[self.current_index])
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
    pub fn reset(&mut self) {
        self.current_index = 0;
        self.completed = false;
    }
    pub fn is_completed(&self) -> bool {
        self.completed
    }
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }
}

pub struct ActionQueueFactory;

impl ActionQueueFactory {
    pub fn transfer_energy_sequence(target_id: String) -> ActionQueue {
        let mut queue = ActionQueue::new();
        queue.add_action(Action::new(ActionType::HarvestEnergy, None));
        queue.add_action(Action::new(ActionType::TransferEnergyToSpawn, Some(target_id)));
        queue
    }
    pub fn transfer_energy_to_extension_sequence(target_id: String) -> ActionQueue {
        let mut queue = ActionQueue::new();
        queue.add_action(Action::new(ActionType::HarvestEnergy, None));
        queue.add_action(Action::new(ActionType::TransferEnergyToExtension, Some(target_id)));
        queue
    }
    pub fn transfer_energy_to_tower_sequence(target_id: String) -> ActionQueue {
        let mut queue = ActionQueue::new();
        queue.add_action(Action::new(ActionType::HarvestEnergy, None));
        queue.add_action(Action::new(ActionType::TransferEnergyToTower, Some(target_id)));
        queue
    }
    pub fn upgrade_controller_sequence(target_id: String) -> ActionQueue {
        let mut queue = ActionQueue::new();
        queue.add_action(Action::new(ActionType::HarvestEnergy, None));
        queue.add_action(Action::new(ActionType::UpgradeController, Some(target_id)));
        queue
    }
    pub fn build_sequence(target_id: String) -> ActionQueue {
        let mut queue = ActionQueue::new();
        queue.add_action(Action::new(ActionType::HarvestEnergy, None));
        queue.add_action(Action::new(ActionType::Build, Some(target_id)));
        queue
    }
    pub fn repair_sequence(target_id: String) -> ActionQueue {
        let mut queue = ActionQueue::new();
        queue.add_action(Action::new(ActionType::HarvestEnergy, None));
        queue.add_action(Action::new(ActionType::Repair, Some(target_id)));
        queue
    }
    pub fn repair_walls_sequence(target_id: String) -> ActionQueue {
        let mut queue = ActionQueue::new();
        queue.add_action(Action::new(ActionType::HarvestEnergy, None));
        queue.add_action(Action::new(ActionType::RepairWalls, Some(target_id)));
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