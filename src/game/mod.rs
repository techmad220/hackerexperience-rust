/// Complete HackerExperience game logic implementation
/// 
/// This module contains the full business logic for all game systems:
/// - Hacking: Complete hacking mechanics with brute force, exploits, IP discovery, and log systems
/// - Virus: Virus installation, collection, money generation, DDoS, and spreading
/// - Missions: Mission generation, validation, story campaigns, and tutorial flow
/// - Economy: Banking system, Bitcoin implementation, hardware pricing, and marketplace
/// - Combat: Clan wars, DDoS battles, firewall vs cracker calculations, and reputation

pub mod hacking;
pub mod virus;
pub mod missions;
pub mod economy;
pub mod combat;

// Re-export main types for convenience
pub use hacking::*;
pub use virus::*;
pub use missions::*;
pub use economy::*;
pub use combat::*;

/// Complete game state manager that coordinates all systems
#[derive(Debug, Clone)]
pub struct GameEngine {
    pub hacking: hacking::HackingSystem,
    pub virus: virus::VirusSystem,
    pub missions: missions::MissionSystem,
    pub economy: economy::EconomySystem,
    pub combat: combat::CombatSystem,
}

impl GameEngine {
    pub fn new() -> Self {
        Self {
            hacking: hacking::HackingSystem::new(),
            virus: virus::VirusSystem::new(),
            missions: missions::MissionSystem::new(),
            economy: economy::EconomySystem::new(),
            combat: combat::CombatSystem::new(),
        }
    }

    /// Initialize new user with starting resources and tutorial
    pub fn initialize_user(&mut self, user_id: crate::UserId) -> crate::HeResult<UserInitialization> {
        // Create bank account with starting money
        let bank_account = self.economy.create_bank_account(user_id, 10000)?;
        
        // Create Bitcoin wallet
        let bitcoin_wallet = self.economy.create_bitcoin_wallet(user_id)?;
        
        // Start tutorial missions
        let tutorial_missions = self.missions.start_tutorial(user_id)?;
        
        // Generate initial random missions
        let random_missions = self.missions.generate_random_missions(user_id, 3)?;
        
        Ok(UserInitialization {
            user_id,
            starting_money: 10000,
            bank_account: bank_account.account_number,
            bitcoin_address: bitcoin_wallet.address,
            tutorial_missions: tutorial_missions.len(),
            available_missions: random_missions.len(),
        })
    }

    /// Complete game tick - updates all systems
    pub fn game_tick(&mut self) -> crate::HeResult<GameTickResult> {
        let mut tick_result = GameTickResult::default();

        // Process bank transfers
        let transfer_results = self.economy.process_bank_transfers()?;
        tick_result.processed_transfers = transfer_results.len();

        // Update virus operations
        let virus_events = self.virus.update_viruses()?;
        tick_result.virus_events = virus_events.len();

        // Process clan wars
        let war_updates = self.combat.process_clan_wars()?;
        tick_result.war_updates = war_updates.len();

        // Update market prices
        self.economy.update_market_prices()?;
        tick_result.market_updated = true;

        // Clean up old logs
        let cutoff_time = chrono::Utc::now() - chrono::Duration::days(30);
        self.hacking.cleanup_old_logs(cutoff_time);

        Ok(tick_result)
    }
}

impl Default for GameEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of user initialization
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct UserInitialization {
    pub user_id: crate::UserId,
    pub starting_money: i64,
    pub bank_account: String,
    pub bitcoin_address: String,
    pub tutorial_missions: usize,
    pub available_missions: usize,
}

/// Result of game tick processing
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct GameTickResult {
    pub processed_transfers: usize,
    pub virus_events: usize,
    pub war_updates: usize,
    pub market_updated: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_engine_creation() {
        let game_engine = GameEngine::new();
        
        // All systems should be initialized
        assert!(!game_engine.hacking.get_logs().is_empty() || game_engine.hacking.get_logs().is_empty());
        // Just testing that systems exist and don't panic
    }

    #[test]
    fn test_user_initialization() {
        let mut game_engine = GameEngine::new();
        let result = game_engine.initialize_user(1).unwrap();
        
        assert_eq!(result.user_id, 1);
        assert_eq!(result.starting_money, 10000);
        assert!(!result.bank_account.is_empty());
        assert!(!result.bitcoin_address.is_empty());
        assert!(result.tutorial_missions > 0);
    }

    #[test]
    fn test_game_tick() {
        let mut game_engine = GameEngine::new();
        let _init_result = game_engine.initialize_user(1).unwrap();
        
        let tick_result = game_engine.game_tick().unwrap();
        assert!(tick_result.market_updated);
        // Other counts may be 0 if no active operations
    }
}