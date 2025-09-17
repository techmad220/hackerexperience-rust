//! Comprehensive security module for HackerExperience
//!
//! Provides audit logging, intrusion detection, DDoS protection, and encryption at rest

pub mod audit;
pub mod intrusion;
pub mod ddos;
pub mod encryption;

pub use audit::{AuditLogger, SecurityEvent};
pub use intrusion::{IntrusionDetector, ThreatLevel};
pub use ddos::{DDoSProtection, ConnectionThrottle};
pub use encryption::{FieldEncryption, encrypt_field, decrypt_field};