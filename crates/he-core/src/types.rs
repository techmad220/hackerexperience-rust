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