//! Client model types and validation

use serde::{Deserialize, Serialize};
use std::fmt;
use crate::{ClientError, ClientResult};

/// Client types supported by the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Client {
    Web1,
    Web2,
    Mobile1,
    Mobile2,
    Unknown,
}

impl Client {
    /// Get all valid client types
    pub fn valid_clients() -> &'static [Client] {
        &[Client::Web1, Client::Web2, Client::Mobile1, Client::Mobile2, Client::Unknown]
    }

    /// Check if a client type is valid
    pub fn is_valid_client(client: &str) -> bool {
        matches!(client, "web1" | "web2" | "mobile1" | "mobile2" | "unknown")
    }

    /// Parse client from string
    pub fn from_str(s: &str) -> ClientResult<Self> {
        match s {
            "web1" => Ok(Client::Web1),
            "web2" => Ok(Client::Web2),
            "mobile1" => Ok(Client::Mobile1),
            "mobile2" => Ok(Client::Mobile2),
            "unknown" => Ok(Client::Unknown),
            _ => Err(ClientError::InvalidClientType {
                client_type: s.to_string(),
            }),
        }
    }

    /// Get client as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Client::Web1 => "web1",
            Client::Web2 => "web2",
            Client::Mobile1 => "mobile1",
            Client::Mobile2 => "mobile2",
            Client::Unknown => "unknown",
        }
    }

    /// Check if client is a web client
    pub fn is_web_client(&self) -> bool {
        matches!(self, Client::Web1 | Client::Web2)
    }

    /// Check if client is a mobile client
    pub fn is_mobile_client(&self) -> bool {
        matches!(self, Client::Mobile1 | Client::Mobile2)
    }
}

impl fmt::Display for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for Client {
    type Err = ClientError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Client::from_str(s)
    }
}

/// Client actions that can be performed
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    // Tutorial actions
    TutorialBootstrap,
    TutorialSetup,
    
    // App actions
    OpenApp(String),
    CloseApp(String),
    
    // System actions
    Shutdown,
    Reboot,
    
    // Custom actions
    Custom(String),
}

impl Action {
    /// Create a new custom action
    pub fn custom(action: impl Into<String>) -> Self {
        Action::Custom(action.into())
    }

    /// Get action as string representation
    pub fn as_str(&self) -> String {
        match self {
            Action::TutorialBootstrap => "tutorial_bootstrap".to_string(),
            Action::TutorialSetup => "tutorial_setup".to_string(),
            Action::OpenApp(app) => format!("open_app:{}", app),
            Action::CloseApp(app) => format!("close_app:{}", app),
            Action::Shutdown => "shutdown".to_string(),
            Action::Reboot => "reboot".to_string(),
            Action::Custom(action) => action.clone(),
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_from_str() {
        assert_eq!(Client::from_str("web1").unwrap(), Client::Web1);
        assert_eq!(Client::from_str("mobile1").unwrap(), Client::Mobile1);
        assert!(Client::from_str("invalid").is_err());
    }

    #[test]
    fn test_client_is_valid() {
        assert!(Client::is_valid_client("web1"));
        assert!(Client::is_valid_client("mobile2"));
        assert!(!Client::is_valid_client("invalid"));
    }

    #[test]
    fn test_client_types() {
        assert!(Client::Web1.is_web_client());
        assert!(!Client::Web1.is_mobile_client());
        assert!(Client::Mobile1.is_mobile_client());
        assert!(!Client::Mobile1.is_web_client());
    }

    #[test]
    fn test_action_display() {
        let action = Action::OpenApp("terminal".to_string());
        assert_eq!(action.as_str(), "open_app:terminal");
        
        let custom = Action::custom("custom_tutorial_step");
        assert_eq!(custom.as_str(), "custom_tutorial_step");
    }
}