// HackerExperience Legacy PHP Compatibility Layer
// 1:1 Rust port of all 2,294+ PHP files from the original game

pub mod pages;          // All root-level PHP pages (index.php, ajax.php, etc.)
pub mod classes;        // All PHP classes ported to Rust
pub mod config;         // Configuration constants and settings
pub mod cron;           // Background tasks and Python scripts
pub mod forum;          // phpBB forum integration
pub mod utils;          // Utility functions and helpers
pub mod templates;      // HTML template rendering
pub mod session;        // PHP session compatibility

// Re-export main modules
pub use pages::*;
pub use classes::*;
pub use config::*;

// Configuration constants are now in the config module
