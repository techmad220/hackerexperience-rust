//! Helix Factor System - Game balance and calculation factors

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Factor calculation system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Factor {
    pub name: String,
    pub base_value: f64,
    pub modifiers: HashMap<String, f64>,
}

impl Factor {
    pub fn new(name: impl Into<String>, base_value: f64) -> Self {
        Self {
            name: name.into(),
            base_value,
            modifiers: HashMap::new(),
        }
    }

    pub fn add_modifier(&mut self, name: impl Into<String>, value: f64) {
        self.modifiers.insert(name.into(), value);
    }

    pub fn calculate(&self) -> f64 {
        self.modifiers.values().fold(self.base_value, |acc, &modifier| acc * modifier)
    }
}

/// Common factors for the game
pub mod factors {
    use super::*;

    pub fn cpu_factor() -> Factor {
        Factor::new("cpu", 1.0)
    }

    pub fn ram_factor() -> Factor {
        Factor::new("ram", 1.0)
    }

    pub fn network_factor() -> Factor {
        Factor::new("network", 1.0)
    }
}