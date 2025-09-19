//! Utility functions for cron jobs

use crate::error::{CronError, CronResult};
use chrono::{DateTime, Utc};
use std::process::Command;
use tracing::{info, error};
use uuid::Uuid;
use rand::Rng;

/// Generate a random string of the specified length using the given charset
pub fn rand_string(length: usize, charset: &str) -> String {
    let mut rng = rand::thread_rng();
    let charset_bytes = charset.as_bytes();
    let charset_len = charset_bytes.len();
    
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..charset_len);
            charset_bytes[idx] as char
        })
        .collect()
}

/// Generate a random string with default alphanumeric charset
pub fn rand_string_default(length: usize) -> String {
    const DEFAULT_CHARSET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    rand_string(length, DEFAULT_CHARSET)
}

/// Execute a shell command and return the output
pub async fn execute_command(command: &str, args: &[&str]) -> CronResult<String> {
    info!("Executing command: {} {}", command, args.join(" "));
    
    let output = Command::new(command)
        .args(args)
        .output()
        .map_err(|e| CronError::Runtime(format!("Failed to execute command: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("Command failed with stderr: {}", stderr);
        return Err(CronError::Runtime(format!("Command failed: {}", stderr)));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    info!("Command completed successfully");
    Ok(stdout)
}

/// Format a timestamp for use in backup file names
pub fn format_backup_timestamp(dt: DateTime<Utc>) -> String {
    dt.format("%Y%m%d-%H%M").to_string()
}

/// Convert IP address from integer to string format
pub fn long_to_ip(ip: u32) -> String {
    let a = (ip >> 24) & 0xFF;
    let b = (ip >> 16) & 0xFF;
    let c = (ip >> 8) & 0xFF;
    let d = ip & 0xFF;
    format!("{}.{}.{}.{}", a, b, c, d)
}

/// Generate a random bank account number
pub fn generate_bank_account() -> u64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(111111111..999999999)
}

/// Generate a shorter bank account number for certain operations
pub fn generate_short_bank_account() -> u64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(111111..999999)
}

/// Calculate percentage of a value
pub fn calculate_percentage(value: f64, percentage: f64) -> f64 {
    value * (percentage / 100.0)
}

/// Round a value to the nearest integer (ceiling)
pub fn round_up(value: f64) -> i64 {
    value.ceil() as i64
}

/// Get the file extension for a software type
pub fn get_software_extension(soft_type: i32) -> &'static str {
    match soft_type {
        1 => ".crc",
        2 => ".hash", 
        3 => ".scan",
        4 => ".fwl",
        5 => ".hdr",
        6 => ".skr",
        7 => ".av",
        8 => ".vspam",
        9 => ".vwarez",
        10 => ".vddos",
        11 => ".vcol",
        12 => ".vbrk",
        13 => ".exp",
        14 => ".exp",
        15 => ".nmap",
        16 => ".ana",
        17 => ".torrent",
        18 => ".exe",
        19 => ".exe",
        20 => ".vminer",
        29 => ".doom",
        30 => ".txt",
        31 => "",
        50 => ".nsa",
        51 => ".emp",
        90 => ".vdoom",
        96 => ".vminer",
        97 => ".vddos",
        98 => ".vwarez",
        99 => ".vspam",
        _ => ".unknown"
    }
}

/// Format version number with dots
pub fn dot_version(version: i32) -> String {
    let major = version / 10;
    let minor = version % 10;
    format!("{}.{}", major, minor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rand_string() {
        let result = rand_string(10, "ABC");
        assert_eq!(result.len(), 10);
        assert!(result.chars().all(|c| "ABC".contains(c)));
    }

    #[test]
    fn test_long_to_ip() {
        // Test IP conversion: 192.168.1.1 = 3232235777
        assert_eq!(long_to_ip(3232235777), "192.168.1.1");
    }

    #[test]
    fn test_dot_version() {
        assert_eq!(dot_version(15), "1.5");
        assert_eq!(dot_version(23), "2.3");
    }

    #[test]
    fn test_software_extension() {
        assert_eq!(get_software_extension(1), ".crc");
        assert_eq!(get_software_extension(30), ".txt");
    }
}