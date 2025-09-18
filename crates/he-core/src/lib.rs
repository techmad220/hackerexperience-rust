// HackerExperience Core - 1:1 Rust port of legacy PHP classes

pub mod entities;
pub mod error;
pub mod types;
pub mod id;
pub mod entity_core;

// Infrastructure modules (Priority 1)
pub mod database;
pub mod security;
pub mod validation;
pub mod pagination;
pub mod cursor_pagination;

// Utility modules (Priority 3)
pub mod utils;

// External integration modules (Priority 4)
pub mod external;

// Re-export main types for convenience
pub use entities::*;
pub use error::*;
pub use types::*;
pub use id::*;
pub use entity_core::*;

// Re-export infrastructure modules
pub use database::*;
pub use security::*;
pub use validation::*;
pub use pagination::*;

// Re-export utility modules
pub use utils::*;

// Re-export external modules
pub use external::*;

// Version constants matching the original
pub const VERSION: &str = "0.8";
pub const VERSION_STATUS: &str = " BETA";
pub const GAME_TITLE: &str = "Hacker Experience 0.8 BETA";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_constants() {
        assert_eq!(VERSION, "0.8");
        assert_eq!(VERSION_STATUS, " BETA");
        assert_eq!(GAME_TITLE, "Hacker Experience 0.8 BETA");
    }
}
