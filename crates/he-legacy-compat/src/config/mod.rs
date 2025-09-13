//! Configuration module - 1:1 port of config.php
//! 
//! Original: Contains game version, title, and process configuration constants
//! Features:
//! - Game version and title configuration
//! - Process action type definitions  
//! - Process timing configuration constants
//! - Static arrays converted to Rust constants and HashMap structures
//! 
//! This module maintains exact functional parity with the original PHP config.php,
//! providing the same constants and data structures used throughout the game.

use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Wiki path configuration - equivalent to $wikiPath
pub const WIKI_PATH: &str = "http://localhost/wiki/";

/// Game version - equivalent to $version
pub const VERSION: &str = "0.8";

/// Version status - equivalent to $versionStatus  
pub const VERSION_STATUS: &str = " BETA";

/// Game title - equivalent to $gameTitle
pub const GAME_TITLE: &str = "Hacker Experience 0.8 BETA";

/// Process actions - 1:1 port of static $processActions array
/// Each action represents a specific game process type
pub mod process_actions {
    pub const DOWNLOAD: &str = "1";
    pub const UPLOAD: &str = "2";
    pub const DELETE: &str = "3";
    pub const HIDE: &str = "4";
    pub const SEEK: &str = "5";
    pub const COLLECT: &str = "6";        // DEPRECATED
    pub const AV: &str = "7";
    pub const E_LOG: &str = "8";
    pub const D_LOG: &str = "9";          // DEPRECATED
    pub const FORMAT: &str = "10";
    pub const HACK: &str = "11";
    pub const BANK_HACK: &str = "12";
    pub const INSTALL: &str = "13";
    pub const UNINSTALL: &str = "14";
    pub const PORT_SCAN: &str = "15";
    pub const HACK_XP: &str = "16";
    pub const RESEARCH: &str = "17";
    pub const UPLOAD_XHD: &str = "18";
    pub const DOWNLOAD_XHD: &str = "19";
    pub const DELETE_XHD: &str = "20";
    pub const NMAP: &str = "22";
    pub const ANALYZE: &str = "23";
    pub const INSTALL_DOOM: &str = "24";
    pub const RESET_IP: &str = "25";
    pub const RESET_PWD: &str = "26";
    pub const DDOS: &str = "27";
    pub const INSTALL_WEBSERVER: &str = "28";
}

/// Process actions as HashMap - equivalent to static $processActions array
pub static PROCESS_ACTIONS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut actions = HashMap::new();
    actions.insert("DOWNLOAD", process_actions::DOWNLOAD);
    actions.insert("UPLOAD", process_actions::UPLOAD);
    actions.insert("DELETE", process_actions::DELETE);
    actions.insert("HIDE", process_actions::HIDE);
    actions.insert("SEEK", process_actions::SEEK);
    actions.insert("COLLECT", process_actions::COLLECT);
    actions.insert("AV", process_actions::AV);
    actions.insert("E_LOG", process_actions::E_LOG);
    actions.insert("D_LOG", process_actions::D_LOG);
    actions.insert("FORMAT", process_actions::FORMAT);
    actions.insert("HACK", process_actions::HACK);
    actions.insert("BANK_HACK", process_actions::BANK_HACK);
    actions.insert("INSTALL", process_actions::INSTALL);
    actions.insert("UNINSTALL", process_actions::UNINSTALL);
    actions.insert("PORT_SCAN", process_actions::PORT_SCAN);
    actions.insert("HACK_XP", process_actions::HACK_XP);
    actions.insert("RESEARCH", process_actions::RESEARCH);
    actions.insert("UPLOAD_XHD", process_actions::UPLOAD_XHD);
    actions.insert("DOWNLOAD_XHD", process_actions::DOWNLOAD_XHD);
    actions.insert("DELETE_XHD", process_actions::DELETE_XHD);
    actions.insert("NMAP", process_actions::NMAP);
    actions.insert("ANALYZE", process_actions::ANALYZE);
    actions.insert("INSTALL_DOOM", process_actions::INSTALL_DOOM);
    actions.insert("RESET_IP", process_actions::RESET_IP);
    actions.insert("RESET_PWD", process_actions::RESET_PWD);
    actions.insert("DDOS", process_actions::DDOS);
    actions.insert("INSTALL_WEBSERVER", process_actions::INSTALL_WEBSERVER);
    actions
});

/// Process time configuration - 1:1 port of static $processTimeConfig array
/// All timing values are in seconds
pub mod process_time_config {
    pub const DOWNLOAD_MIN: &str = "20";
    pub const DOWNLOAD_MAX: &str = "7200";
    pub const UPLOAD_MIN: &str = "20";
    pub const UPLOAD_MAX: &str = "7200";
    pub const DELETE_MIN: &str = "20";        // hide must be faster than delete
    pub const DELETE_MAX: &str = "1200";
    pub const HIDE_MIN: &str = "5";
    pub const HIDE_MAX: &str = "1200";
    pub const SEEK_MIN: &str = "5";
    pub const SEEK_MAX: &str = "1200";
    pub const INSTALL_MIN: &str = "4";
    pub const INSTALL_MAX: &str = "1200";
    pub const AV_MIN: &str = "60";
    pub const AV_MAX: &str = "600";
    pub const LOG_MIN: &str = "4";
    pub const LOG_MAX: &str = "60";
    pub const FORMAT_MIN: &str = "1200";
    pub const FORMAT_MAX: &str = "3600";
}

/// Process time configuration as HashMap - equivalent to static $processTimeConfig array
pub static PROCESS_TIME_CONFIG: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut config = HashMap::new();
    config.insert("DOWNLOAD_MIN", process_time_config::DOWNLOAD_MIN);
    config.insert("DOWNLOAD_MAX", process_time_config::DOWNLOAD_MAX);
    config.insert("UPLOAD_MIN", process_time_config::UPLOAD_MIN);
    config.insert("UPLOAD_MAX", process_time_config::UPLOAD_MAX);
    config.insert("DELETE_MIN", process_time_config::DELETE_MIN);
    config.insert("DELETE_MAX", process_time_config::DELETE_MAX);
    config.insert("HIDE_MIN", process_time_config::HIDE_MIN);
    config.insert("HIDE_MAX", process_time_config::HIDE_MAX);
    config.insert("SEEK_MIN", process_time_config::SEEK_MIN);
    config.insert("SEEK_MAX", process_time_config::SEEK_MAX);
    config.insert("INSTALL_MIN", process_time_config::INSTALL_MIN);
    config.insert("INSTALL_MAX", process_time_config::INSTALL_MAX);
    config.insert("AV_MIN", process_time_config::AV_MIN);
    config.insert("AV_MAX", process_time_config::AV_MAX);
    config.insert("LOG_MIN", process_time_config::LOG_MIN);
    config.insert("LOG_MAX", process_time_config::LOG_MAX);
    config.insert("FORMAT_MIN", process_time_config::FORMAT_MIN);
    config.insert("FORMAT_MAX", process_time_config::FORMAT_MAX);
    config
});

/// Utility functions for working with process actions
impl ProcessActions {
    /// Get process action ID by name
    pub fn get_action_id(action_name: &str) -> Option<&'static str> {
        PROCESS_ACTIONS.get(action_name).copied()
    }
    
    /// Get all process action names
    pub fn get_all_action_names() -> Vec<&'static str> {
        PROCESS_ACTIONS.keys().copied().collect()
    }
    
    /// Check if action exists
    pub fn action_exists(action_name: &str) -> bool {
        PROCESS_ACTIONS.contains_key(action_name)
    }
    
    /// Check if action is deprecated
    pub fn is_deprecated(action_name: &str) -> bool {
        matches!(action_name, "COLLECT" | "D_LOG")
    }
}

/// Utility struct for process actions
pub struct ProcessActions;

/// Utility functions for working with process time configuration
impl ProcessTimeConfig {
    /// Get time configuration value by key
    pub fn get_time_value(key: &str) -> Option<&'static str> {
        PROCESS_TIME_CONFIG.get(key).copied()
    }
    
    /// Get minimum time for an action type
    pub fn get_min_time(action_type: &str) -> Option<u32> {
        let key = format!("{}_MIN", action_type.to_uppercase());
        PROCESS_TIME_CONFIG.get(key.as_str())
            .and_then(|v| v.parse().ok())
    }
    
    /// Get maximum time for an action type  
    pub fn get_max_time(action_type: &str) -> Option<u32> {
        let key = format!("{}_MAX", action_type.to_uppercase());
        PROCESS_TIME_CONFIG.get(key.as_str())
            .and_then(|v| v.parse().ok())
    }
    
    /// Get time range for an action type (min, max)
    pub fn get_time_range(action_type: &str) -> Option<(u32, u32)> {
        let min = Self::get_min_time(action_type)?;
        let max = Self::get_max_time(action_type)?;
        Some((min, max))
    }
    
    /// Get all time configuration keys
    pub fn get_all_keys() -> Vec<&'static str> {
        PROCESS_TIME_CONFIG.keys().copied().collect()
    }
}

/// Utility struct for process time configuration
pub struct ProcessTimeConfig;

/// Get the full game title (version + status)
pub fn get_game_title() -> String {
    format!("Hacker Experience {}{}", VERSION, VERSION_STATUS)
}

/// Get version information as a tuple (version, status)
pub fn get_version_info() -> (&'static str, &'static str) {
    (VERSION, VERSION_STATUS)
}

/// Check if we're running in beta mode
pub fn is_beta() -> bool {
    VERSION_STATUS.contains("BETA")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version_constants() {
        assert_eq!(VERSION, "0.8");
        assert_eq!(VERSION_STATUS, " BETA");
        assert_eq!(GAME_TITLE, "Hacker Experience 0.8 BETA");
        assert_eq!(WIKI_PATH, "http://localhost/wiki/");
    }
    
    #[test]
    fn test_game_title_generation() {
        let title = get_game_title();
        assert_eq!(title, "Hacker Experience 0.8 BETA");
        
        let (version, status) = get_version_info();
        assert_eq!(version, "0.8");
        assert_eq!(status, " BETA");
        
        assert!(is_beta());
    }
    
    #[test]
    fn test_process_actions_constants() {
        // Test specific action values match original PHP
        assert_eq!(process_actions::DOWNLOAD, "1");
        assert_eq!(process_actions::UPLOAD, "2");
        assert_eq!(process_actions::DELETE, "3");
        assert_eq!(process_actions::HACK, "11");
        assert_eq!(process_actions::DDOS, "27");
        assert_eq!(process_actions::INSTALL_WEBSERVER, "28");
        
        // Test deprecated actions
        assert_eq!(process_actions::COLLECT, "6");
        assert_eq!(process_actions::D_LOG, "9");
    }
    
    #[test]
    fn test_process_actions_hashmap() {
        let actions = &*PROCESS_ACTIONS;
        
        // Test that all expected actions exist
        assert_eq!(actions.get("DOWNLOAD").unwrap(), &"1");
        assert_eq!(actions.get("UPLOAD").unwrap(), &"2");
        assert_eq!(actions.get("DELETE").unwrap(), &"3");
        assert_eq!(actions.get("DDOS").unwrap(), &"27");
        
        // Test utility functions
        assert_eq!(ProcessActions::get_action_id("DOWNLOAD"), Some("1"));
        assert_eq!(ProcessActions::get_action_id("NONEXISTENT"), None);
        
        assert!(ProcessActions::action_exists("DOWNLOAD"));
        assert!(!ProcessActions::action_exists("NONEXISTENT"));
        
        assert!(ProcessActions::is_deprecated("COLLECT"));
        assert!(ProcessActions::is_deprecated("D_LOG"));
        assert!(!ProcessActions::is_deprecated("DOWNLOAD"));
        
        let all_names = ProcessActions::get_all_action_names();
        assert!(all_names.contains(&"DOWNLOAD"));
        assert!(all_names.contains(&"DDOS"));
    }
    
    #[test]
    fn test_process_time_config_constants() {
        // Test specific timing values match original PHP
        assert_eq!(process_time_config::DOWNLOAD_MIN, "20");
        assert_eq!(process_time_config::DOWNLOAD_MAX, "7200");
        assert_eq!(process_time_config::HIDE_MIN, "5");
        assert_eq!(process_time_config::FORMAT_MIN, "1200");
        assert_eq!(process_time_config::FORMAT_MAX, "3600");
    }
    
    #[test]
    fn test_process_time_config_hashmap() {
        let config = &*PROCESS_TIME_CONFIG;
        
        // Test that all expected config exists
        assert_eq!(config.get("DOWNLOAD_MIN").unwrap(), &"20");
        assert_eq!(config.get("DOWNLOAD_MAX").unwrap(), &"7200");
        assert_eq!(config.get("FORMAT_MIN").unwrap(), &"1200");
        
        // Test utility functions
        assert_eq!(ProcessTimeConfig::get_time_value("DOWNLOAD_MIN"), Some("20"));
        assert_eq!(ProcessTimeConfig::get_time_value("NONEXISTENT"), None);
        
        assert_eq!(ProcessTimeConfig::get_min_time("DOWNLOAD"), Some(20));
        assert_eq!(ProcessTimeConfig::get_max_time("DOWNLOAD"), Some(7200));
        assert_eq!(ProcessTimeConfig::get_min_time("NONEXISTENT"), None);
        
        let range = ProcessTimeConfig::get_time_range("DOWNLOAD");
        assert_eq!(range, Some((20, 7200)));
        
        let range_invalid = ProcessTimeConfig::get_time_range("NONEXISTENT");
        assert_eq!(range_invalid, None);
        
        let all_keys = ProcessTimeConfig::get_all_keys();
        assert!(all_keys.contains(&"DOWNLOAD_MIN"));
        assert!(all_keys.contains(&"FORMAT_MAX"));
    }
    
    #[test]
    fn test_time_constraints() {
        // Test that hide is faster than delete (as noted in original comment)
        let hide_min = ProcessTimeConfig::get_min_time("HIDE").unwrap();
        let delete_min = ProcessTimeConfig::get_min_time("DELETE").unwrap();
        assert!(hide_min < delete_min);
        
        // Test reasonable timing ranges
        let (download_min, download_max) = ProcessTimeConfig::get_time_range("DOWNLOAD").unwrap();
        assert!(download_min < download_max);
        assert_eq!(download_min, 20);
        assert_eq!(download_max, 7200);
        
        let (format_min, format_max) = ProcessTimeConfig::get_time_range("FORMAT").unwrap();
        assert!(format_min < format_max);
        assert_eq!(format_min, 1200);
        assert_eq!(format_max, 3600);
    }
}