//! World generation utilities

use crate::{GameWorld, NPCServer};
use rand::Rng;

/// Initialize the game world with content
pub fn initialize_world() -> GameWorld {
    let world = GameWorld::new();

    // The world is already populated in GameWorld::new()
    // This function exists for any additional initialization

    tracing::info!("Game world initialized with {} servers", world.servers.len());
    tracing::info!("Created {} corporations", world.corporations.len());
    tracing::info!("Software catalog has {} crackers", world.software_catalog.crackers.len());
    tracing::info!("Generated {} mission templates", world.mission_templates.len());

    world
}

/// Generate a player's starting environment
pub fn generate_starting_area() -> Vec<String> {
    // Return IPs of beginner-friendly servers
    vec![
        "1.2.3.4".to_string(),    // First Whois
        "10.0.0.1".to_string(),   // Easy home PC
        "10.0.0.2".to_string(),   // Another easy target
        "10.0.0.3".to_string(),   // Small business
        "172.16.0.1".to_string(), // Easy company
    ]
}

/// Generate random events for the world
pub fn generate_world_event() -> WorldEvent {
    let mut rng = rand::thread_rng();

    let events = vec![
        WorldEvent::ServerReset {
            ip: format!("10.0.0.{}", rng.gen_range(1..255)),
            reason: "Scheduled maintenance".to_string(),
        },
        WorldEvent::NewBounty {
            corporation: "MegaCorp".to_string(),
            target_ip: format!("192.168.1.{}", rng.gen_range(1..255)),
            reward: rng.gen_range(10000..100000),
        },
        WorldEvent::SecurityUpgrade {
            ip: format!("172.16.0.{}", rng.gen_range(1..255)),
            new_level: rng.gen_range(50..90),
        },
        WorldEvent::VirusOutbreak {
            affected_servers: rng.gen_range(5..20),
            virus_name: "NewWorm.exe".to_string(),
        },
        WorldEvent::LawEnforcementRaid {
            hacker_name: generate_random_hacker_name(),
            servers_seized: rng.gen_range(1..5),
        },
    ];

    events[rng.gen_range(0..events.len())].clone()
}

#[derive(Debug, Clone)]
pub enum WorldEvent {
    ServerReset {
        ip: String,
        reason: String,
    },
    NewBounty {
        corporation: String,
        target_ip: String,
        reward: i64,
    },
    SecurityUpgrade {
        ip: String,
        new_level: i32,
    },
    VirusOutbreak {
        affected_servers: i32,
        virus_name: String,
    },
    LawEnforcementRaid {
        hacker_name: String,
        servers_seized: i32,
    },
}

fn generate_random_hacker_name() -> String {
    let prefixes = vec!["Dark", "Shadow", "Cyber", "Neo", "Zero", "Phantom", "Ghost"];
    let suffixes = vec!["Knight", "Wolf", "Phoenix", "Dragon", "Hawk", "Storm", "Blade"];

    let mut rng = rand::thread_rng();
    format!("{}{}{}",
        prefixes[rng.gen_range(0..prefixes.len())],
        suffixes[rng.gen_range(0..suffixes.len())],
        rng.gen_range(1..999)
    )
}