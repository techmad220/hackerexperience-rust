//! Web1 client model definitions

use serde::{Deserialize, Serialize};

/// Web1 specific actions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Web1Action {
    // Setup actions
    Setup,
    Bootstrap,
    
    // Tutorial actions
    TutorialNext,
    TutorialPrev,
    TutorialSkip,
    
    // Application actions
    LaunchBrowser,
    LaunchTerminal,
    LaunchTaskManager,
    LaunchLog,
    
    // System actions
    Login,
    Logout,
}

impl Web1Action {
    /// Get action as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Web1Action::Setup => "setup",
            Web1Action::Bootstrap => "bootstrap",
            Web1Action::TutorialNext => "tutorial_next",
            Web1Action::TutorialPrev => "tutorial_prev",
            Web1Action::TutorialSkip => "tutorial_skip",
            Web1Action::LaunchBrowser => "launch_browser",
            Web1Action::LaunchTerminal => "launch_terminal",
            Web1Action::LaunchTaskManager => "launch_task_manager",
            Web1Action::LaunchLog => "launch_log",
            Web1Action::Login => "login",
            Web1Action::Logout => "logout",
        }
    }
}

impl std::fmt::Display for Web1Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web1_action_display() {
        assert_eq!(Web1Action::Setup.as_str(), "setup");
        assert_eq!(Web1Action::LaunchTerminal.as_str(), "launch_terminal");
        assert_eq!(Web1Action::TutorialNext.as_str(), "tutorial_next");
    }
}