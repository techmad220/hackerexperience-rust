// HackerExperience Core - 1:1 Rust port of legacy PHP classes

pub mod entities;
pub mod error;
pub mod types;

// Re-export main types for convenience
pub use entities::*;
pub use error::*;
pub use types::*;

// Version constants matching the original
pub const VERSION: &str = "0.8";
pub const VERSION_STATUS: &str = " BETA";
pub const GAME_TITLE: &str = "Hacker Experience 0.8 BETA";
