use serde::{Deserialize, Serialize};
use regex::Regex;
use std::collections::HashMap;

/// System entity responsible for system-level operations and validations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct System {
    pub id: i64,
    pub name: String,
    pub version: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// System error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemError {
    // General errors
    InvalidGet,
    InvalidId,
    WrongPass,
    NoPermission,
    Custom(String),
    
    // Login/Auth errors
    LoginWrongUser,
    LoginWrongPassword,
    
    // Process errors
    CantCompleteProcess,
    InvalidAction,
    ProcNotFound,
    ProcAlreadyPaused,
    ProcNotPaused,
    ProcAlreadyCompleted,
    
    // Network errors
    InexistentServer,
    InexistentIp,
    InvalidIp,
    
    // Software errors
    NoInstaller,
    NoCollector,
    Downgrade,
    NotListed,
    NotInstalled,
    CantUninstall,
    InsufficientRam,
    AlreadyInstalled,
    NotExecutable,
    VirusAlreadyInstalled,
    NoSeeker,
    NoHaveSoft,
    SoftHidden,
    SoftAlreadyHave,
    InexistentSoftware,
    CantDelete,
    AlreadyListed,
    NoSoft,
    
    // Bank/Money errors
    BadMoney,
    NoAmount,
    BadAcc,
    InexistentAcc,
    BadBank,
    AutoTransfer,
    BankNotLogged,
    BankEmptyMoney,
    BankAccNotExists,
    BankInvalidAcc,
    BankInsufficientMoney,
    BankCantHack,
    BankAccFromInvalid,
    BankAccToInvalid,
    
    // Bitcoin errors
    BtcCantHack,
    BtcCantAdd,
    BtcAccInexistent,
    BtcAddressError,
    BtcInvalidAcc,
    BtcNoFunds,
    BtcAccFromInvalid,
    BtcAccToInvalid,
    BtcInvalidValue,
    
    // Virus/Doom errors
    VirusInvalidNew,
    DoomNotFound,
    DoomAlreadyInfected,
    DoomLogExist,
    DoomMoneyTransferFail,
    DoomClanOnly,
    
    // VPC errors
    VpcInexistentUser,
    VpcInvalidIp,
    VpcNoPower,
    VpcSpecInvalid,
    VpcLowFunds,
    VpcInvalidValue,
    VpcInvalidInfo,
    
    // Spy errors
    SpyInvalidPw,
    SpyInvalidUser,
    SpyNotInfected,
    SpyInvalidId,
    SpyNoUpdate,
    SpyCantFind,
    
    // VC (Verification) errors
    VcInvalidGame,
    VcCantJoin,
    VcBadPassword,
    VcIpLogged,
    VcInvalidVictim,
    VcBankAccNotFound,
    VcCharNotFound,
    VcMissionInvalidId,
    VcMissionNotFound,
    VcInvalidDdos,
    VcIpNotDdosd,
    VcInvalidResearch,
    
    // Password change errors
    PwdchangeWrongPassword,
    PwdchangePasswordTooWeak,
    PwdresetNoTime,
    
    // DDOS errors
    DdosInexistent,
    DdosAlredyDdosd,
    DdosCantBypass,
    
    // Edit virus errors
    EditVirusInvalidId,
    EditVirusInexistent,
    EditVirusInvalidVersion,
    EditVirusInvalidTime,
    EditVirusInvalidRunning,
    
    // Chat errors
    ChatNoMessage,
    
    // Certificate errors
    CertDuplicate,
    
    // Hardware errors
    BadXhd,
    DownloadInsufficientHd,
    UploadInsufficientHd,
    
    // License/Certification
    NoLicense,
    NoCertification,
    
    // Upload errors
    UploadSoftAlreadyHave,
    
    // Hide errors
    HideInstalledSoftware,
    
    // Cracker errors
    BadCracker,
    BahBadCracker,
    BahAlreadyCracked,
    
    // Experience errors
    HxpBadExp,
    HxpNoExp,
    NoFtpExp,
    NoSshExp,
    HxpNoScan,
    
    // Nmap errors
    NoNmapVictim,
    
    // IP reset errors
    IpresetNoTime,
    
    // Folder errors
    FolderInexistentSoftware,
    FolderAlreadyHave,
    FolderInexistent,
}

impl System {
    /// Creates a new System instance
    pub fn new() -> Self {
        Self {
            id: 0,
            name: String::from("HackerExperience"),
            version: String::from("1.0.0"),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    /// Get system configuration
    pub fn get_config(&self) -> HashMap<String, String> {
        let mut config = HashMap::new();
        config.insert("version".to_string(), self.version.clone());
        config.insert("name".to_string(), self.name.clone());
        config
    }

    /// Validate a value based on its type
    pub fn validate(&self, value_type: &str, value: &str) -> bool {
        match value_type {
            "username" => self.validate_username(value),
            "email" => self.validate_email(value),
            "ip" => self.validate_ip(value),
            _ => true,
        }
    }

    /// Validates username format
    fn validate_username(&self, username: &str) -> bool {
        if username.is_empty() || username.len() > 30 {
            return false;
        }
        
        let re = Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
        re.is_match(username)
    }

    /// Validates IP address format
    fn validate_ip(&self, ip: &str) -> bool {
        let parts: Vec<&str> = ip.split('.').collect();
        if parts.len() != 4 {
            return false;
        }
        
        for part in parts {
            match part.parse::<u8>() {
                Ok(_) => continue,
                Err(_) => return false,
            }
        }
        
        true
    }

    /// Validates email address format
    fn validate_email(&self, email: &str) -> bool {
        let re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        re.is_match(email)
    }
}

impl Default for System {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&str> for SystemError {
    fn from(s: &str) -> Self {
        SystemError::Custom(s.to_string())
    }
}