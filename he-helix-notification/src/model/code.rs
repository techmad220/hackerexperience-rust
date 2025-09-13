//! Notification code system for type-safe notification handling

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use once_cell::sync::Lazy;

use crate::{NotificationError, NotificationResult};

/// Notification code identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NotificationCode {
    pub name: String,
    pub enum_id: u32,
    pub class: super::NotificationClass,
}

impl NotificationCode {
    /// Create a new notification code
    pub fn new(name: impl Into<String>, enum_id: u32, class: super::NotificationClass) -> Self {
        Self {
            name: name.into(),
            enum_id,
            class,
        }
    }

    /// Get code as string
    pub fn as_str(&self) -> &str {
        &self.name
    }
}

impl fmt::Display for NotificationCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Registry for notification codes
pub struct CodeRegistry {
    codes: HashMap<(super::NotificationClass, String), NotificationCode>,
}

impl CodeRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            codes: HashMap::new(),
        }
    }

    /// Register a notification code
    pub fn register_code(&mut self, code: NotificationCode) {
        let key = (code.class, code.name.clone());
        self.codes.insert(key, code);
    }

    /// Check if a code exists for the given class
    pub fn code_exists(&self, class: super::NotificationClass, name: &str) -> bool {
        self.codes.contains_key(&(class, name.to_string()))
    }

    /// Get a code by class and name
    pub fn get_code(&self, class: super::NotificationClass, name: &str) -> Option<&NotificationCode> {
        self.codes.get(&(class, name.to_string()))
    }

    /// Get all codes for a class
    pub fn get_codes_for_class(&self, class: super::NotificationClass) -> Vec<&NotificationCode> {
        self.codes
            .values()
            .filter(|code| code.class == class)
            .collect()
    }
}

impl Default for CodeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global code registry
static CODE_REGISTRY: Lazy<CodeRegistry> = Lazy::new(|| {
    let mut registry = CodeRegistry::new();
    
    // Server notification codes
    registry.register_code(NotificationCode::new(
        "connection_established", 1, super::NotificationClass::Server
    ));
    registry.register_code(NotificationCode::new(
        "connection_lost", 2, super::NotificationClass::Server
    ));
    registry.register_code(NotificationCode::new(
        "login_attempt", 3, super::NotificationClass::Server
    ));
    registry.register_code(NotificationCode::new(
        "firewall_activated", 4, super::NotificationClass::Server
    ));
    registry.register_code(NotificationCode::new(
        "process_completed", 5, super::NotificationClass::Server
    ));
    
    // Chat notification codes
    registry.register_code(NotificationCode::new(
        "new_message", 101, super::NotificationClass::Chat
    ));
    registry.register_code(NotificationCode::new(
        "user_joined", 102, super::NotificationClass::Chat
    ));
    registry.register_code(NotificationCode::new(
        "user_left", 103, super::NotificationClass::Chat
    ));
    
    // Entity notification codes
    registry.register_code(NotificationCode::new(
        "account_created", 201, super::NotificationClass::Entity
    ));
    registry.register_code(NotificationCode::new(
        "tutorial_step", 202, super::NotificationClass::Entity
    ));
    registry.register_code(NotificationCode::new(
        "achievement_unlocked", 203, super::NotificationClass::Entity
    ));
    
    registry
});

/// Validate that a code exists for the given class
pub fn validate_code(class: super::NotificationClass, code: &str) -> NotificationResult<()> {
    if CODE_REGISTRY.code_exists(class, code) {
        Ok(())
    } else {
        Err(NotificationError::InvalidCode {
            class: format!("{:?}", class),
            code: code.to_string(),
        })
    }
}

/// Get a code by class and name
pub fn get_code(class: super::NotificationClass, name: &str) -> Option<&NotificationCode> {
    CODE_REGISTRY.get_code(class, name)
}

/// Get all codes for a class
pub fn get_codes_for_class(class: super::NotificationClass) -> Vec<&NotificationCode> {
    CODE_REGISTRY.get_codes_for_class(class)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_registry() {
        let mut registry = CodeRegistry::new();
        let code = NotificationCode::new(
            "test_code", 999, super::NotificationClass::Server
        );
        
        registry.register_code(code.clone());
        
        assert!(registry.code_exists(super::NotificationClass::Server, "test_code"));
        assert!(!registry.code_exists(super::NotificationClass::Chat, "test_code"));
        
        let retrieved = registry.get_code(super::NotificationClass::Server, "test_code");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test_code");
    }

    #[test]
    fn test_global_registry() {
        assert!(validate_code(super::NotificationClass::Server, "connection_lost").is_ok());
        assert!(validate_code(super::NotificationClass::Chat, "new_message").is_ok());
        assert!(validate_code(super::NotificationClass::Server, "invalid_code").is_err());
    }

    #[test]
    fn test_get_codes_for_class() {
        let server_codes = get_codes_for_class(super::NotificationClass::Server);
        let chat_codes = get_codes_for_class(super::NotificationClass::Chat);
        
        assert!(server_codes.len() > 0);
        assert!(chat_codes.len() > 0);
        
        // Verify all server codes are actually server codes
        for code in server_codes {
            assert_eq!(code.class, super::NotificationClass::Server);
        }
    }
}