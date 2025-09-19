//! Config constants - 1:1 port of config.php
//! 
//! Contains global game configuration constants and process definitions.
//! These are the core constants that define game mechanics and timing.

use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Wiki path configuration
pub const WIKI_PATH: &str = "http://localhost/wiki/";

/// Game version and title information
pub const VERSION: &str = "0.8";
pub const VERSION_STATUS: &str = " BETA";

/// Get the full game title
pub fn game_title() -> String {
    format!("Hacker Experience {}{}", VERSION, VERSION_STATUS)
}

/// Process action constants - maps action names to numeric IDs
pub static PROCESS_ACTIONS: Lazy<HashMap<&'static str, u32>> = Lazy::new(|| {
    let mut actions = HashMap::new();
    
    actions.insert("DOWNLOAD", 1);
    actions.insert("UPLOAD", 2);
    actions.insert("DELETE", 3);
    actions.insert("HIDE", 4);
    actions.insert("SEEK", 5);
    actions.insert("COLLECT", 6);  // DEPRECATED
    actions.insert("AV", 7);
    actions.insert("E_LOG", 8);
    actions.insert("D_LOG", 9);  // DEPRECATED
    actions.insert("FORMAT", 10);
    actions.insert("HACK", 11);
    actions.insert("BANK_HACK", 12);
    actions.insert("INSTALL", 13);
    actions.insert("UNINSTALL", 14);
    actions.insert("PORT_SCAN", 15);
    actions.insert("HACK_XP", 16);
    actions.insert("RESEARCH", 17);
    actions.insert("UPLOAD_XHD", 18);
    actions.insert("DOWNLOAD_XHD", 19);
    actions.insert("DELETE_XHD", 20);
    actions.insert("NMAP", 22);
    actions.insert("ANALYZE", 23);
    actions.insert("INSTALL_DOOM", 24);
    actions.insert("RESET_IP", 25);
    actions.insert("RESET_PWD", 26);
    actions.insert("DDOS", 27);
    actions.insert("INSTALL_WEBSERVER", 28);
    
    actions
});

/// Process timing configuration - defines min/max execution times in seconds
pub static PROCESS_TIME_CONFIG: Lazy<HashMap<&'static str, u32>> = Lazy::new(|| {
    let mut config = HashMap::new();
    
    config.insert("DOWNLOAD_MIN", 20);
    config.insert("DOWNLOAD_MAX", 7200);
    config.insert("UPLOAD_MIN", 20);
    config.insert("UPLOAD_MAX", 7200);
    config.insert("DELETE_MIN", 20);  // hide must be faster than delete*
    config.insert("DELETE_MAX", 1200);
    config.insert("HIDE_MIN", 5);
    config.insert("HIDE_MAX", 1200);
    config.insert("SEEK_MIN", 5);
    config.insert("SEEK_MAX", 1200);
    config.insert("INSTALL_MIN", 4);
    config.insert("INSTALL_MAX", 1200);
    config.insert("AV_MIN", 60);
    config.insert("AV_MAX", 600);
    config.insert("LOG_MIN", 4);
    config.insert("LOG_MAX", 60);
    config.insert("FORMAT_MIN", 1200);
    config.insert("FORMAT_MAX", 3600);
    
    config
});

/// Helper functions for accessing configuration values

/// Get process action ID by name
pub fn get_process_action(action_name: &str) -> Option<u32> {
    PROCESS_ACTIONS.get(action_name).copied()
}

/// Get process timing configuration value
pub fn get_process_time_config(config_key: &str) -> Option<u32> {
    PROCESS_TIME_CONFIG.get(config_key).copied()
}

/// Get minimum time for a process action
pub fn get_process_min_time(action: &str) -> Option<u32> {
    let key = format!("{}_MIN", action);
    get_process_time_config(&key)
}

/// Get maximum time for a process action
pub fn get_process_max_time(action: &str) -> Option<u32> {
    let key = format!("{}_MAX", action);
    get_process_time_config(&key)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_game_title_generation() {
        let title = game_title();
        assert_eq!(title, "Hacker Experience 0.8 BETA");
    }
    
    #[test]
    fn test_process_actions_lookup() {
        assert_eq!(get_process_action("DOWNLOAD"), Some(1));
        assert_eq!(get_process_action("HACK"), Some(11));
        assert_eq!(get_process_action("NONEXISTENT"), None);
    }
    
    #[test]
    fn test_process_time_config() {
        assert_eq!(get_process_time_config("DOWNLOAD_MIN"), Some(20));
        assert_eq!(get_process_time_config("DOWNLOAD_MAX"), Some(7200));
        assert_eq!(get_process_time_config("INVALID_KEY"), None);
    }
    
    #[test]
    fn test_process_time_helpers() {
        assert_eq!(get_process_min_time("DOWNLOAD"), Some(20));
        assert_eq!(get_process_max_time("DOWNLOAD"), Some(7200));
        assert_eq!(get_process_min_time("INVALID"), None);
    }
    
    #[test]
    fn test_deprecated_actions_still_work() {
        // Ensure deprecated actions are still accessible
        assert_eq!(get_process_action("COLLECT"), Some(6));
        assert_eq!(get_process_action("D_LOG"), Some(9));
    }
}