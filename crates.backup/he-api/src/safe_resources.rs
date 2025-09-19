//! Safe resource management with overflow protection

use std::sync::Arc;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Units(pub u32);

impl Units {
    pub fn try_sub(&self, other: Self) -> Option<Self> {
        self.0.checked_sub(other.0).map(Units)
    }

    pub fn try_add(&self, other: Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Units)
    }

    pub fn min(&self, other: Self) -> Self {
        Units(self.0.min(other.0))
    }

    pub fn saturating_add(&self, other: Self) -> Self {
        Units(self.0.saturating_add(other.0))
    }

    pub fn saturating_sub(&self, other: Self) -> Self {
        Units(self.0.saturating_sub(other.0))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ResourceCaps {
    pub cpu: Units,
    pub ram: Units,
}

/// Safe resource allocation with overflow protection
pub fn allocate(
    mut want_cpu: Units,
    mut want_ram: Units,
    caps: ResourceCaps,
    used: (Units, Units),
) -> anyhow::Result<(Units, Units)> {
    // Fail fast if request is absurd
    if want_cpu.0 == 0 && want_ram.0 == 0 {
        anyhow::bail!("zero allocation request");
    }

    // Calculate free resources with underflow protection
    let free_cpu = caps.cpu
        .try_sub(used.0)
        .ok_or_else(|| anyhow::anyhow!("cpu underflow: cap {} < used {}", caps.cpu.0, used.0))?;

    let free_ram = caps.ram
        .try_sub(used.1)
        .ok_or_else(|| anyhow::anyhow!("ram underflow: cap {} < used {}", caps.ram.0, used.1))?;

    // Clamp wants to available resources
    want_cpu = want_cpu.min(free_cpu);
    want_ram = want_ram.min(free_ram);

    // Fail if we can't allocate anything meaningful
    if want_cpu.0 == 0 || want_ram.0 == 0 {
        anyhow::bail!("insufficient resources: want cpu={}, ram={} but only have cpu={}, ram={}",
            want_cpu.0, want_ram.0, free_cpu.0, free_ram.0);
    }

    Ok((want_cpu, want_ram))
}

/// Safely deallocate resources with saturation
pub fn deallocate(
    cpu_to_free: Units,
    ram_to_free: Units,
    caps: ResourceCaps,
    current_free: (Units, Units),
) -> (Units, Units) {
    // Use saturating_add to prevent overflow, then cap at hardware limits
    let new_free_cpu = current_free.0.saturating_add(cpu_to_free).min(caps.cpu);
    let new_free_ram = current_free.1.saturating_add(ram_to_free).min(caps.ram);

    (new_free_cpu, new_free_ram)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Queued,
    Running,
    Cancelling,
    Cancelled,
    Completed,
    Failed,
}

impl ProcessState {
    /// Check if this is a terminal state (can't transition further)
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Cancelled | Self::Completed | Self::Failed)
    }

    /// Validate state transition
    pub fn can_transition_to(&self, next: Self) -> bool {
        match (self, next) {
            // Queued can go to Running or Cancelled
            (Self::Queued, Self::Running) => true,
            (Self::Queued, Self::Cancelled) => true,

            // Running can go to Cancelling, Completed, or Failed
            (Self::Running, Self::Cancelling) => true,
            (Self::Running, Self::Completed) => true,
            (Self::Running, Self::Failed) => true,

            // Cancelling can go to Cancelled or Completed (if it finishes first)
            (Self::Cancelling, Self::Cancelled) => true,
            (Self::Cancelling, Self::Completed) => true,

            // Terminal states can't transition
            (s, _) if s.is_terminal() => false,

            // All other transitions are invalid
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_allocation() {
        let caps = ResourceCaps {
            cpu: Units(1000),
            ram: Units(2048),
        };

        let used = (Units(600), Units(1024));

        // Normal allocation should work
        let result = allocate(Units(200), Units(512), caps, used).unwrap();
        assert_eq!(result, (Units(200), Units(512)));

        // Over-allocation should be clamped
        let result = allocate(Units(500), Units(2000), caps, used).unwrap();
        assert_eq!(result, (Units(400), Units(1024)));

        // Zero allocation should fail
        assert!(allocate(Units(0), Units(0), caps, used).is_err());
    }

    #[test]
    fn test_overflow_protection() {
        let caps = ResourceCaps {
            cpu: Units(1000),
            ram: Units(2048),
        };

        // Test underflow protection
        let used = (Units(1500), Units(1024));
        assert!(allocate(Units(100), Units(100), caps, used).is_err());

        // Test deallocation with saturation
        let current_free = (Units(900), Units(1900));
        let freed = deallocate(Units(500), Units(500), caps, current_free);
        assert_eq!(freed, (Units(1000), Units(2048))); // Capped at hardware limits
    }

    #[test]
    fn test_process_state_transitions() {
        assert!(ProcessState::Queued.can_transition_to(ProcessState::Running));
        assert!(ProcessState::Running.can_transition_to(ProcessState::Cancelling));
        assert!(ProcessState::Cancelling.can_transition_to(ProcessState::Cancelled));

        // Can't go backwards
        assert!(!ProcessState::Running.can_transition_to(ProcessState::Queued));

        // Terminal states can't transition
        assert!(!ProcessState::Completed.can_transition_to(ProcessState::Running));
        assert!(!ProcessState::Cancelled.can_transition_to(ProcessState::Running));
    }
}