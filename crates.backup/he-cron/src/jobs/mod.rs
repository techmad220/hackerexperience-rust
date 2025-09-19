//! Cron job implementations
//! 
//! This module contains all the individual cron jobs ported from PHP to async Rust.
//! Each job maintains the exact same business logic as the original PHP implementation
//! while using modern async/await patterns and proper error handling.

pub mod backup_forum;
pub mod backup_game;
pub mod restore_software;
pub mod update_server_stats;
pub mod end_war;
pub mod generate_missions;
pub mod defcon;
pub mod update_premium;
pub mod safenet_update;
pub mod doom_updater;
pub mod finish_round;

// Re-export all job modules for easier access
pub use backup_forum::*;
pub use backup_game::*;
pub use restore_software::*;
pub use update_server_stats::*;
pub use end_war::*;
pub use generate_missions::*;
pub use defcon::*;
pub use update_premium::*;
pub use safenet_update::*;
pub use doom_updater::*;
pub use finish_round::*;