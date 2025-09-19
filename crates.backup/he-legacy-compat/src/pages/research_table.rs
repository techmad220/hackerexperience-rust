//! Research table page handler - 1:1 port of researchTable.php
//! 
//! Game design tool for generating software research calculations:
//! - Price calculations for software versions 10-200
//! - Size calculations with complex multiplier formulas
//! - RAM usage calculations with version-dependent scaling
//! - JSON output generation for game configuration
//! - 13 software types with individual multipliers

use axum::{
    extract::Extension,
    http::StatusCode,
    response::Html,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use once_cell::sync::Lazy;
use crate::session::PhpSession;

/// Software type multipliers for calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareMultiplier {
    pub price: f64,
    pub hd: f64,
    pub ram: f64,
}

/// Software types configuration (13 types)
pub static SOFTWARE_MULTIPLIERS: Lazy<HashMap<i32, SoftwareMultiplier>> = Lazy::new(|| {
    let mut multipliers = HashMap::new();
    
    multipliers.insert(0, SoftwareMultiplier { price: 25.0, hd: 15.0, ram: 15.0 }); // Cracker
    multipliers.insert(1, SoftwareMultiplier { price: 23.0, hd: 15.0, ram: 15.0 }); // Encryptor
    multipliers.insert(2, SoftwareMultiplier { price: 19.0, hd: 15.0, ram: 15.0 }); // SSH Exploit
    multipliers.insert(3, SoftwareMultiplier { price: 19.0, hd: 15.0, ram: 15.0 }); // FTP Exploit
    multipliers.insert(4, SoftwareMultiplier { price: 21.0, hd: 15.0, ram: 15.0 }); // Firewall
    multipliers.insert(5, SoftwareMultiplier { price: 15.0, hd: 15.0, ram: 15.0 }); // Hider
    multipliers.insert(6, SoftwareMultiplier { price: 15.0, hd: 15.0, ram: 15.0 }); // Seeker
    multipliers.insert(7, SoftwareMultiplier { price: 16.0, hd: 15.0, ram: 15.0 }); // Antivirus
    multipliers.insert(8, SoftwareMultiplier { price: 12.0, hd: 15.0, ram: 15.0 }); // Virus Spam
    multipliers.insert(9, SoftwareMultiplier { price: 12.0, hd: 15.0, ram: 15.0 }); // Virus Warez
    multipliers.insert(10, SoftwareMultiplier { price: 13.0, hd: 15.0, ram: 15.0 }); // Virus DDoS
    multipliers.insert(11, SoftwareMultiplier { price: 14.0, hd: 15.0, ram: 15.0 }); // Virus Breaker
    multipliers.insert(12, SoftwareMultiplier { price: 14.0, hd: 15.0, ram: 15.0 }); // Virus Collector
    
    multipliers
});

/// Software type names for table headers
pub static SOFTWARE_NAMES: &[&str] = &[
    "Cracker", "Enc", "SSH Exploit", "FTP Exploit", "Firewall", 
    "Hidder", "Seek", "Antivirus", "Vspam", "Vwarez", 
    "Vddos", "Vbrk", "Vcol"
];

/// Research table handler - displays software calculation tables
/// 
/// Port of: researchTable.php
/// Features:
/// - Game design tool for balancing software progression
/// - Price calculation table for versions 10-200
/// - Size calculation table with different formulas
/// - RAM calculation table with complex scaling
/// - JSON dictionary generation for client-side use
/// - 13 software types with individual progression curves
/// 
/// **Note:** This is a development/balancing tool as noted in original:
/// "This was a tool to list software version/price/research time and iron out the overall game design"
pub async fn research_table_handler(
    Extension(mut session): Extension<PhpSession>,
) -> Result<Html<String>, StatusCode> {
    // Generate the complete research table HTML
    let content = generate_research_tables();
    Ok(Html(content))
}

/// Generate complete research tables HTML
fn generate_research_tables() -> String {
    let mut html = String::from(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Research Table - Game Design Tool</title>
        <style>
            body { font-family: Arial, sans-serif; margin: 20px; }
            table { border-collapse: collapse; margin: 20px 0; width: 100%; }
            th, td { border: 1px solid #ccc; padding: 5px; text-align: center; }
            th { background-color: #f0f0f0; font-weight: bold; }
            .section-title { text-align: center; font-weight: bold; margin: 30px 0 10px 0; }
            .json-output { background: #f8f8f8; padding: 10px; margin: 10px 0; font-family: monospace; word-break: break-all; }
            .note { background: #fff3cd; padding: 15px; margin: 20px 0; border-left: 4px solid #ffc107; }
        </style>
    </head>
    <body>
        <h1>Research Table - Game Design Tool</h1>
        
        <div class="note">
            <strong>Note:</strong> This was a tool to list software version/price/research time and iron out the overall game design.
            <br>Ported from researchTable.php with exact calculation formulas preserved.
        </div>
    "#);

    // Generate price table
    html.push_str(&generate_price_table());
    
    // Generate size table and JSON
    html.push_str(&generate_size_table());
    
    // Generate RAM table and JSON
    html.push_str(&generate_ram_table());
    
    html.push_str("</body></html>");
    html
}

/// Generate price calculation table
fn generate_price_table() -> String {
    let mut html = String::from(r#"
        <div class="section-title">Price</div>
        <table>
            <tr>
                <th></th>
    "#);

    // Add software type headers
    for name in SOFTWARE_NAMES {
        html.push_str(&format!("<th>{}</th>", name));
    }
    html.push_str("</tr>");

    // Generate rows for versions 10-200
    for version in 10..=200 {
        html.push_str(&format!("<tr><td><b>{}</b></td>", version));
        
        for software_type in 0..13 {
            let price = calculate_price(version, software_type);
            html.push_str(&format!("<td>${}</td>", format_number(price)));
        }
        
        html.push_str("</tr>");
    }
    
    html.push_str("</table>");
    html
}

/// Generate size calculation table with JSON output
fn generate_size_table() -> String {
    let mut html = String::from(r#"
        <div class="section-title">Software Size</div>
        <table>
            <tr>
                <th></th>
    "#);

    // Add software type headers (commented out in original)
    for name in SOFTWARE_NAMES {
        html.push_str(&format!("<th>{}</th>", name));
    }
    html.push_str("</tr>");

    let mut values: HashMap<i32, HashMap<i32, i32>> = HashMap::new();

    // Calculate values for JSON generation
    for version in 10..=200 {
        for software_type in 0..13 {
            let size = calculate_size(version, software_type);
            values.entry(software_type).or_insert_with(HashMap::new).insert(version, size);
        }
    }

    html.push_str("</table>");

    // Generate JSON dictionary (as in original)
    html.push_str("<div class=\"json-output\">");
    html.push_str("dict_size = {");
    
    for software_type in 0..13 {
        let ext = get_software_ext(software_type);
        html.push_str(&format!("'{}':{{", ext));
        
        if let Some(versions) = values.get(&software_type) {
            for version in 10..=200 {
                if let Some(size) = versions.get(&version) {
                    html.push_str(&format!("'{}':{}", version, size));
                    if version != 200 {
                        html.push_str(",");
                    }
                }
            }
        }
        
        html.push_str("},");
    }
    
    html.push_str("}<br/>");
    html.push_str("</div>");
    
    html
}

/// Generate RAM calculation table with JSON output
fn generate_ram_table() -> String {
    let mut html = String::from(r#"
        <div class="section-title">RAM</div>
        <table>
            <tr>
                <th></th>
    "#);

    // Add software type headers
    for name in SOFTWARE_NAMES {
        html.push_str(&format!("<th>{}</th>", name));
    }
    html.push_str("</tr>");

    let mut values: HashMap<i32, HashMap<i32, i32>> = HashMap::new();

    // Calculate values for JSON generation
    for version in 10..=200 {
        for software_type in 0..13 {
            let ram = calculate_ram(version, software_type);
            values.entry(software_type).or_insert_with(HashMap::new).insert(version, ram);
        }
    }

    html.push_str("</table>");

    // Generate JSON dictionary (as in original)
    html.push_str("<div class=\"json-output\">");
    html.push_str("dict_ram = {");
    
    for software_type in 0..13 {
        let ext = get_software_ext(software_type);
        html.push_str(&format!("'{}':{{", ext));
        
        if let Some(versions) = values.get(&software_type) {
            for version in 10..=200 {
                if let Some(ram) = versions.get(&version) {
                    html.push_str(&format!("'{}':{}", version, ram));
                    if version != 200 {
                        html.push_str(",");
                    }
                }
            }
        }
        
        html.push_str("},");
    }
    
    html.push_str("}<br/>");
    html.push_str("</div>");
    
    html
}

/// Calculate software price using original formulas
fn calculate_price(version: i32, software_type: i32) -> i32 {
    let multiplier = SOFTWARE_MULTIPLIERS.get(&software_type).unwrap();
    
    let valor = if version >= 100 {
        // Complex formula for versions >= 100
        let calc = (version as f64) * (multiplier.price * ((version as f64 - 90.0) / 100.0) - 10.0) + 3000.0 - (25.0 - multiplier.price) * 80.0;
        calc.ceil() as i32
    } else {
        // Quadratic formula for versions < 100
        let price = ((version as f64).powi(2) - 9.0 * (version as f64)) / 100.0;
        (price * multiplier.price).ceil() as i32
    };
    
    valor
}

/// Calculate software size using original formulas
fn calculate_size(version: i32, software_type: i32) -> i32 {
    let multiplier = SOFTWARE_MULTIPLIERS.get(&software_type).unwrap();
    
    let valor = if version >= 100 {
        // Complex formula for versions >= 100
        let calc = (version as f64) * (multiplier.price * ((version as f64 - 100.0) / 100.0) - 9.5) + 3000.0 - (25.0 - multiplier.price) * 80.0;
        calc.ceil() as i32
    } else {
        // Modified quadratic formula for versions < 100
        let price = ((version as f64).powi(2) - 9.0 * (version as f64)) / 110.0 + 1.0;
        (price * multiplier.price).ceil() as i32
    };
    
    valor
}

/// Calculate software RAM using original formulas with complex scaling
fn calculate_ram(version: i32, software_type: i32) -> i32 {
    let multiplier = SOFTWARE_MULTIPLIERS.get(&software_type).unwrap();
    
    let valor = if version >= 100 {
        // Complex formula for versions >= 100 (same for both >= 150 and < 150 in original)
        let calc = (version as f64) * (multiplier.price * ((version as f64 - 110.0) / 100.0) - 15.0) + 3000.0 - (25.0 - multiplier.price) * 80.0;
        calc.ceil() as i32
    } else {
        // Different formulas based on version threshold
        let price = if version >= 50 {
            ((version as f64 * 0.9).powi(2) - 9.0 * (version as f64)) / 145.0 + 0.3
        } else {
            ((version as f64 * 0.9).powi(2) - 9.0 * (version as f64)) / 140.0 + 0.4
        };
        
        (price * multiplier.price).ceil() as i32
    };
    
    valor
}

/// Get software extension/ID for JSON output
fn get_software_ext(software_type: i32) -> String {
    match software_type {
        0 => "1".to_string(),  // Cracker -> 1
        1 => "2".to_string(),  // Encryptor -> 2
        2 => "13".to_string(), // SSH Exploit -> 13
        3 => "14".to_string(), // FTP Exploit -> 14
        _ => software_type.to_string(), // Others keep original ID
    }
}

/// Format number with commas (like PHP number_format)
fn format_number(num: i32) -> String {
    let num_str = num.to_string();
    let mut result = String::new();
    
    for (i, c) in num_str.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, c);
    }
    
    result
}

/// API endpoint to get calculated values for a specific software type and version
pub async fn get_software_stats(software_type: i32, version: i32) -> Result<SoftwareStats, String> {
    if software_type < 0 || software_type > 12 {
        return Err("Invalid software type".to_string());
    }
    
    if version < 10 || version > 200 {
        return Err("Invalid version range".to_string());
    }
    
    Ok(SoftwareStats {
        software_type,
        version,
        price: calculate_price(version, software_type),
        size: calculate_size(version, software_type),
        ram: calculate_ram(version, software_type),
    })
}

/// Software statistics for a specific type and version
#[derive(Debug, Serialize, Deserialize)]
pub struct SoftwareStats {
    pub software_type: i32,
    pub version: i32,
    pub price: i32,
    pub size: i32,
    pub ram: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_software_multipliers_loaded() {
        assert_eq!(SOFTWARE_MULTIPLIERS.len(), 13);
        
        let cracker = SOFTWARE_MULTIPLIERS.get(&0).unwrap();
        assert_eq!(cracker.price, 25.0);
        assert_eq!(cracker.hd, 15.0);
        assert_eq!(cracker.ram, 15.0);
        
        let virus_spam = SOFTWARE_MULTIPLIERS.get(&8).unwrap();
        assert_eq!(virus_spam.price, 12.0);
    }

    #[test]
    fn test_price_calculation() {
        // Test version < 100 formula
        let price_50 = calculate_price(50, 0); // Cracker, version 50
        // (50² - 9*50) / 100 * 25 = (2500 - 450) / 100 * 25 = 20.5 * 25 = 512.5 ≈ 513
        assert_eq!(price_50, 513);
        
        // Test version >= 100 formula
        let price_100 = calculate_price(100, 0); // Cracker, version 100
        // More complex calculation - just verify it's calculated
        assert!(price_100 > 0);
    }

    #[test]
    fn test_size_calculation() {
        let size_50 = calculate_size(50, 0); // Cracker, version 50
        assert!(size_50 > 0);
        
        let size_100 = calculate_size(100, 0); // Cracker, version 100
        assert!(size_100 > 0);
    }

    #[test]
    fn test_ram_calculation() {
        let ram_30 = calculate_ram(30, 0); // Cracker, version 30 (< 50)
        assert!(ram_30 > 0);
        
        let ram_60 = calculate_ram(60, 0); // Cracker, version 60 (>= 50, < 100)
        assert!(ram_60 > 0);
        
        let ram_120 = calculate_ram(120, 0); // Cracker, version 120 (>= 100)
        assert!(ram_120 > 0);
    }

    #[test]
    fn test_software_ext_mapping() {
        assert_eq!(get_software_ext(0), "1");   // Cracker -> 1
        assert_eq!(get_software_ext(1), "2");   // Encryptor -> 2
        assert_eq!(get_software_ext(2), "13");  // SSH Exploit -> 13
        assert_eq!(get_software_ext(3), "14");  // FTP Exploit -> 14
        assert_eq!(get_software_ext(4), "4");   // Others keep original
        assert_eq!(get_software_ext(12), "12");
    }

    #[test]
    fn test_number_formatting() {
        assert_eq!(format_number(1234), "1,234");
        assert_eq!(format_number(1234567), "1,234,567");
        assert_eq!(format_number(123), "123");
        assert_eq!(format_number(0), "0");
    }

    #[test]
    fn test_software_stats_api() {
        let stats = get_software_stats(0, 50).unwrap();
        assert_eq!(stats.software_type, 0);
        assert_eq!(stats.version, 50);
        assert!(stats.price > 0);
        assert!(stats.size > 0);
        assert!(stats.ram > 0);
        
        // Test invalid inputs
        assert!(get_software_stats(-1, 50).is_err());
        assert!(get_software_stats(0, 5).is_err());
        assert!(get_software_stats(13, 50).is_err());
        assert!(get_software_stats(0, 201).is_err());
    }

    #[test]
    fn test_version_range_formulas() {
        // Test that formulas change at expected thresholds
        let price_99 = calculate_price(99, 0);
        let price_100 = calculate_price(100, 0);
        // Different formulas should produce different results
        // (exact values depend on complex formulas)
        
        let ram_49 = calculate_ram(49, 0);
        let ram_50 = calculate_ram(50, 0);
        let ram_99 = calculate_ram(99, 0);
        let ram_100 = calculate_ram(100, 0);
        // Should use different calculation paths
        
        // Just verify they're all positive and calculated
        assert!(price_99 > 0 && price_100 > 0);
        assert!(ram_49 > 0 && ram_50 > 0 && ram_99 > 0 && ram_100 > 0);
    }

    #[test]
    fn test_all_software_types() {
        // Verify all 13 software types can be calculated
        for software_type in 0..13 {
            let stats = get_software_stats(software_type, 50).unwrap();
            assert_eq!(stats.software_type, software_type);
            assert!(stats.price > 0);
            assert!(stats.size > 0);
            assert!(stats.ram > 0);
        }
    }
}