use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

// Type aliases for IDs
pub type UserId = i64;
pub type ProcessId = i64;
pub type SoftwareId = i64;
pub type HardwareId = i64;
pub type ClanId = i64;
pub type IpAddress = String;

// Process actions mapping from config.php
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(i32)]
pub enum ProcessAction {
    Download = 1,
    Upload = 2,
    Delete = 3,
    Hide = 4,
    Seek = 5,
    Collect = 6,     // DEPRECATED
    Av = 7,
    ELog = 8,
    DLog = 9,        // DEPRECATED
    Format = 10,
    Hack = 11,
    BankHack = 12,
    Install = 13,
    Uninstall = 14,
    PortScan = 15,
    HackXp = 16,
    Research = 17,
    UploadXhd = 18,
    DownloadXhd = 19,
    DeleteXhd = 20,
    Nmap = 22,
    Analyze = 23,
    InstallDoom = 24,
    ResetIp = 25,
    ResetPwd = 26,
    Ddos = 27,
    InstallWebserver = 28,
}

impl ProcessAction {
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            1 => Some(Self::Download),
            2 => Some(Self::Upload),
            3 => Some(Self::Delete),
            4 => Some(Self::Hide),
            5 => Some(Self::Seek),
            6 => Some(Self::Collect),
            7 => Some(Self::Av),
            8 => Some(Self::ELog),
            9 => Some(Self::DLog),
            10 => Some(Self::Format),
            11 => Some(Self::Hack),
            12 => Some(Self::BankHack),
            13 => Some(Self::Install),
            14 => Some(Self::Uninstall),
            15 => Some(Self::PortScan),
            16 => Some(Self::HackXp),
            17 => Some(Self::Research),
            18 => Some(Self::UploadXhd),
            19 => Some(Self::DownloadXhd),
            20 => Some(Self::DeleteXhd),
            22 => Some(Self::Nmap),
            23 => Some(Self::Analyze),
            24 => Some(Self::InstallDoom),
            25 => Some(Self::ResetIp),
            26 => Some(Self::ResetPwd),
            27 => Some(Self::Ddos),
            28 => Some(Self::InstallWebserver),
            _ => None,
        }
    }
    
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

// Process time configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessTimeConfig {
    pub download_min: u32,
    pub download_max: u32,
    pub upload_min: u32,
    pub upload_max: u32,
    pub delete_min: u32,
    pub delete_max: u32,
    pub hide_min: u32,
    pub hide_max: u32,
    pub seek_min: u32,
    pub seek_max: u32,
    pub install_min: u32,
    pub install_max: u32,
    pub av_min: u32,
    pub av_max: u32,
    pub log_min: u32,
    pub log_max: u32,
    pub format_min: u32,
    pub format_max: u32,
}

impl Default for ProcessTimeConfig {
    fn default() -> Self {
        // Values from original config.php
        Self {
            download_min: 20,
            download_max: 7200,
            upload_min: 20,
            upload_max: 7200,
            delete_min: 20,
            delete_max: 1200,
            hide_min: 5,
            hide_max: 1200,
            seek_min: 5,
            seek_max: 1200,
            install_min: 4,
            install_max: 1200,
            av_min: 60,
            av_max: 600,
            log_min: 4,
            log_max: 60,
            format_min: 1200,
            format_max: 3600,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_action_from_i32() {
        assert_eq!(ProcessAction::from_i32(1), Some(ProcessAction::Download));
        assert_eq!(ProcessAction::from_i32(2), Some(ProcessAction::Upload));
        assert_eq!(ProcessAction::from_i32(11), Some(ProcessAction::Hack));
        assert_eq!(ProcessAction::from_i32(27), Some(ProcessAction::Ddos));
        assert_eq!(ProcessAction::from_i32(999), None);
        assert_eq!(ProcessAction::from_i32(-1), None);
    }

    #[test]
    fn test_process_action_as_i32() {
        assert_eq!(ProcessAction::Download.as_i32(), 1);
        assert_eq!(ProcessAction::Upload.as_i32(), 2);
        assert_eq!(ProcessAction::Hack.as_i32(), 11);
        assert_eq!(ProcessAction::Ddos.as_i32(), 27);
    }

    #[test]
    fn test_process_action_roundtrip() {
        // Test that converting to i32 and back gives the same result
        let actions = vec![
            ProcessAction::Download,
            ProcessAction::Upload,
            ProcessAction::Hack,
            ProcessAction::BankHack,
            ProcessAction::Ddos,
            ProcessAction::InstallWebserver,
        ];

        for action in actions {
            let value = action.as_i32();
            let converted = ProcessAction::from_i32(value);
            assert_eq!(converted, Some(action));
        }
    }

    #[test]
    fn test_software_extension_from_type() {
        assert_eq!(SoftwareExtension::from_type(1), Some(SoftwareExtension::Crc));
        assert_eq!(SoftwareExtension::from_type(2), Some(SoftwareExtension::Crc));
        assert_eq!(SoftwareExtension::from_type(3), Some(SoftwareExtension::Crc));
        assert_eq!(SoftwareExtension::from_type(4), Some(SoftwareExtension::Fwl));
        assert_eq!(SoftwareExtension::from_type(5), Some(SoftwareExtension::Hdr));
        assert_eq!(SoftwareExtension::from_type(6), Some(SoftwareExtension::Skr));
        assert_eq!(SoftwareExtension::from_type(999), None);
    }

    #[test]
    fn test_software_extension_as_str() {
        assert_eq!(SoftwareExtension::Crc.as_str(), "crc");
        assert_eq!(SoftwareExtension::Exp.as_str(), "exp");
        assert_eq!(SoftwareExtension::Fwl.as_str(), "fwl");
        assert_eq!(SoftwareExtension::Hdr.as_str(), "hdr");
    }

    #[test]
    fn test_software_type_from_i32() {
        assert_eq!(SoftwareType::from_i32(1), Some(SoftwareType::Cracker));
        assert_eq!(SoftwareType::from_i32(4), Some(SoftwareType::Firewall));
        assert_eq!(SoftwareType::from_i32(10), Some(SoftwareType::Av));
        assert_eq!(SoftwareType::from_i32(26), Some(SoftwareType::Doom));
        assert_eq!(SoftwareType::from_i32(999), None);
    }

    #[test]
    fn test_software_type_as_i32() {
        assert_eq!(SoftwareType::Cracker.as_i32(), 1);
        assert_eq!(SoftwareType::Firewall.as_i32(), 4);
        assert_eq!(SoftwareType::Av.as_i32(), 10);
        assert_eq!(SoftwareType::Doom.as_i32(), 26);
    }

    #[test]
    fn test_process_timing_default() {
        let timing = ProcessTiming::default();
        assert_eq!(timing.hack_min, 10);
        assert_eq!(timing.hack_max, 600);
        assert_eq!(timing.download_min, 20);
        assert_eq!(timing.download_max, 7200);
        assert_eq!(timing.format_min, 1200);
        assert_eq!(timing.format_max, 3600);
    }

    #[test]
    fn test_type_aliases() {
        // Test that type aliases compile and can be used
        let _user_id: UserId = 123;
        let _process_id: ProcessId = 456;
        let _software_id: SoftwareId = 789;
        let _ip: IpAddress = "192.168.1.1".to_string();
    }

    #[test]
    fn test_process_action_serialization() {
        // Test that ProcessAction can be serialized and deserialized
        let action = ProcessAction::Hack;
        let json = serde_json::to_string(&action).map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        let deserialized: ProcessAction = serde_json::from_str(&json).map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        assert_eq!(action, deserialized);
    }

    #[test]
    fn test_software_type_serialization() {
        let software = SoftwareType::Firewall;
        let json = serde_json::to_string(&software).map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        let deserialized: SoftwareType = serde_json::from_str(&json).map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        assert_eq!(software, deserialized);
    }
}