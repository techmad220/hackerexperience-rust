//! Process resource management

use crate::types::*;
use serde::{Deserialize, Serialize};
use std::ops::{Add, Sub};

/// Process-specific resource container
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ProcessResources {
    pub cpu: u32,
    pub ram: u64,
    pub hdd: u64,
    pub net: u32,
}

impl ProcessResources {
    pub fn new() -> Self {
        Self {
            cpu: 0,
            ram: 0,
            hdd: 0,
            net: 0,
        }
    }
    
    pub fn new_with_values(cpu: u32, ram: u64, hdd: u64, net: u32) -> Self {
        Self { cpu, ram, hdd, net }
    }
}

impl Default for ProcessResources {
    fn default() -> Self {
        Self::new()
    }
}

impl Add for ProcessResources {
    type Output = Self;
    
    fn add(self, other: Self) -> Self {
        Self {
            cpu: self.cpu + other.cpu,
            ram: self.ram + other.ram,
            hdd: self.hdd + other.hdd,
            net: self.net + other.net,
        }
    }
}

impl Sub for ProcessResources {
    type Output = Self;
    
    fn sub(self, other: Self) -> Self {
        Self {
            cpu: self.cpu.saturating_sub(other.cpu),
            ram: self.ram.saturating_sub(other.ram),
            hdd: self.hdd.saturating_sub(other.hdd),
            net: self.net.saturating_sub(other.net),
        }
    }
}