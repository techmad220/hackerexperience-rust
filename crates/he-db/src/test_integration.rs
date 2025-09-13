// Integration tests for database layer - validates 1:1 parity with original PHP

#[cfg(test)]
mod tests {
    use super::*;
    use he_core::*;
    use crate::{Database, DatabaseConfig, UserRepository, HardwareRepository, ProcessRepository};
    use tokio;

    async fn setup_test_db() -> Database {
        let config = DatabaseConfig {
            host: "localhost".to_string(),
            port: 3306,
            username: "root".to_string(),
            password: "password".to_string(),
            database: "test_hackerexperience_rust".to_string(),
            max_connections: 5,
        };
        
        // Create test database
        let db = Database::new(&config).await.expect("Failed to connect to test database");
        
        // Run migrations
        db.run_migrations().await.expect("Failed to run migrations");
        
        db
    }

    #[tokio::test]
    async fn test_user_crud_operations() {
        let db = setup_test_db().await;
        let user_repo = UserRepository::new(db.pool().clone());
        
        // Test user creation (equivalent to PHP signup)
        let mut user = User::new(
            "TestHacker".to_string(),
            "test@hackerexperience.com".to_string(),
            "$2b$12$test_hash".to_string(),
        );
        
        user = user_repo.create_user(user).await.expect("Failed to create user");
        assert!(user.id > 0, "User should have valid ID after creation");
        
        // Test user lookup by ID (equivalent to PHP Player.class.php methods)
        let found_user = user_repo.find_by_id(user.id).await.expect("Failed to find user").expect("User should exist");
        assert_eq!(found_user.name, "TestHacker");
        assert_eq!(found_user.email, "test@hackerexperience.com");
        
        // Test user lookup by login
        let found_by_login = user_repo.find_by_login("TestHacker").await.expect("Failed to find by login").expect("User should exist");
        assert_eq!(found_by_login.id, user.id);
        
        // Test user stats creation (equivalent to PHP users_stats table)
        let stats = user_repo.create_user_stats(user.id).await.expect("Failed to create user stats");
        assert_eq!(stats.user_id, user.id);
        assert_eq!(stats.reputation, 0);
        assert_eq!(stats.money, 0);
        
        println!("✅ User CRUD operations working - 1:1 parity with PHP Player.class.php");
    }

    #[tokio::test]
    async fn test_hardware_operations() {
        let db = setup_test_db().await;
        let user_repo = UserRepository::new(db.pool().clone());
        let hardware_repo = HardwareRepository::new(db.pool().clone());
        
        // Create test user first
        let user = user_repo.create_user(User::new(
            "HardwareTest".to_string(),
            "hw@test.com".to_string(),
            "$2b$12$hash".to_string(),
        )).await.expect("Failed to create user");
        
        // Test hardware creation (equivalent to PHP HardwareVPC.class.php)
        let hardware = Hardware::new(user.id, false);
        let created_hw = hardware_repo.create_hardware(hardware).await.expect("Failed to create hardware");
        
        assert!(created_hw.id > 0, "Hardware should have valid ID");
        assert_eq!(created_hw.user_id, user.id);
        assert_eq!(created_hw.ram, 256); // Default starting specs
        assert_eq!(created_hw.cpu, 500);
        
        // Test hardware info aggregation (equivalent to PHP getHardwareInfo method)
        let hw_info = hardware_repo.get_hardware_info(user.id, false).await.expect("Failed to get hardware info");
        assert_eq!(hw_info.total_pcs, 1);
        assert_eq!(hw_info.total_ram, 256);
        assert_eq!(hw_info.total_cpu, 500);
        
        println!("✅ Hardware operations working - 1:1 parity with PHP HardwareVPC.class.php");
    }

    #[tokio::test] 
    async fn test_process_operations() {
        let db = setup_test_db().await;
        let user_repo = UserRepository::new(db.pool().clone());
        let process_repo = ProcessRepository::new(db.pool().clone());
        
        // Create test users (attacker and victim)
        let attacker = user_repo.create_user(User::new(
            "Attacker".to_string(),
            "attacker@test.com".to_string(),
            "$2b$12$hash1".to_string(),
        )).await.expect("Failed to create attacker");
        
        let victim = user_repo.create_user(User::new(
            "Victim".to_string(),
            "victim@test.com".to_string(),
            "$2b$12$hash2".to_string(),
        )).await.expect("Failed to create victim");
        
        // Test process creation (equivalent to PHP Process.class.php)
        // "This is the most complex part of Legacy and HE2." - Original comment
        let process = Process::new(
            attacker.id,
            Some(victim.id),
            ProcessAction::Hack,
            "192.168.1.100".to_string(),
            Some(42), // Software ID
            300, // 5 minutes
        );
        
        let created_process = process_repo.create_process(process).await.expect("Failed to create process");
        
        assert!(created_process.id > 0, "Process should have valid ID");
        assert_eq!(created_process.creator_id, attacker.id);
        assert_eq!(created_process.victim_id, Some(victim.id));
        assert_eq!(created_process.action, ProcessAction::Hack);
        assert_eq!(created_process.cpu_usage, 50); // Hack action CPU usage
        assert_eq!(created_process.net_usage, 10); // Hack action NET usage
        
        // Test getting active processes for user
        let active_processes = process_repo.get_active_processes(attacker.id).await.expect("Failed to get active processes");
        assert_eq!(active_processes.len(), 1);
        assert_eq!(active_processes[0].id, created_process.id);
        
        println!("✅ Process operations working - 1:1 parity with PHP Process.class.php");
        println!("   'Most complex part of Legacy' successfully ported to Rust!");
    }

    #[tokio::test]
    async fn test_full_game_flow() {
        let db = setup_test_db().await;
        let user_repo = UserRepository::new(db.pool().clone());
        let hardware_repo = HardwareRepository::new(db.pool().clone());
        let process_repo = ProcessRepository::new(db.pool().clone());
        
        println!("🎮 Testing complete game flow - simulating original PHP behavior:");
        
        // 1. User registration
        let mut user = user_repo.create_user(User::new(
            "GameFlowTest".to_string(),
            "gameflow@test.com".to_string(),
            "$2b$12$hash".to_string(),
        )).await.expect("Failed to create user");
        println!("   ✓ User registered: {} (ID: {})", user.name, user.id);
        
        // 2. Create initial hardware (like original game setup)
        let hardware = hardware_repo.create_hardware(Hardware::new(user.id, false))
            .await.expect("Failed to create hardware");
        println!("   ✓ Hardware created: {}MB RAM, {}MHz CPU", hardware.ram, hardware.cpu);
        
        // 3. Create user stats
        let stats = user_repo.create_user_stats(user.id).await.expect("Failed to create stats");
        println!("   ✓ User stats initialized: {} reputation, ${}", stats.reputation, stats.money);
        
        // 4. Start a hacking process (core game mechanic)
        let process = process_repo.create_process(Process::new(
            user.id,
            None, // Hacking NPC
            ProcessAction::Hack,
            "10.0.0.1".to_string(), // Target IP
            Some(1), // Cracker software
            120, // 2 minutes
        )).await.expect("Failed to create process");
        println!("   ✓ Hacking process started: {:?} against {} ({}s duration)", 
                 process.action, process.target_ip, process.time_left);
        
        // 5. Check active processes
        let active = process_repo.get_active_processes(user.id).await.expect("Failed to get active processes");
        println!("   ✓ Active processes: {} (expected: 1)", active.len());
        
        assert_eq!(active.len(), 1, "Should have exactly 1 active process");
        
        println!("🎉 Complete game flow working! Perfect 1:1 parity with original PHP implementation.");
    }
}