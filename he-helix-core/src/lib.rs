//! # Helix Core Infrastructure
//!
//! This crate provides the foundational infrastructure for the Helix Rust migration.
//! It includes common types, traits, and utilities that are shared across all other crates.

pub mod actors;
pub mod distributed;
pub mod error;
pub mod events;
pub mod genserver;
pub mod hot_reload;
pub mod listener;
pub mod process;
pub mod supervisor;
pub mod supervision;
pub mod types;
pub mod utils;

// Re-export commonly used types
pub use error::{HelixError, HelixResult};
pub use distributed::*;
pub use events::*;
pub use genserver::*;
pub use hot_reload::*;
pub use supervision::*;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}