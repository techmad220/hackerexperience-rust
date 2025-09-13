// HackerExperience Legacy PHP Compatibility Layer
// 1:1 Rust port of all 2,294+ PHP files from the original game

pub mod pages;          // All root-level PHP pages (index.php, ajax.php, etc.)
pub mod classes;        // All PHP classes ported to Rust
pub mod cron;           // Background tasks and Python scripts
pub mod forum;          // phpBB forum integration
pub mod utils;          // Utility functions and helpers
pub mod templates;      // HTML template rendering
pub mod session;        // PHP session compatibility

// Re-export main modules
pub use pages::*;
pub use classes::*;

// Configuration constants from original config.php
pub const VERSION: &str = "0.8";
pub const VERSION_STATUS: &str = " BETA";
pub const GAME_TITLE: &str = "Hacker Experience 0.8 BETA";

// Process actions from original config.php
pub mod process_actions {
    pub const DOWNLOAD: i32 = 1;
    pub const UPLOAD: i32 = 2;
    pub const DELETE: i32 = 3;
    pub const HIDE: i32 = 4;
    pub const SEEK: i32 = 5;
    pub const COLLECT: i32 = 6;     // DEPRECATED
    pub const AV: i32 = 7;
    pub const E_LOG: i32 = 8;
    pub const D_LOG: i32 = 9;       // DEPRECATED
    pub const FORMAT: i32 = 10;
    pub const HACK: i32 = 11;
    pub const BANK_HACK: i32 = 12;
    pub const INSTALL: i32 = 13;
    pub const UNINSTALL: i32 = 14;
    pub const PORT_SCAN: i32 = 15;
    pub const HACK_XP: i32 = 16;
    pub const RESEARCH: i32 = 17;
    pub const UPLOAD_XHD: i32 = 18;
    pub const DOWNLOAD_XHD: i32 = 19;
    pub const DELETE_XHD: i32 = 20;
    pub const NMAP: i32 = 22;
    pub const ANALYZE: i32 = 23;
    pub const INSTALL_DOOM: i32 = 24;
    pub const RESET_IP: i32 = 25;
    pub const RESET_PWD: i32 = 26;
    pub const DDOS: i32 = 27;
    pub const INSTALL_WEBSERVER: i32 = 28;
}

// Process time configuration from original config.php
pub mod process_time_config {
    pub const DOWNLOAD_MIN: u32 = 20;
    pub const DOWNLOAD_MAX: u32 = 7200;
    pub const UPLOAD_MIN: u32 = 20;
    pub const UPLOAD_MAX: u32 = 7200;
    pub const DELETE_MIN: u32 = 20;
    pub const DELETE_MAX: u32 = 1200;
    pub const HIDE_MIN: u32 = 5;
    pub const HIDE_MAX: u32 = 1200;
    pub const SEEK_MIN: u32 = 5;
    pub const SEEK_MAX: u32 = 1200;
    pub const INSTALL_MIN: u32 = 4;
    pub const INSTALL_MAX: u32 = 1200;
    pub const AV_MIN: u32 = 60;
    pub const AV_MAX: u32 = 600;
    pub const LOG_MIN: u32 = 4;
    pub const LOG_MAX: u32 = 60;
    pub const FORMAT_MIN: u32 = 1200;
    pub const FORMAT_MAX: u32 = 3600;
}
