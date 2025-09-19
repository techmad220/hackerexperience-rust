// Database infrastructure classes (Priority 1)

pub mod database;
pub mod pdo;

// Re-export for convenience
pub use database::*;
pub use pdo::*;