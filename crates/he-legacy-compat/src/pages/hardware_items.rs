//! Hardware items configuration - 1:1 port of hardwareItens.php
//! 
//! Static hardware item definitions including:
//! - CPU processors with power and pricing
//! - RAM modules with capacity and pricing  
//! - HDD/SSD storage with capacity and pricing
//! - Network cards with speed and pricing
//! - External HD with capacity and pricing

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Hardware item definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareItem {
    pub name: String,
    pub description: String,
    pub power: i32,    // Performance/capacity value
    pub price: i32,    // Purchase price
}

/// CPU processor items - processing power units
pub static CPU_ITEMS: Lazy<HashMap<i32, HardwareItem>> = Lazy::new(|| {
    let mut items = HashMap::new();
    
    items.insert(1, HardwareItem {
        name: "i286".to_string(),
        description: "i286 bla bla".to_string(),
        power: 500,
        price: 99,
    });
    
    items.insert(2, HardwareItem {
        name: "P2".to_string(),
        description: "i7 irra".to_string(),
        power: 1000,
        price: 99,
    });
    
    items.insert(3, HardwareItem {
        name: "P3".to_string(),
        description: "P3".to_string(),
        power: 1500,
        price: 150,
    });
    
    items.insert(4, HardwareItem {
        name: "P4".to_string(),
        description: "P4".to_string(),
        power: 2000,
        price: 300,
    });
    
    items.insert(5, HardwareItem {
        name: "Dual Core".to_string(),
        description: "i7 irra".to_string(),
        power: 2500,
        price: 600,
    });
    
    items.insert(6, HardwareItem {
        name: "Quad Core".to_string(),
        description: "i7 irra".to_string(),
        power: 3000,
        price: 1500,
    });
    
    items.insert(7, HardwareItem {
        name: "AMD FX 8350".to_string(),
        description: "i7 irra".to_string(),
        power: 3500,
        price: 2000,
    });
    
    items.insert(8, HardwareItem {
        name: "i7 4990k".to_string(),
        description: "i7 irra".to_string(),
        power: 4000,
        price: 5000,
    });
    
    items
});

/// RAM memory items - capacity in MB
pub static RAM_ITEMS: Lazy<HashMap<i32, HardwareItem>> = Lazy::new(|| {
    let mut items = HashMap::new();
    
    items.insert(1, HardwareItem {
        name: "Alzheimer".to_string(),
        description: "Better than nothing...".to_string(),
        power: 256,
        price: 100,
    });
    
    items.insert(2, HardwareItem {
        name: "King Ton".to_string(),
        description: "...".to_string(),
        power: 512,
        price: 99,
    });
    
    items.insert(3, HardwareItem {
        name: "Cross 1G".to_string(),
        description: "...".to_string(),
        power: 1024,
        price: 500,
    });
    
    items.insert(4, HardwareItem {
        name: "Elephpant".to_string(),
        description: "...".to_string(),
        power: 2048,
        price: 2500,
    });
    
    items
});

/// HDD/SSD storage items - capacity in MB
pub static HDD_ITEMS: Lazy<HashMap<i32, HardwareItem>> = Lazy::new(|| {
    let mut items = HashMap::new();
    
    items.insert(1, HardwareItem {
        name: "miniSSD 100".to_string(),
        description: "Old...".to_string(),
        power: 100,
        price: 99,
    });
    
    items.insert(2, HardwareItem {
        name: "miniSSD 500".to_string(),
        description: "Old...".to_string(),
        power: 500,
        price: 99,
    });
    
    items.insert(3, HardwareItem {
        name: "SSD1".to_string(),
        description: "...".to_string(),
        power: 1000,
        price: 300,
    });
    
    items.insert(4, HardwareItem {
        name: "SSD2".to_string(),
        description: "...".to_string(),
        power: 2000,
        price: 1000,
    });
    
    items.insert(5, HardwareItem {
        name: "SSD5".to_string(),
        description: "...".to_string(),
        power: 5000,
        price: 3000,
    });
    
    items.insert(6, HardwareItem {
        name: "SSD10".to_string(),
        description: "...".to_string(),
        power: 10000,
        price: 8000,
    });
    
    items
});

/// Network card items - speed in Mbps
pub static NET_ITEMS: Lazy<HashMap<i32, HardwareItem>> = Lazy::new(|| {
    let mut items = HashMap::new();
    
    items.insert(1, HardwareItem {
        name: "NET1".to_string(),
        description: "Better than nothing...".to_string(),
        power: 1,
        price: 100,
    });
    
    items.insert(2, HardwareItem {
        name: "NET2".to_string(),
        description: "...".to_string(),
        power: 2,
        price: 50,
    });
    
    items.insert(3, HardwareItem {
        name: "NET4".to_string(),
        description: "...".to_string(),
        power: 4,
        price: 99,
    });
    
    items.insert(4, HardwareItem {
        name: "MEGA10".to_string(),
        description: "...".to_string(),
        power: 10,
        price: 250,
    });
    
    items.insert(5, HardwareItem {
        name: "MEGA25".to_string(),
        description: "...".to_string(),
        power: 25,
        price: 1000,
    });
    
    items.insert(6, HardwareItem {
        name: "MEGA50".to_string(),
        description: "...".to_string(),
        power: 50,
        price: 2500,
    });
    
    items.insert(7, HardwareItem {
        name: "BROAD100".to_string(),
        description: "...".to_string(),
        power: 100,
        price: 10000,
    });
    
    items.insert(8, HardwareItem {
        name: "BROAD250".to_string(),
        description: "...".to_string(),
        power: 250,
        price: 25000,
    });
    
    items.insert(9, HardwareItem {
        name: "ULTRA500".to_string(),
        description: "...".to_string(),
        power: 500,
        price: 50000,
    });
    
    items.insert(10, HardwareItem {
        name: "ULTRA1000".to_string(),
        description: "...".to_string(),
        power: 1000,
        price: 100000,
    });
    
    items
});

/// External HD items - capacity in MB
pub static XHD_ITEMS: Lazy<HashMap<i32, HardwareItem>> = Lazy::new(|| {
    let mut items = HashMap::new();
    
    items.insert(0, HardwareItem {
        name: "".to_string(),
        description: "".to_string(),
        power: 0,
        price: 500,
    });
    
    items.insert(1, HardwareItem {
        name: "Basic X".to_string(),
        description: "...".to_string(),
        power: 200,
        price: 50,
    });
    
    items.insert(2, HardwareItem {
        name: "Inter X".to_string(),
        description: "...".to_string(),
        power: 500,
        price: 99,
    });
    
    items.insert(3, HardwareItem {
        name: "Mega X".to_string(),
        description: "...".to_string(),
        power: 1000,
        price: 500,
    });
    
    items
});

/// Hardware item type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HardwareType {
    CPU,
    RAM,
    HDD,
    NET,
    XHD,
}

/// Get hardware item by type and ID
pub fn get_hardware_item(hardware_type: HardwareType, item_id: i32) -> Option<&'static HardwareItem> {
    match hardware_type {
        HardwareType::CPU => CPU_ITEMS.get(&item_id),
        HardwareType::RAM => RAM_ITEMS.get(&item_id),
        HardwareType::HDD => HDD_ITEMS.get(&item_id),
        HardwareType::NET => NET_ITEMS.get(&item_id),
        HardwareType::XHD => XHD_ITEMS.get(&item_id),
    }
}

/// Get all hardware items of a specific type
pub fn get_hardware_items(hardware_type: HardwareType) -> &'static HashMap<i32, HardwareItem> {
    match hardware_type {
        HardwareType::CPU => &CPU_ITEMS,
        HardwareType::RAM => &RAM_ITEMS,
        HardwareType::HDD => &HDD_ITEMS,
        HardwareType::NET => &NET_ITEMS,
        HardwareType::XHD => &XHD_ITEMS,
    }
}

/// Calculate total hardware power for a system
pub fn calculate_system_power(
    cpu_id: i32,
    ram_id: i32,
    hdd_id: i32,
    net_id: i32,
) -> Option<i32> {
    let cpu_power = CPU_ITEMS.get(&cpu_id)?.power;
    let ram_power = RAM_ITEMS.get(&ram_id)?.power;
    let hdd_power = HDD_ITEMS.get(&hdd_id)?.power;
    let net_power = NET_ITEMS.get(&net_id)?.power;
    
    Some(cpu_power + ram_power + hdd_power + net_power)
}

/// Calculate total hardware cost for a system
pub fn calculate_system_cost(
    cpu_id: i32,
    ram_id: i32,
    hdd_id: i32,
    net_id: i32,
) -> Option<i32> {
    let cpu_cost = CPU_ITEMS.get(&cpu_id)?.price;
    let ram_cost = RAM_ITEMS.get(&ram_id)?.price;
    let hdd_cost = HDD_ITEMS.get(&hdd_id)?.price;
    let net_cost = NET_ITEMS.get(&net_id)?.price;
    
    Some(cpu_cost + ram_cost + hdd_cost + net_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_items_loaded() {
        assert_eq!(CPU_ITEMS.len(), 8);
        
        let i286 = CPU_ITEMS.get(&1).unwrap();
        assert_eq!(i286.name, "i286");
        assert_eq!(i286.power, 500);
        assert_eq!(i286.price, 99);
        
        let i7 = CPU_ITEMS.get(&8).unwrap();
        assert_eq!(i7.name, "i7 4990k");
        assert_eq!(i7.power, 4000);
        assert_eq!(i7.price, 5000);
    }

    #[test]
    fn test_ram_items_loaded() {
        assert_eq!(RAM_ITEMS.len(), 4);
        
        let alzheimer = RAM_ITEMS.get(&1).unwrap();
        assert_eq!(alzheimer.name, "Alzheimer");
        assert_eq!(alzheimer.power, 256);
        assert_eq!(alzheimer.price, 100);
        
        let elephpant = RAM_ITEMS.get(&4).unwrap();
        assert_eq!(elephpant.name, "Elephpant");
        assert_eq!(elephpant.power, 2048);
        assert_eq!(elephpant.price, 2500);
    }

    #[test]
    fn test_hdd_items_loaded() {
        assert_eq!(HDD_ITEMS.len(), 6);
        
        let mini_ssd = HDD_ITEMS.get(&1).unwrap();
        assert_eq!(mini_ssd.name, "miniSSD 100");
        assert_eq!(mini_ssd.power, 100);
        assert_eq!(mini_ssd.price, 99);
        
        let ssd10 = HDD_ITEMS.get(&6).unwrap();
        assert_eq!(ssd10.name, "SSD10");
        assert_eq!(ssd10.power, 10000);
        assert_eq!(ssd10.price, 8000);
    }

    #[test]
    fn test_net_items_loaded() {
        assert_eq!(NET_ITEMS.len(), 10);
        
        let net1 = NET_ITEMS.get(&1).unwrap();
        assert_eq!(net1.name, "NET1");
        assert_eq!(net1.power, 1);
        assert_eq!(net1.price, 100);
        
        let ultra1000 = NET_ITEMS.get(&10).unwrap();
        assert_eq!(ultra1000.name, "ULTRA1000");
        assert_eq!(ultra1000.power, 1000);
        assert_eq!(ultra1000.price, 100000);
    }

    #[test]
    fn test_xhd_items_loaded() {
        assert_eq!(XHD_ITEMS.len(), 4);
        
        let empty = XHD_ITEMS.get(&0).unwrap();
        assert_eq!(empty.name, "");
        assert_eq!(empty.power, 0);
        assert_eq!(empty.price, 500);
        
        let mega_x = XHD_ITEMS.get(&3).unwrap();
        assert_eq!(mega_x.name, "Mega X");
        assert_eq!(mega_x.power, 1000);
        assert_eq!(mega_x.price, 500);
    }

    #[test]
    fn test_get_hardware_item() {
        let cpu = get_hardware_item(HardwareType::CPU, 1).unwrap();
        assert_eq!(cpu.name, "i286");
        
        let ram = get_hardware_item(HardwareType::RAM, 1).unwrap();
        assert_eq!(ram.name, "Alzheimer");
        
        let invalid = get_hardware_item(HardwareType::CPU, 999);
        assert!(invalid.is_none());
    }

    #[test]
    fn test_system_calculations() {
        // Test system with basic components (all ID 1)
        let power = calculate_system_power(1, 1, 1, 1).unwrap();
        let cost = calculate_system_cost(1, 1, 1, 1).unwrap();
        
        // i286(500) + Alzheimer(256) + miniSSD 100(100) + NET1(1) = 857
        assert_eq!(power, 857);
        
        // i286(99) + Alzheimer(100) + miniSSD 100(99) + NET1(100) = 398
        assert_eq!(cost, 398);
    }

    #[test]
    fn test_get_hardware_items() {
        let cpu_items = get_hardware_items(HardwareType::CPU);
        assert_eq!(cpu_items.len(), 8);
        
        let ram_items = get_hardware_items(HardwareType::RAM);
        assert_eq!(ram_items.len(), 4);
    }

    #[test]
    fn test_hardware_progression() {
        // Test that higher-tier items generally have higher power and cost
        let basic_cpu = CPU_ITEMS.get(&1).unwrap();
        let advanced_cpu = CPU_ITEMS.get(&8).unwrap();
        
        assert!(advanced_cpu.power > basic_cpu.power);
        assert!(advanced_cpu.price > basic_cpu.price);
        
        let basic_ram = RAM_ITEMS.get(&1).unwrap();
        let advanced_ram = RAM_ITEMS.get(&4).unwrap();
        
        assert!(advanced_ram.power > basic_ram.power);
        assert!(advanced_ram.price > basic_ram.price);
    }
}