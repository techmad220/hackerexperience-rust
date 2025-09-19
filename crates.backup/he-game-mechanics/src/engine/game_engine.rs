//! Main Game Engine - Integrates all subsystems into a cohesive game experience

use super::{
    ProcessEngine, HardwareEngine, SoftwareEngine, NetworkEngine,
    EngineComponent, EngineError, EngineResult, ComponentStatus, Resources,
    process_engine::{Process, ProcessType, Priority},
    hardware_engine::HardwareConfiguration,
    software_engine::{Software, SoftwareCategory, SoftwareVersion},
    network_engine::{NetworkLog, LogAction},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Player state in the game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: Uuid,
    pub username: String,
    pub ip_address: Ipv4Addr,
    pub hardware_id: Uuid,
    pub money: f64,
    pub bitcoin: f64,
    pub experience: u64,
    pub level: u32,
    pub reputation: i32,
    pub clan_id: Option<Uuid>,
    pub created_at: SystemTime,
    pub last_login: SystemTime,
}

impl Player {
    pub fn new(username: String, ip_address: Ipv4Addr, hardware_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            username,
            ip_address,
            hardware_id,
            money: 1000.0,
            bitcoin: 0.0,
            experience: 0,
            level: 1,
            reputation: 0,
            clan_id: None,
            created_at: SystemTime::now(),
            last_login: SystemTime::now(),
        }
    }

    pub fn add_experience(&mut self, amount: u64) {
        self.experience += amount;
        // Level up formula
        while self.experience >= self.experience_for_next_level() {
            self.level += 1;
        }
    }

    pub fn experience_for_next_level(&self) -> u64 {
        (self.level as u64 + 1) * 1000
    }
}

/// Game action that can be performed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameAction {
    // Process actions
    StartProcess {
        process_type: ProcessType,
        target_ip: Option<Ipv4Addr>,
        priority: Priority,
    },
    PauseProcess(Uuid),
    ResumeProcess(Uuid),
    CancelProcess(Uuid),

    // Hardware actions
    PurchaseHardware(String),
    UpgradeComponent(Uuid),
    RepairComponent(Uuid),

    // Software actions
    ResearchSoftware(String),
    InstallSoftware(Uuid),
    UninstallSoftware(Uuid),
    HideSoftware(Uuid),
    SeekHidden,

    // Network actions
    Connect(Ipv4Addr),
    Disconnect,
    Scan(Ipv4Addr),
    StartDDoS {
        target: Ipv4Addr,
        botnet_size: usize,
    },
    EditLog {
        ip: Ipv4Addr,
        log_id: Uuid,
        new_content: String,
    },
    DeleteLog {
        ip: Ipv4Addr,
        log_id: Uuid,
    },

    // Financial actions
    TransferMoney {
        to_player: Uuid,
        amount: f64,
    },
    MineBitcoin,
    ConvertBitcoin(f64),
}

/// Game event notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameEvent {
    ProcessCompleted {
        process_id: Uuid,
        process_type: ProcessType,
        result: String,
    },
    ProcessFailed {
        process_id: Uuid,
        reason: String,
    },
    HackSuccessful {
        target_ip: Ipv4Addr,
        loot: String,
    },
    HackDetected {
        by_ip: Ipv4Addr,
    },
    MoneyReceived {
        from: Uuid,
        amount: f64,
    },
    LevelUp {
        new_level: u32,
    },
    ServerCompromised {
        ip: Ipv4Addr,
    },
    MissionCompleted {
        mission_id: Uuid,
        reward: f64,
    },
}

/// Complete game state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub tick: u64,
    pub time: SystemTime,
    pub players: HashMap<Uuid, Player>,
    pub events: Vec<GameEvent>,
}

/// Main Game Engine - The heart of HackerExperience
pub struct GameEngine {
    // Sub-engines
    process_engine: ProcessEngine,
    hardware_engine: HardwareEngine,
    software_engine: SoftwareEngine,
    network_engine: NetworkEngine,

    // Game state
    players: HashMap<Uuid, Player>,
    active_sessions: HashMap<Uuid, Ipv4Addr>,
    event_queue: Vec<GameEvent>,

    // Engine state
    tick: u64,
    last_update: SystemTime,
    tick_rate: Duration,
}

impl GameEngine {
    pub fn new() -> Self {
        let base_resources = Resources::new(1000.0, 1024.0, 10000.0, 100.0);

        Self {
            process_engine: ProcessEngine::new(base_resources, 10),
            hardware_engine: HardwareEngine::new(),
            software_engine: SoftwareEngine::new(),
            network_engine: NetworkEngine::new(),
            players: HashMap::new(),
            active_sessions: HashMap::new(),
            event_queue: Vec::new(),
            tick: 0,
            last_update: SystemTime::now(),
            tick_rate: Duration::from_millis(100),
        }
    }

    /// Register a new player
    pub fn register_player(&mut self, username: String) -> EngineResult<Uuid> {
        // Create hardware configuration
        let hardware_id = self.hardware_engine.create_configuration(
            Uuid::new_v4(),
            format!("{}'s PC", username),
        );

        // Add starter hardware
        self.hardware_engine.purchase_component(hardware_id, "Pentium III")?;
        self.hardware_engine.purchase_component(hardware_id, "DDR 256MB")?;
        self.hardware_engine.purchase_component(hardware_id, "10GB IDE")?;
        self.hardware_engine.purchase_component(hardware_id, "10Mbps Ethernet")?;

        // Create network node
        let ip = self.network_engine.create_player_server(Uuid::new_v4());

        // Create software inventory
        self.software_engine.create_inventory(Uuid::new_v4(), 10000.0);

        // Create player
        let player = Player::new(username, ip, hardware_id);
        let player_id = player.id;

        self.players.insert(player_id, player);

        Ok(player_id)
    }

    /// Execute a game action
    pub fn execute_action(&mut self, player_id: Uuid, action: GameAction) -> EngineResult<()> {
        let player = self.players.get(&player_id)
            .ok_or_else(|| EngineError::NotFound("Player not found".into()))?;

        match action {
            GameAction::StartProcess { process_type, target_ip, priority } => {
                self.handle_start_process(player_id, process_type, target_ip, priority)?;
            }
            GameAction::PauseProcess(id) => {
                self.process_engine.pause_process(id)?;
            }
            GameAction::ResumeProcess(id) => {
                self.process_engine.resume_process(id)?;
            }
            GameAction::CancelProcess(id) => {
                self.process_engine.cancel_process(id)?;
            }
            GameAction::PurchaseHardware(model) => {
                let hardware_id = player.hardware_id;
                self.hardware_engine.purchase_component(hardware_id, &model)?;
            }
            GameAction::ResearchSoftware(name) => {
                self.software_engine.research_software(player_id, &name)?;
            }
            GameAction::InstallSoftware(id) => {
                self.software_engine.install_software(player_id, id)?;
            }
            GameAction::Connect(ip) => {
                let from_ip = player.ip_address;
                self.network_engine.connect(player_id, from_ip, ip)?;
                self.active_sessions.insert(player_id, ip);
            }
            GameAction::Disconnect => {
                self.network_engine.disconnect(player_id)?;
                self.active_sessions.remove(&player_id);
            }
            GameAction::Scan(target) => {
                let from_ip = player.ip_address;
                let ports = self.network_engine.scan(from_ip, target)?;
                // Create scan process
                self.handle_scan_result(player_id, target, ports)?;
            }
            GameAction::TransferMoney { to_player, amount } => {
                self.handle_money_transfer(player_id, to_player, amount)?;
            }
            _ => {
                // Handle other actions
            }
        }

        Ok(())
    }

    fn handle_start_process(
        &mut self,
        player_id: Uuid,
        process_type: ProcessType,
        target_ip: Option<Ipv4Addr>,
        priority: Priority,
    ) -> EngineResult<()> {
        let player = self.players.get(&player_id).unwrap();
        let hardware = self.hardware_engine.get_configuration(player.hardware_id)
            .ok_or_else(|| EngineError::NotFound("Hardware not found".into()))?;

        // Calculate resource requirements
        let complexity = process_type.base_complexity();
        let resources = Resources::new(
            complexity * 100.0,  // CPU
            complexity * 50.0,   // RAM
            complexity * 10.0,   // Disk
            complexity * 5.0,    // Network
        );

        // Calculate process time
        let time = self.hardware_engine.calculate_process_time(complexity, player.hardware_id)?;

        // Create process
        let target_id = target_ip.and_then(|ip| {
            self.network_engine.get_topology()
                .get_node(ip)
                .map(|node| node.id)
        });

        let process = Process::new(process_type, player_id, target_id, resources, time)
            .with_priority(priority)
            .with_callback(format!("complete_process_{}", player_id));

        // Submit to process engine
        self.process_engine.submit_process(process)?;

        Ok(())
    }

    fn handle_scan_result(&mut self, player_id: Uuid, target: Ipv4Addr, ports: Vec<u16>) -> EngineResult<()> {
        // Log scan results
        if let Some(node) = self.network_engine.get_topology_mut().get_node_mut(target) {
            node.add_log(NetworkLog::new(
                self.players.get(&player_id).unwrap().ip_address,
                LogAction::Scan,
                format!("Open ports: {:?}", ports),
            ));
        }

        // Give experience
        if let Some(player) = self.players.get_mut(&player_id) {
            player.add_experience(10);
        }

        Ok(())
    }

    fn handle_money_transfer(&mut self, from_id: Uuid, to_id: Uuid, amount: f64) -> EngineResult<()> {
        let from_player = self.players.get_mut(&from_id)
            .ok_or_else(|| EngineError::NotFound("Sender not found".into()))?;

        if from_player.money < amount {
            return Err(EngineError::InvalidOperation("Insufficient funds".into()));
        }

        from_player.money -= amount;

        let to_player = self.players.get_mut(&to_id)
            .ok_or_else(|| EngineError::NotFound("Recipient not found".into()))?;

        to_player.money += amount;

        self.event_queue.push(GameEvent::MoneyReceived {
            from: from_id,
            amount,
        });

        Ok(())
    }

    /// Get current game state
    pub fn get_state(&self) -> GameState {
        GameState {
            tick: self.tick,
            time: self.last_update,
            players: self.players.clone(),
            events: self.event_queue.clone(),
        }
    }

    /// Get player by ID
    pub fn get_player(&self, id: Uuid) -> Option<&Player> {
        self.players.get(&id)
    }

    /// Get player's hardware
    pub fn get_player_hardware(&self, player_id: Uuid) -> Option<&HardwareConfiguration> {
        self.players.get(&player_id)
            .and_then(|p| self.hardware_engine.get_configuration(p.hardware_id))
    }

    /// Get player's software inventory
    pub fn get_player_software(&self, player_id: Uuid) -> Option<&Software> {
        // Implementation needed
        None
    }

    /// Get active processes for player
    pub fn get_player_processes(&self, player_id: Uuid) -> Vec<&Process> {
        self.process_engine.executor.list_running()
            .into_iter()
            .filter(|p| p.owner_id == player_id)
            .collect()
    }

    /// Process completed processes
    fn process_completed(&mut self) {
        // This would be called when processes complete
        // Generate appropriate events
    }
}

impl EngineComponent for GameEngine {
    fn initialize(&mut self) -> EngineResult<()> {
        self.process_engine.initialize()?;
        self.hardware_engine.initialize()?;
        self.software_engine.initialize()?;
        self.network_engine.initialize()?;

        // Create initial NPCs and world state
        for i in 1..=10 {
            let username = format!("NPC_{}", i);
            self.register_player(username)?;
        }

        Ok(())
    }

    fn update(&mut self, delta: Duration) -> EngineResult<()> {
        // Update all sub-engines
        self.process_engine.update(delta)?;
        self.hardware_engine.update(delta)?;
        self.software_engine.update(delta)?;
        self.network_engine.update(delta)?;

        // Process game logic
        self.process_completed();

        // Clear old events
        if self.event_queue.len() > 100 {
            self.event_queue.drain(0..50);
        }

        self.tick += 1;
        self.last_update = SystemTime::now();

        Ok(())
    }

    fn status(&self) -> ComponentStatus {
        ComponentStatus {
            name: "GameEngine".to_string(),
            healthy: true,
            last_update: self.last_update,
            metrics: vec![
                ("tick".to_string(), self.tick as f64),
                ("players".to_string(), self.players.len() as f64),
                ("active_sessions".to_string(), self.active_sessions.len() as f64),
                ("events".to_string(), self.event_queue.len() as f64),
            ],
        }
    }

    fn reset(&mut self) -> EngineResult<()> {
        self.process_engine.reset()?;
        self.hardware_engine.reset()?;
        self.software_engine.reset()?;
        self.network_engine.reset()?;

        self.players.clear();
        self.active_sessions.clear();
        self.event_queue.clear();
        self.tick = 0;
        self.last_update = SystemTime::now();

        Ok(())
    }
}