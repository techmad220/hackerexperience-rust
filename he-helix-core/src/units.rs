//! Safe resource units with overflow protection

use serde::{Serialize, Deserialize};

/// Resource units that prevent arithmetic overflow
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Units(pub u64);

impl Units {
    /// Try to add units, returning None on overflow
    #[inline]
    pub fn try_add(self, rhs: Units) -> Option<Units> {
        self.0.checked_add(rhs.0).map(Units)
    }

    /// Try to subtract units, returning None on underflow
    #[inline]
    pub fn try_sub(self, rhs: Units) -> Option<Units> {
        self.0.checked_sub(rhs.0).map(Units)
    }

    /// Return the minimum of two units
    #[inline]
    pub fn min(self, cap: Units) -> Units {
        Units(self.0.min(cap.0))
    }

    /// Saturating addition (caps at u64::MAX)
    #[inline]
    pub fn saturating_add(self, rhs: Units) -> Units {
        Units(self.0.saturating_add(rhs.0))
    }

    /// Saturating subtraction (floors at 0)
    #[inline]
    pub fn saturating_sub(self, rhs: Units) -> Units {
        Units(self.0.saturating_sub(rhs.0))
    }
}

/// Hardware resource capacities
#[derive(Clone, Copy, Debug)]
pub struct ResourceCaps {
    pub cpu: Units,
    pub ram: Units,
}

/// Safely allocate resources with overflow protection
///
/// This function:
/// - Rejects zero allocations
/// - Checks for underflow when calculating free resources
/// - Clamps requests to available resources
/// - Ensures non-zero allocation or returns error
pub fn allocate(
    mut want_cpu: Units,
    mut want_ram: Units,
    caps: ResourceCaps,
    used: (Units, Units),
) -> anyhow::Result<(Units, Units)> {
    // Reject zero allocations
    if want_cpu.0 == 0 && want_ram.0 == 0 {
        anyhow::bail!("zero allocation requested");
    }

    // Calculate free resources with underflow protection
    let free_cpu = caps.cpu
        .try_sub(used.0)
        .ok_or_else(|| anyhow::anyhow!("cpu underflow: cap {} < used {}", caps.cpu.0, used.0))?;

    let free_ram = caps.ram
        .try_sub(used.1)
        .ok_or_else(|| anyhow::anyhow!("ram underflow: cap {} < used {}", caps.ram.0, used.1))?;

    // Clamp requests to available resources
    want_cpu = want_cpu.min(free_cpu);
    want_ram = want_ram.min(free_ram);

    // Ensure we can allocate something meaningful
    if want_cpu.0 == 0 || want_ram.0 == 0 {
        anyhow::bail!(
            "insufficient resources: want cpu={}, ram={} but only have cpu={}, ram={}",
            want_cpu.0, want_ram.0, free_cpu.0, free_ram.0
        );
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_units_arithmetic() {
        let a = Units(100);
        let b = Units(50);

        assert_eq!(a.try_add(b), Some(Units(150)));
        assert_eq!(a.try_sub(b), Some(Units(50)));
        assert_eq!(b.try_sub(a), None); // Underflow

        let max = Units(u64::MAX - 10);
        let overflow = Units(20);
        assert_eq!(max.try_add(overflow), None); // Overflow
        assert_eq!(max.saturating_add(overflow), Units(u64::MAX)); // Saturates
    }

    #[test]
    fn test_safe_allocation() {
        let caps = ResourceCaps {
            cpu: Units(1000),
            ram: Units(2048),
        };

        // Normal allocation
        let used = (Units(600), Units(1024));
        let result = allocate(Units(200), Units(512), caps, used).unwrap();
        assert_eq!(result, (Units(200), Units(512)));

        // Clamped allocation
        let result = allocate(Units(500), Units(2000), caps, used).unwrap();
        assert_eq!(result, (Units(400), Units(1024))); // Clamped to available

        // Underflow protection
        let bad_used = (Units(1500), Units(1024));
        assert!(allocate(Units(100), Units(100), caps, bad_used).is_err());
    }
}